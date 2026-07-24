//! Gateway Service
//!
//! Combined API gateway for frontend, aggregating all microservices.
//! Provides REST endpoints, WebSocket streams, and Swagger/OpenAPI documentation.

mod metrics;
mod proxy;
mod websocket;

use axum::{
    extract::ws::WebSocketUpgrade, extract::State, http::StatusCode, response::Response,
    routing::get, Router,
};
use fx_gateway::{handlers, GatewayApi};
use fx_utils::Result;
use prometheus::Registry;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Gateway Service");

    // Initialize WebSocket state
    let ws_state = websocket::WebSocketState::new();

    // Start background task to aggregate backend streams
    let ws_state_for_aggregator = ws_state.clone();
    tokio::spawn(async move {
        websocket::aggregate_backend_streams(ws_state_for_aggregator).await;
    });

    let openapi = GatewayApi::openapi();

    // Initialize Prometheus metrics
    let registry = Registry::new();
    let _metrics = Arc::new(
        metrics::Metrics::new(&registry).map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?,
    );

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/ws", get(websocket_handler))
        // Nest + `/*path` at inner root: `/prefix/*` is reliable with matchit (flat `/prefix/*path` can 404).
        .nest(
            "/matching",
            Router::new().route(
                "/*path",
                get(proxy::proxy_matching)
                    .post(proxy::proxy_matching)
                    .put(proxy::proxy_matching)
                    .delete(proxy::proxy_matching)
                    .options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .nest(
            "/risk",
            Router::new().route(
                "/*path",
                get(proxy::proxy_risk)
                    .post(proxy::proxy_risk)
                    .put(proxy::proxy_risk)
                    .delete(proxy::proxy_risk)
                    .options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .nest(
            "/market-data",
            Router::new().route(
                "/*path",
                get(proxy::proxy_market_data).options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .nest(
            "/pricing",
            Router::new().route(
                "/*path",
                get(proxy::proxy_pricing)
                    .post(proxy::proxy_pricing)
                    .put(proxy::proxy_pricing)
                    .options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .nest(
            "/liquidity",
            Router::new().route(
                "/*path",
                get(proxy::proxy_liquidity)
                    .post(proxy::proxy_liquidity)
                    .put(proxy::proxy_liquidity)
                    .delete(proxy::proxy_liquidity)
                    .options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .nest(
            "/execution",
            Router::new().route(
                "/*path",
                get(proxy::proxy_execution)
                    .post(proxy::proxy_execution)
                    .put(proxy::proxy_execution)
                    .delete(proxy::proxy_execution)
                    .options(|| async { StatusCode::NO_CONTENT }),
            ),
        )
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .layer(CorsLayer::permissive())
        .with_state(ws_state);

    let port = std::env::var("GATEWAY_HTTP_PORT")
        .ok()
        .and_then(|s| u16::from_str(&s).ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(fx_utils::Error::Io)?;

    info!(
        "Gateway Service listening on http://{} (set GATEWAY_HTTP_PORT to override)",
        addr
    );
    info!("Swagger UI available at http://{}/docs", addr);
    info!("WebSocket endpoint available at ws://{}/ws", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<websocket::WebSocketState>,
) -> Response {
    ws.on_upgrade(move |socket| websocket::gateway_websocket_handler(socket, State(state)))
}
