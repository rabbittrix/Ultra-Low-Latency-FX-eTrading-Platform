//! Risk Service
//!
//! Pre-trade risk checks, position tracking, and exposure calculation.

use axum::{routing::get, Router};
use fx_risk::{RiskEngine, RiskLimits};
use fx_utils::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Risk Service");

    let _engine = RiskEngine::new(RiskLimits::default());

    let app = Router::new().route("/health", get(|| async { "healthy" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084")
        .await
        .map_err(|e| fx_utils::Error::Io(e))?;

    info!("Risk Service listening on http://0.0.0.0:8084");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
