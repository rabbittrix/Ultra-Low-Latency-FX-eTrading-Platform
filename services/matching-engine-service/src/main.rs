//! Matching Engine Service
//!
//! Ultra-low-latency matching algorithm with lock-free structures.
//! Supports Market, Limit, Stop, IOC, and FOK order types.

use axum::{routing::get, Router};
use fx_core::MatchingEngine;
use fx_utils::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Matching Engine Service");

    #[allow(dead_code)]
    let _engine = MatchingEngine::new("EURUSD".to_string());

    let app = Router::new().route("/health", get(|| async { "healthy" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Matching Engine Service listening on http://0.0.0.0:8083");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
