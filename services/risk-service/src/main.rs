//! Risk Service
//!
//! Pre-trade risk checks, position tracking, and exposure calculation.

mod handlers;

use axum::{
    routing::{get, post},
    Json, Router,
};
use fx_risk::{RiskEngine, RiskLimits};
use fx_utils::Result;
use handlers::AppState;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Risk Service");

    let engine = Arc::new(Mutex::new(RiskEngine::new(RiskLimits::default())));
    let app_state = AppState::new(engine);

    let app = Router::new()
        .route("/health", get(health))
        .route("/check", post(handlers::check_order))
        .route("/position/:instrument", get(handlers::get_position))
        .route("/exposure", get(handlers::get_exposure_summary))
        .route(
            "/exposure/:instrument",
            get(handlers::get_instrument_exposure),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Risk Service listening on http://0.0.0.0:8084");
    info!("REST API endpoints:");
    info!("  POST /check - Check order risk");
    info!("  GET /position/:instrument - Get position for instrument");
    info!("  GET /exposure - Get exposure summary");
    info!("  GET /exposure/:instrument - Get exposure for specific instrument");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "risk-service" }))
}
