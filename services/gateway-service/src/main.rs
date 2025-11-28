//! Gateway Service
//!
//! Combined API gateway for frontend, aggregating all microservices.
//! Provides REST endpoints, WebSocket streams, and Swagger/OpenAPI documentation.

mod websocket;

use axum::{
    extract::ws::WebSocketUpgrade, extract::State, response::Response, routing::get, Router,
};
use fx_gateway::{handlers, GatewayApi};
use fx_utils::Result;
use std::net::SocketAddr;
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
    let openapi_for_route = openapi.clone();

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health))
        .route("/ws", get(websocket_handler))
        .route(
            "/api-docs/openapi.json",
            get(move || async move { axum::Json(openapi_for_route) }),
        )
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .layer(CorsLayer::permissive())
        .with_state(ws_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Gateway Service listening on http://{}", addr);
    info!("Swagger UI available at http://{}/docs", addr);
    info!("WebSocket endpoint available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(fx_utils::Error::Io)?;

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
