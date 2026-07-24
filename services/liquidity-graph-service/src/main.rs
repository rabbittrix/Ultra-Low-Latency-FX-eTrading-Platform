//! HTTP API for the global liquidity graph: snapshot, recompute, plan.

mod metrics;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use fx_liquidity_graph::{plan_execution, ExecutionPlan, GraphPlanner, LiquidityGraph};
use fx_utils::Result;
use metrics::Metrics;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};

#[derive(Clone)]
struct AppState {
    graph: Arc<RwLock<LiquidityGraph>>,
    planner: Arc<GraphPlanner>,
    metrics: Arc<Metrics>,
}

#[derive(Deserialize)]
struct PlanRequest {
    instrument: String,
    side: String,
    quantity: f64,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "liquidity-graph-service" }))
}

async fn snapshot(State(state): State<AppState>) -> Json<LiquidityGraph> {
    let g = state.graph.read().await.clone();
    Json(g)
}

async fn recompute(State(state): State<AppState>) -> Json<LiquidityGraph> {
    let _t = state.metrics.graph_recompute_time.start_timer();
    let mut g = state.graph.write().await;
    *g = LiquidityGraph::mock_global_liquidity(&g.instrument);
    state.metrics.graph_recomputes.inc();
    Json(g.clone())
}

async fn plan(
    State(state): State<AppState>,
    Json(body): Json<PlanRequest>,
) -> Json<Option<ExecutionPlan>> {
    let g = state.graph.read().await;
    let terminals = ["INTERNAL", "LP_A", "LP_B", "ECN_SIM"];
    let plan = plan_execution(
        &g,
        &state.planner,
        &body.instrument,
        &body.side,
        body.quantity,
        &terminals,
    );
    Json(plan)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("Starting Liquidity Graph Service");

    let metrics = Arc::new(Metrics::new().map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?);

    let instrument = std::env::var("LIQUIDITY_INSTRUMENT").unwrap_or_else(|_| "EURUSD".into());
    let graph = Arc::new(RwLock::new(LiquidityGraph::mock_global_liquidity(
        &instrument,
    )));
    let planner = Arc::new(GraphPlanner::default());

    let state = AppState {
        graph,
        planner,
        metrics,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/v1/graph/snapshot", get(snapshot))
        .route("/v1/graph/recompute", post(recompute))
        .route("/v1/plan", post(plan))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8091";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(fx_utils::Error::Io)?;
    info!("Liquidity Graph Service listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;
    Ok(())
}
