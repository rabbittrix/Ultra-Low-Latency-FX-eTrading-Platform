//! Gateway Service
//!
//! Combined API gateway for frontend, aggregating all microservices.
//! Provides REST endpoints, WebSocket streams, and Swagger/OpenAPI documentation.

use axum::{routing::get, Router};
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

    let openapi = GatewayApi::openapi();

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Gateway Service listening on http://{}", addr);
    info!("Swagger UI available at http://{}/docs", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| fx_utils::Error::Io(e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
