//! JSON shapes for proxied liquidity / execution routes (OpenAPI + gateway docs).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LiquidityPlanRequestBody {
    pub instrument: String,
    pub side: String,
    pub quantity: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExecutionSubmitRequest {
    pub instrument: String,
    pub side: String,
    pub quantity: f64,
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExecutionFillLeg {
    pub venue_id: String,
    pub quantity: f64,
    pub latency_us: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExecutionSubmitResponse {
    pub client_id: String,
    pub risk_ok: bool,
    pub plan: fx_liquidity_graph::ExecutionPlan,
    pub fills: Vec<ExecutionFillLeg>,
    pub total_latency_us: u64,
    pub ai_notes: String,
}
