//! Order Router Service
//!
//! Routes orders to external venues with minimal latency overhead.

use axum::{routing::get, Router};
use fx_router::OrderRouter;
use fx_utils::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Router Service");

    let _router = OrderRouter::new();

    let app = Router::new().route("/health", get(|| async { "healthy" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8085")
        .await
        .map_err(|e| fx_utils::Error::Io(e))?;

    info!("Router Service listening on http://0.0.0.0:8085");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
