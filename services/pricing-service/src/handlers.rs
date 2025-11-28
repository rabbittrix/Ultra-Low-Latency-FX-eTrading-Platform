//! HTTP handlers for pricing service

use axum::extract::State;
use axum::response::Json;
use fx_md::Quote;
use fx_pricing::PricingEngine;
use fx_utils::{Price, Quantity};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuoteRequest {
    pub instrument: String,
    pub bid_price: u64,
    pub ask_price: u64,
    pub bid_size: u64,
    pub ask_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuoteResponse {
    pub instrument: String,
    pub bid_price: u64,
    pub ask_price: u64,
    pub mid_price: u64,
    pub spread: u64,
}

/// Application state for pricing service
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<PricingEngine>>,
    pub price_tx: broadcast::Sender<Quote>,
}

impl AppState {
    pub fn new(engine: Arc<Mutex<PricingEngine>>) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            engine,
            price_tx: tx,
        }
    }

    /// Publish a pricing update
    pub fn publish_price(&self, quote: Quote) {
        let _ = self.price_tx.send(quote);
    }
}

#[axum::debug_handler]
pub async fn calculate_prices(
    State(state): State<AppState>,
    Json(req): Json<PriceQuoteRequest>,
) -> Json<PriceQuoteResponse> {
    let base_quote = Quote {
        instrument: req.instrument.clone(),
        bid_price: Price(req.bid_price),
        ask_price: Price(req.ask_price),
        bid_size: Quantity(req.bid_size),
        ask_size: Quantity(req.ask_size),
        timestamp_ns: fx_utils::time::now_nanos(),
    };

    // tokio::sync::MutexGuard is Send, so we can hold it across await points
    let engine_guard = state.engine.lock().await;
    let result = engine_guard.calculate_prices(&base_quote).await;

    match result {
        Ok((bid, ask)) => {
            let mid = (bid.0 + ask.0) / 2;
            let spread = ask.0.saturating_sub(bid.0);

            // Publish pricing update via WebSocket
            let updated_quote = Quote {
                instrument: req.instrument.clone(),
                bid_price: bid,
                ask_price: ask,
                bid_size: Quantity(req.bid_size),
                ask_size: Quantity(req.ask_size),
                timestamp_ns: fx_utils::time::now_nanos(),
            };
            state.publish_price(updated_quote);

            Json(PriceQuoteResponse {
                instrument: req.instrument,
                bid_price: bid.0,
                ask_price: ask.0,
                mid_price: mid,
                spread,
            })
        }
        Err(_) => {
            // Fallback to original prices on error
            let mid = (req.bid_price + req.ask_price) / 2;
            let spread = req.ask_price.saturating_sub(req.bid_price);
            Json(PriceQuoteResponse {
                instrument: req.instrument,
                bid_price: req.bid_price,
                ask_price: req.ask_price,
                mid_price: mid,
                spread,
            })
        }
    }
}
