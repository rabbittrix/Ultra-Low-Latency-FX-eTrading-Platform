//! WebSocket handler for publishing pricing updates

use crate::handlers::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::State;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tracing::{error, info};

/// WebSocket handler for pricing updates
pub async fn pricing_websocket_handler(ws: WebSocket, State(state): State<AppState>) {
    let (mut sender, mut receiver) = ws.split();

    // Subscribe to pricing updates
    let mut rx = state.price_tx.subscribe();
    info!("New WebSocket client connected for pricing updates");

    // Spawn task to send pricing updates
    let mut send_task = tokio::spawn(async move {
        while let Ok(quote) = rx.recv().await {
            let quote_json = json!({
                "instrument": quote.instrument,
                "bid_price": quote.bid_price.0,
                "ask_price": quote.ask_price.0,
                "mid_price": quote.mid_price().0,
                "spread": quote.ask_price.0.saturating_sub(quote.bid_price.0),
                "bid_size": quote.bid_size.0,
                "ask_size": quote.ask_size.0,
                "timestamp": quote.timestamp_ns
            });

            if let Err(e) = sender
                .send(Message::Text(serde_json::to_string(&quote_json).unwrap()))
                .await
            {
                error!(error = %e, "Failed to send WebSocket message");
                break;
            }
        }
    });

    // Spawn task to receive messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    break;
                }
                Message::Ping(_) => {
                    // Ping is handled automatically by axum
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    info!("WebSocket client disconnected");
}
