//! Market Data Service
//!
//! Ingests FX market data feeds, normalizes quotes, and publishes
//! L2 + L3 order books via WebSocket and REST APIs.

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{Json, Response},
    routing::get,
    Router,
};
use fx_md::{MarketDataFeed, Quote};
use fx_utils::{Price, Quantity, Result};
use prometheus::{Encoder, TextEncoder};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, Level};

#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    feed: Arc<MarketDataFeed>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Market Data Service");

    // Initialize market data feed
    let (feed, _quote_rx) = MarketDataFeed::new("EURUSD".to_string());
    let feed = Arc::new(feed);

    // Start mock feed generator
    let feed_clone = feed.clone();
    tokio::spawn(async move {
        generate_mock_feed(feed_clone).await;
    });

    // Setup Prometheus metrics exporter
    // Note: prometheus_exporter API may vary by version
    // For now, metrics are exposed via the /metrics endpoint

    // Setup HTTP server
    let state = AppState { feed };
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .route("/quote", get(get_latest_quote))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Market Data Service listening on http://0.0.0.0:8081");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "market-data-service" }))
}

async fn metrics_handler() -> Response<Body> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Failed to encode metrics: {}", e)))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}

async fn get_latest_quote(State(_state): State<AppState>) -> Json<serde_json::Value> {
    // In a real implementation, this would return the latest quote
    Json(json!({
        "instrument": "EURUSD",
        "bid": 1.0850,
        "ask": 1.0852,
        "timestamp": fx_utils::time::now_nanos()
    }))
}

async fn generate_mock_feed(feed: Arc<MarketDataFeed>) {
    let mut counter = 0u64;
    loop {
        let base_price = 1.0850 + (counter as f64 * 0.0001).sin() * 0.001;
        let quote = Quote {
            instrument: "EURUSD".to_string(),
            bid_price: Price::from_decimal(base_price - 0.0001, 4),
            ask_price: Price::from_decimal(base_price + 0.0001, 4),
            bid_size: Quantity(1_000_000),
            ask_size: Quantity(1_000_000),
            timestamp_ns: fx_utils::time::now_nanos(),
        };

        if let Err(e) = feed.publish(quote) {
            tracing::error!(error = %e, "Failed to publish quote");
        }

        counter += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
