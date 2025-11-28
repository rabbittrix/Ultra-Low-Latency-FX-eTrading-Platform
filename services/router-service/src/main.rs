//! Order Router Service
//!
//! Routes orders to external venues with minimal latency overhead.

mod metrics;

use axum::{routing::get, Router};
use fx_router::OrderRouter;
use fx_utils::Result;
use prometheus::Registry;
use std::sync::Arc;
use tracing::{info, Level};

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    router: Arc<OrderRouter>,
}

impl AppState {
    fn new(router: OrderRouter) -> Self {
        Self {
            router: Arc::new(router),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Router Service");

    let router = OrderRouter::new();
    let app_state = AppState::new(router);

    // Initialize Prometheus metrics
    let registry = Registry::new();
    let _metrics = Arc::new(
        metrics::Metrics::new(&registry).map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?,
    );

    let app = Router::new()
        .route("/health", get(|| async { "healthy" }))
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8085")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Router Service listening on http://0.0.0.0:8085");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
