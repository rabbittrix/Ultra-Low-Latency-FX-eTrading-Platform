//! Pricing Service
//!
//! Generates BID/ASK spreads, applies risk adjustments, and integrates
//! with AI/ML modules for volatility prediction.

use axum::{routing::get, Router};
use fx_pricing::PricingEngine;
use fx_utils::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Pricing Service");

    let _engine = PricingEngine::new();

    let app = Router::new().route("/health", get(|| async { "healthy" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082")
        .await
        .map_err(|e| fx_utils::Error::Io(e))?;

    info!("Pricing Service listening on http://0.0.0.0:8082");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
