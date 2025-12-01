//! Yahoo Finance data feed integration
//!
//! Fetches real-time FX quotes from Yahoo Finance API
//! Note: Yahoo Finance provides delayed data (15-20 min) for free tier

use crate::Quote;
use fx_utils::{Price, Quantity, Result};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

/// Convert FX pair to Yahoo Finance format
/// Example: "EURUSD" -> "EURUSD=X"
fn to_yahoo_symbol(instrument: &str) -> String {
    if instrument.ends_with("=X") {
        instrument.to_string()
    } else {
        format!("{}={}", instrument, "X")
    }
}

/// Fetch quote from Yahoo Finance API
async fn fetch_yahoo_quote(symbol: &str) -> Result<Quote> {
    let yahoo_symbol = to_yahoo_symbol(symbol);
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d",
        yahoo_symbol
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| fx_utils::Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Yahoo Finance API error: {}", e)))?;

    if !response.status().is_success() {
        return Err(fx_utils::Error::Internal(format!(
            "Yahoo Finance API returned status: {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Failed to parse JSON: {}", e)))?;

    // Extract quote data from Yahoo Finance response
    let result = json
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| {
            fx_utils::Error::Internal("Invalid Yahoo Finance response format".to_string())
        })?;

    let meta = result
        .get("meta")
        .ok_or_else(|| fx_utils::Error::Internal("Missing meta in response".to_string()))?;

    let regular_price = meta
        .get("regularMarketPrice")
        .and_then(|p| p.as_f64())
        .ok_or_else(|| fx_utils::Error::Internal("Missing price in response".to_string()))?;

    // Yahoo Finance doesn't always provide bid/ask for FX, so we calculate from mid price
    let bid = meta
        .get("bid")
        .and_then(|b| b.as_f64())
        .unwrap_or(regular_price - 0.0001);
    let ask = meta
        .get("ask")
        .and_then(|a| a.as_f64())
        .unwrap_or(regular_price + 0.0001);

    let bid_size = meta
        .get("bidSize")
        .and_then(|s| s.as_u64())
        .unwrap_or(1_000_000);
    let ask_size = meta
        .get("askSize")
        .and_then(|s| s.as_u64())
        .unwrap_or(1_000_000);

    // Convert to our Quote format (4 decimal places for FX)
    Ok(Quote {
        instrument: symbol.to_string(),
        bid_price: Price::from_decimal(bid, 4),
        ask_price: Price::from_decimal(ask, 4),
        bid_size: Quantity(bid_size),
        ask_size: Quantity(ask_size),
        timestamp_ns: fx_utils::time::now_nanos(),
    })
}

/// Generate real-time feed from Yahoo Finance
pub async fn generate_yahoo_feed(
    feed: Arc<fx_md::MarketDataFeed>,
    instruments: Vec<String>,
    update_interval_ms: u64,
) {
    info!(
        "Starting Yahoo Finance feed for instruments: {:?}",
        instruments
    );
    let mut error_count = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 10;

    loop {
        for instrument in &instruments {
            match fetch_yahoo_quote(instrument).await {
                Ok(quote) => {
                    error_count = 0; // Reset error count on success
                    if let Err(e) = feed.publish(quote) {
                        error!(error = %e, instrument = %instrument, "Failed to publish Yahoo Finance quote");
                    }
                }
                Err(e) => {
                    error_count += 1;
                    warn!(
                        error = %e,
                        instrument = %instrument,
                        error_count = error_count,
                        "Failed to fetch Yahoo Finance quote"
                    );

                    if error_count >= MAX_CONSECUTIVE_ERRORS {
                        error!(
                            "Too many consecutive errors ({}), falling back to mock data",
                            error_count
                        );
                        return; // Exit and let mock feed take over
                    }
                }
            }

            // Small delay between instruments
            sleep(Duration::from_millis(100)).await;
        }

        // Wait before next update cycle
        sleep(Duration::from_millis(update_interval_ms)).await;
    }
}
