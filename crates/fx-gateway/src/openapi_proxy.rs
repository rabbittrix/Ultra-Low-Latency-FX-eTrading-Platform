//! OpenAPI-only path stubs for reverse-proxied routes (no Axum handlers here).
#![allow(dead_code)]

use crate::proxy_types::{
    ExecutionSubmitRequest, ExecutionSubmitResponse, LiquidityPlanRequestBody,
};
use fx_liquidity_graph::{ExecutionPlan, LiquidityGraph};

/// `GET /liquidity/v1/graph/snapshot` — proxied to liquidity-graph-service.
#[utoipa::path(
    get,
    path = "/liquidity/v1/graph/snapshot",
    tag = "liquidity",
    responses(
        (status = 200, description = "Current liquidity graph", body = LiquidityGraph)
    )
)]
pub fn liquidity_graph_snapshot() {}

/// `POST /liquidity/v1/graph/recompute` — refresh mock graph; proxied to liquidity-graph-service.
#[utoipa::path(
    post,
    path = "/liquidity/v1/graph/recompute",
    tag = "liquidity",
    responses(
        (status = 200, description = "Graph after recompute", body = LiquidityGraph)
    )
)]
pub fn liquidity_graph_recompute() {}

/// `POST /liquidity/v1/plan` — compute execution plan; proxied to liquidity-graph-service.
#[utoipa::path(
    post,
    path = "/liquidity/v1/plan",
    tag = "liquidity",
    request_body = LiquidityPlanRequestBody,
    responses(
        (status = 200, description = "Plan JSON or null when no route", body = Option<ExecutionPlan>)
    )
)]
pub fn liquidity_plan() {}

/// `POST /execution/v1/execute` — risk stub → AI → plan → mock fills; proxied to execution-engine.
#[utoipa::path(
    post,
    path = "/execution/v1/execute",
    tag = "execution",
    request_body = ExecutionSubmitRequest,
    responses(
        (status = 200, description = "Execution result", body = ExecutionSubmitResponse)
    )
)]
pub fn execution_v1_execute() {}
