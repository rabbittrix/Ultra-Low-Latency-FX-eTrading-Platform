//! WebSocket aggregation for frontend

use axum::extract::ws::{Message, WebSocket};
use axum::extract::State;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// WebSocket message types from backend services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GatewayMessage {
    #[serde(rename = "market_data")]
    MarketData {
        instrument: String,
        bid: f64,
        ask: f64,
        bid_size: u64,
        ask_size: u64,
        spread: u64,
        mid_price: f64,
        timestamp: u64,
    },
    #[serde(rename = "pricing")]
    Pricing {
        instrument: String,
        bid_price: u64,
        ask_price: u64,
        mid_price: u64,
        spread: u64,
        timestamp: u64,
    },
    #[serde(rename = "trade")]
    Trade {
        trade_id: String,
        instrument: String,
        quantity: u64,
        price: u64,
        side: String,
        timestamp: u64,
    },
    #[serde(rename = "exposure")]
    Exposure {
        instrument: String,
        position: i64,
        position_abs: u64,
        utilization: f64,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

/// Application state for WebSocket aggregation
#[derive(Clone)]
pub struct WebSocketState {
    /// Broadcast channel for aggregated messages
    pub message_tx: broadcast::Sender<GatewayMessage>,
    /// Active client connections
    pub clients: Arc<RwLock<HashMap<String, broadcast::Sender<GatewayMessage>>>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            message_tx: tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: GatewayMessage) {
        let _ = self.message_tx.send(message);
    }
}

/// WebSocket handler for frontend connections
pub async fn gateway_websocket_handler(ws: WebSocket, State(state): State<WebSocketState>) {
    let (mut sender, mut receiver) = ws.split();
    let client_id = uuid::Uuid::new_v4().to_string();

    // Subscribe to aggregated messages
    let mut rx = state.message_tx.subscribe();
    info!("New WebSocket client connected: {}", client_id);

    // Add client to registry
    {
        let (tx, _) = broadcast::channel(100);
        state.clients.write().await.insert(client_id.clone(), tx);
    }

    // Spawn task to send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    error!(error = %e, "Failed to serialize message");
                    continue;
                }
            };

            if let Err(e) = sender.send(Message::Text(json_msg)).await {
                error!(error = %e, "Failed to send WebSocket message");
                break;
            }
        }
    });

    // Spawn task to receive messages from client
    let client_id_for_recv = client_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Handle client messages (subscriptions, etc.)
                    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                        if let Some(msg_type) = json_value.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "subscribe" => {
                                    info!(
                                        "Client {} subscribed to: {:?}",
                                        client_id_for_recv, json_value
                                    );
                                }
                                "unsubscribe" => {
                                    info!(
                                        "Client {} unsubscribed from: {:?}",
                                        client_id_for_recv, json_value
                                    );
                                }
                                "ping" => {
                                    // Ping/pong is handled automatically by axum
                                    // Client can send ping, but we don't need to respond here
                                }
                                _ => {
                                    warn!("Unknown message type from client: {}", msg_type);
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("Client {} disconnected", client_id_for_recv);
                    break;
                }
                Message::Ping(_) => {
                    // Ping is handled automatically by axum
                }
                _ => {
                    // Ignore other message types
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

    // Remove client from registry
    state.clients.write().await.remove(&client_id);
    info!("WebSocket client {} disconnected", client_id);
}

/// Background task to aggregate messages from backend services
pub async fn aggregate_backend_streams(state: WebSocketState) {
    // In a production system, this would connect to:
    // - Market Data Service WebSocket (ws://market-data-service:8081/ws)
    // - Pricing Service WebSocket (ws://pricing-service:8082/ws)
    // - Matching Engine trade events
    // - Risk Service exposure updates

    // For now, we'll create a mock aggregator that simulates aggregated streams
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    loop {
        interval.tick().await;

        // In production, this would:
        // 1. Connect to backend WebSocket streams
        // 2. Parse incoming messages
        // 3. Transform and aggregate them
        // 4. Broadcast via state.message_tx

        // Mock: Simulate aggregated market data
        let mock_message = GatewayMessage::MarketData {
            instrument: "EURUSD".to_string(),
            bid: 1.0850,
            ask: 1.0852,
            bid_size: 1_000_000,
            ask_size: 1_000_000,
            spread: 2,
            mid_price: 1.0851,
            timestamp: fx_utils::time::now_nanos(),
        };

        state.broadcast(mock_message);
    }
}
