//! HTTP handlers for risk service

use axum::extract::State;
use axum::response::Json;
use fx_risk::{ExposureCalculator, RiskEngine};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state for risk service
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<RiskEngine>>,
}

impl AppState {
    pub fn new(engine: Arc<Mutex<RiskEngine>>) -> Self {
        Self { engine }
    }
}

#[derive(serde::Deserialize)]
pub struct CheckOrderRequest {
    pub instrument: String,
    pub side: String, // "Buy" or "Sell"
    pub quantity: u64,
    pub order_id: String,
}

#[derive(serde::Serialize)]
pub struct CheckOrderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct PositionResponse {
    pub instrument: String,
    pub position: i64,
}

pub async fn check_order(
    State(state): State<AppState>,
    Json(req): Json<CheckOrderRequest>,
) -> Json<CheckOrderResponse> {
    use fx_utils::{Quantity, Side};
    use uuid::Uuid;

    let order_id = Uuid::parse_str(&req.order_id).unwrap_or_else(|_| Uuid::new_v4());
    let side = match req.side.as_str() {
        "Buy" => Side::Buy,
        "Sell" => Side::Sell,
        _ => {
            return Json(CheckOrderResponse {
                success: false,
                message: format!("Invalid side: {}", req.side),
            });
        }
    };

    let engine_guard = state.engine.lock().await;
    match engine_guard.check_order(&req.instrument, side, Quantity(req.quantity), order_id) {
        Ok(_) => Json(CheckOrderResponse {
            success: true,
            message: "Order passed risk checks".to_string(),
        }),
        Err(e) => Json(CheckOrderResponse {
            success: false,
            message: format!("Risk check failed: {}", e),
        }),
    }
}

pub async fn get_position(
    State(state): State<AppState>,
    axum::extract::Path(instrument): axum::extract::Path<String>,
) -> Json<PositionResponse> {
    let engine_guard = state.engine.lock().await;
    let position = engine_guard.get_position(&instrument);

    Json(PositionResponse {
        instrument,
        position,
    })
}

pub async fn get_exposure_summary(State(state): State<AppState>) -> Json<fx_risk::ExposureSummary> {
    let engine_guard = state.engine.lock().await;
    let summary = ExposureCalculator::calculate_exposure(
        engine_guard.positions(),
        engine_guard.open_orders(),
        engine_guard.limits(),
    );

    Json(summary)
}

pub async fn get_instrument_exposure(
    State(state): State<AppState>,
    axum::extract::Path(instrument): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let engine_guard = state.engine.lock().await;
    match ExposureCalculator::calculate_instrument_exposure(
        &instrument,
        engine_guard.positions(),
        engine_guard.open_orders(),
        engine_guard.limits(),
    ) {
        Some(exposure) => Json(json!(exposure)),
        None => Json(json!({
            "instrument": instrument,
            "position": 0,
            "position_abs": 0,
            "position_utilization": 0.0,
            "open_orders_count": 0
        })),
    }
}
