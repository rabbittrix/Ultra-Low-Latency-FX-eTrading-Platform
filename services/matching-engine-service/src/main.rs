//! Matching Engine Service
//!
//! Ultra-low-latency matching algorithm with lock-free structures.
//! Supports Market, Limit, Stop, IOC, and FOK order types.

mod grpc;
mod handlers;
mod metrics;

use axum::{
    routing::{get, post},
    Json, Router,
};
use fx_core::MatchingEngine;
use fx_proto::fx::etrading::matching_engine_service_server::MatchingEngineServiceServer;
use fx_utils::Result;
use parking_lot::Mutex;
use prometheus::Registry;
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Matching Engine Service");

    let engine = Arc::new(Mutex::new(MatchingEngine::new("EURUSD".to_string())));
    let engine_for_grpc = engine.clone();

    // Start gRPC server
    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_service =
        MatchingEngineServiceServer::new(grpc::MatchingEngineGrpcService::new(engine_for_grpc));
    tokio::spawn(async move {
        Server::builder()
            .add_service(grpc_service)
            .serve(grpc_addr)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "gRPC server error");
            });
    });

    // Initialize Prometheus metrics
    let registry = Registry::new();
    let _metrics = Arc::new(
        metrics::Metrics::new(&registry).map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?,
    );

    // Start REST API server
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/orders", post(handlers::submit_order))
        .route("/orders/cancel", post(handlers::cancel_order))
        .route("/trades", get(handlers::get_trades))
        .route("/audit", get(handlers::get_audit_events))
        .with_state(engine);

    let listener = TcpListener::bind("0.0.0.0:8083")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Matching Engine Service listening on http://0.0.0.0:8083");
    info!("gRPC server listening on http://0.0.0.0:50051");
    info!("REST API endpoints:");
    info!("  POST /orders - Submit order");
    info!("  POST /orders/cancel - Cancel order");
    info!("  GET /trades - Get all trades");
    info!("  GET /audit - Get audit events");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "matching-engine-service" }))
}
