//! HTTP handlers for the gateway

use crate::api::{GatewayInfo, HealthResponse};
use axum::response::Json;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "gateway",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "gateway".to_string(),
    })
}

/// Root endpoint
#[utoipa::path(
    get,
    path = "/",
    tag = "gateway",
    responses(
        (status = 200, description = "Gateway information", body = GatewayInfo)
    )
)]
pub async fn root() -> Json<GatewayInfo> {
    Json(GatewayInfo {
        name: "FX eTrading Gateway".to_string(),
        version: "0.1.0".to_string(),
    })
}
