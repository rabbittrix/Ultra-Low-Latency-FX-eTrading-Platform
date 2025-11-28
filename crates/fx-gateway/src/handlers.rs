//! HTTP handlers for the gateway

use axum::response::Json;
use serde_json::{json, Value};

/// Health check endpoint
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "gateway"
    }))
}

/// Root endpoint
pub async fn root() -> Json<Value> {
    Json(json!({
        "name": "FX eTrading Gateway",
        "version": "0.1.0"
    }))
}
