//! End-to-end deterministic execution pipeline: risk stub → graph → AI → parallel mock routing.

mod metrics;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use fx_ai_execution::{AiExecutionClient, InferRequest, VenueFeatures};
use fx_deterministic_core::OrderEventRing;
use fx_liquidity_graph::{plan_execution, GraphPlanner, LiquidityGraph};
use fx_utils::Result;
use futures_util::future::join_all;
use metrics::Metrics;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, Level};

#[derive(Clone)]
struct AppState {
    graph: Arc<RwLock<LiquidityGraph>>,
    planner: Arc<GraphPlanner>,
    ai: Arc<AiExecutionClient>,
    metrics: Arc<Metrics>,
    /// Hot-path handoff demo ring (preallocated, lock-free).
    ring: Arc<OrderEventRing>,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    instrument: String,
    side: String,
    quantity: f64,
    client_id: String,
}

#[derive(Serialize)]
struct FillLeg {
    venue_id: String,
    quantity: f64,
    latency_us: u64,
}

#[derive(Serialize)]
struct ExecuteResponse {
    /// Echo for correlation / auditing (ensures request fields are used end-to-end).
    client_id: String,
    risk_ok: bool,
    plan: fx_liquidity_graph::ExecutionPlan,
    fills: Vec<FillLeg>,
    total_latency_us: u64,
    ai_notes: String,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "execution-engine" }))
}

async fn execute(State(state): State<AppState>, Json(body): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    let ExecuteRequest {
        instrument,
        side,
        quantity,
        client_id,
    } = body;

    let t0 = Instant::now();
    tracing::debug!(
        client_id = %client_id,
        instrument = %instrument,
        side = %side,
        quantity,
        "execute request"
    );
    let _risk_timer = state.metrics.risk_check_time.start_timer();

    // Fast risk stub: non-empty client + notional cap (replace with risk-service gRPC in prod).
    let risk_ok = !client_id.is_empty() && quantity <= 50_000_000.0;
    state.metrics.risk_checks.inc();
    if risk_ok {
        state.metrics.risk_passed.inc();
    } else {
        state.metrics.risk_failed.inc();
    }
    drop(_risk_timer);

    let mut graph = state.graph.read().await.clone();

    // Build AI feature vector from CLIENT edges
    let features: Vec<VenueFeatures> = graph
        .edges_from("CLIENT")
        .iter()
        .map(|e| VenueFeatures {
            venue_id: e.to.clone(),
            spread_bps: ((e.price - 1.10).abs() * 10_000.0).min(50.0),
            depth: e.available_size,
            recent_reject_rate: (1.0 - e.fill_probability).clamp(0.0, 1.0),
            latency_ewma_us: e.latency_us,
            toxicity_hint: e.toxicity,
            mid_move_bps: 0.5,
        })
        .collect();

    let infer_req = InferRequest {
        instrument: instrument.clone(),
        side: side.clone(),
        quantity,
        venues: features,
    };

    let ai_notes = match state.ai.infer(&infer_req).await {
        Ok(resp) => {
            for v in &resp.venues {
                graph.apply_venue_fill_probs(&v.venue_id, v.fill_probability);
            }
            state.metrics.ai_calls.inc();
            resp.recommendation.notes
        }
        Err(e) => {
            warn!("AI infer failed (using graph defaults): {}", e);
            state.metrics.ai_failures.inc();
            "fallback: graph-only planning".into()
        }
    };

    let _g_timer = state.metrics.graph_plan_time.start_timer();
    let terminals = ["INTERNAL", "LP_A", "LP_B", "ECN_SIM"];
    let Some(plan) = plan_execution(
        &graph,
        &state.planner,
        &instrument,
        &side,
        quantity,
        &terminals,
    ) else {
        state.metrics.exec_failures.inc();
        let empty = fx_liquidity_graph::ExecutionPlan {
            instrument: instrument.clone(),
            side: side.clone(),
            total_quantity: quantity,
            allocations: vec![],
            slice_strategy: fx_liquidity_graph::SliceStrategy::Immediate,
            expected_slippage_bps: 0.0,
            primary_path: vec![],
            path_cost: 0.0,
        };
        return Json(ExecuteResponse {
            client_id,
            risk_ok,
            plan: empty,
            fills: vec![],
            total_latency_us: t0.elapsed().as_micros() as u64,
            ai_notes,
        });
    };
    drop(_g_timer);

    // Parallel mock venue dispatch
    let dispatch_futures: Vec<_> = plan
        .allocations
        .iter()
        .map(|a| {
            let vid = a.venue_id.clone();
            let q = a.quantity;
            async move {
                // Simulate venue RTT
                let us = 30 + (vid.len() as u64 * 7) % 120;
                tokio::time::sleep(std::time::Duration::from_micros(us)).await;
                FillLeg {
                    venue_id: vid,
                    quantity: q,
                    latency_us: us,
                }
            }
        })
        .collect();

    let _d_timer = state.metrics.dispatch_parallel_time.start_timer();
    let fills = join_all(dispatch_futures).await;
    drop(_d_timer);

    // Push synthetic events into lock-free ring (demo hot path)
    for (i, f) in fills.iter().enumerate() {
        let _ = state.ring.try_push(fx_deterministic_core::OrderEventSlot {
            order_id: i as u64,
            qty: f.quantity as u64,
            price_ticks: 0,
            flags: 1,
        });
    }

    let total_us = t0.elapsed().as_micros() as u64;
    state.metrics.exec_latency.observe(total_us as f64 / 1_000_000.0);
    state.metrics.exec_success.inc();

    Json(ExecuteResponse {
        client_id,
        risk_ok,
        plan,
        fills,
        total_latency_us: total_us,
        ai_notes,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("Starting Execution Engine (deterministic core integration)");

    let ai_url =
        std::env::var("AI_EXECUTION_URL").unwrap_or_else(|_| "http://127.0.0.1:8093".into());
    let ai = Arc::new(
        AiExecutionClient::new(&ai_url).map_err(|e| fx_utils::Error::Internal(e.to_string()))?,
    );

    let metrics = Arc::new(Metrics::new().map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?);

    let instrument = std::env::var("LIQUIDITY_INSTRUMENT").unwrap_or_else(|_| "EURUSD".into());
    let graph = Arc::new(RwLock::new(LiquidityGraph::mock_global_liquidity(&instrument)));
    let planner = Arc::new(GraphPlanner::default());
    let ring = Arc::new(OrderEventRing::new(65_536));

    let state = AppState {
        graph,
        planner,
        ai,
        metrics,
        ring,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/v1/execute", post(execute))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8092";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(fx_utils::Error::Io)?;
    info!("Execution Engine listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod execute_request_tests {
    use super::ExecuteRequest;

    #[test]
    fn request_fields_are_used_via_destructure() {
        let body = ExecuteRequest {
            instrument: "EURUSD".into(),
            side: "buy".into(),
            quantity: 1_000_000.0,
            client_id: "test".into(),
        };
        let ExecuteRequest {
            instrument,
            side,
            quantity,
            client_id,
        } = body;
        assert_eq!(instrument, "EURUSD");
        assert_eq!(side, "buy");
        assert!(quantity > 0.0);
        assert!(!client_id.is_empty());
    }
}
