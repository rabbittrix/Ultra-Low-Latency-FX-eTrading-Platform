//! Pricing Service
//!
//! Generates BID/ASK spreads, applies risk adjustments, and integrates
//! with AI/ML modules for volatility prediction.

mod handlers;
mod websocket;

use axum::{
    extract::ws::WebSocketUpgrade,
    extract::State,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use fx_pricing::{AiClient, PricingEngine};
use fx_risk::{RiskEngine, RiskLimits};
use fx_utils::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Pricing Service");

    // Initialize AI client (optional)
    let ai_client = Arc::new(AiClient::new(
        std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8086".to_string()),
    ));

    // Initialize risk engine (optional)
    let risk_engine = Arc::new(RiskEngine::new(RiskLimits::default()));

    // Create pricing engine with AI and risk integration
    let engine = Arc::new(Mutex::new(
        PricingEngine::new()
            .with_ai_client(ai_client.clone())
            .with_risk_engine(risk_engine.clone()),
    ));

    let app_state = handlers::AppState::new(engine);

    let app = Router::new()
        .route("/health", get(health))
        .route("/prices", post(handlers::calculate_prices))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Pricing Service listening on http://0.0.0.0:8082");
    info!("REST API endpoints:");
    info!("  POST /prices - Calculate risk-adjusted prices");
    info!("  GET /ws - WebSocket stream for pricing updates");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "pricing-service" }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<handlers::AppState>,
) -> Response {
    ws.on_upgrade(move |socket| websocket::pricing_websocket_handler(socket, State(state)))
}
