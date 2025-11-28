//! Market data feed interface

use crate::quote::Quote;
use fx_utils::Result;
use tokio::sync::broadcast;
use tracing::info;

/// Market data feed that generates or ingests quotes
pub struct MarketDataFeed {
    instrument: String,
    tx: broadcast::Sender<Quote>,
}

impl MarketDataFeed {
    pub fn new(instrument: String) -> (Self, broadcast::Receiver<Quote>) {
        let (tx, _) = broadcast::channel(1024);
        let rx = tx.subscribe();
        (
            Self {
                instrument,
                tx: tx.clone(),
            },
            rx,
        )
    }

    pub fn publish(&self, quote: Quote) -> Result<()> {
        info!(
            instrument = %self.instrument,
            bid = quote.bid_price.0,
            ask = quote.ask_price.0,
            "Publishing quote"
        );
        self.tx
            .send(quote)
            .map_err(|e| fx_utils::Error::Internal(format!("Failed to publish quote: {}", e)))?;
        Ok(())
    }
}
