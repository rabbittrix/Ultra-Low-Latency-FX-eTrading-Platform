//! SMC advisory API (M9): Axum + `OpenAPI` + optional Telegram (ADR-0010).
//!
//! Informational research surface only — not investment advice; no returns promised.

mod dto;
mod telegram;

use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use fx_smc_advisory::{
    best_entry_window, classify_regime, conf_from_structure_breaks, score_window, suitability,
};
use fx_smc_common::AppConfig;
use fx_smc_liquidity::map_from_ticks;
use fx_smc_marketdata::{generate_ticks, SynthParams};
use fx_smc_risk::KillSwitch;
use fx_smc_strategy::build_plans;
use fx_smc_structure::{detect_fvgs, detect_structure_breaks, detect_swings};
use fx_smc_sweep::detect_sweeps;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::dto::{
    AnalyzeRequest, AnalyzeResponse, DisclaimerResponse, HealthResponse, PlanDto, RegimeDto,
    SuitabilityDto, SweepDto, WindowScoreDto,
};
use crate::telegram::maybe_send_telegram;

/// Hard cap so debug builds cannot wedge the async runtime for minutes.
const MAX_ANALYZE_TICKS: usize = 2_500;
const DEFAULT_ANALYZE_TICKS: usize = 400;

#[derive(Clone)]
struct AppState {
    cfg: Arc<AppConfig>,
}

#[derive(OpenApi)]
#[openapi(
    paths(health, disclaimer, analyze),
    components(schemas(
        HealthResponse,
        DisclaimerResponse,
        AnalyzeRequest,
        AnalyzeResponse,
        PlanDto,
        SweepDto,
        RegimeDto,
        WindowScoreDto,
        SuitabilityDto
    )),
    tags((name = "smc-advisory", description = "SMC research advisory API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("config/default.toml"), PathBuf::from);
    let cfg = AppConfig::load_path(&config_path)
        .with_context(|| format!("load config {}", config_path.display()))?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cfg.tracing.default_filter.clone()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!(disclaimer = %cfg.disclaimer.text, "advisory API starting — informational only");

    let port = env::var("SMC_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(cfg.api.http_port);
    let state = AppState { cfg: Arc::new(cfg) };

    let app = Router::new()
        .route("/health", get(health))
        .route("/disclaimer", get(disclaimer))
        .route("/v1/analyze", post(analyze))
        .route("/ws", get(ws_handler))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "OK", body = HealthResponse)),
    tag = "smc-advisory"
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".into(),
        service: "fx-smc-advisory-api".into(),
    })
}

#[utoipa::path(
    get,
    path = "/disclaimer",
    responses((status = 200, description = "Risk disclaimer", body = DisclaimerResponse)),
    tag = "smc-advisory"
)]
async fn disclaimer(State(state): State<AppState>) -> Json<DisclaimerResponse> {
    Json(DisclaimerResponse {
        text: state.cfg.disclaimer.text.clone(),
    })
}

#[utoipa::path(
    post,
    path = "/v1/analyze",
    request_body = AnalyzeRequest,
    responses((status = 200, description = "Synth analysis", body = AnalyzeResponse)),
    tag = "smc-advisory"
)]
async fn analyze(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    let cfg = Arc::clone(&state.cfg);
    let tick_count = req
        .tick_count
        .unwrap_or(DEFAULT_ANALYZE_TICKS)
        .clamp(32, MAX_ANALYZE_TICKS);

    // CPU-heavy pipeline must not block the Tokio reactor (otherwise /health hangs too).
    let resp = tokio::task::spawn_blocking(move || run_analyze(&cfg, tick_count))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("analyze task join: {e}"),
            )
        })?;

    if let Err(e) = maybe_send_telegram(state.cfg.as_ref(), &resp).await {
        warn!(error = %e, "telegram alert skipped or failed");
    }

    Ok(Json(resp))
}

fn run_analyze(cfg: &AppConfig, tick_count: usize) -> AnalyzeResponse {
    const MAX_SWEEPS: usize = 32;
    const MAX_PLANS: usize = 32;

    let meta = cfg.instrument.default.to_meta();
    let mut params = SynthParams::from_config(&cfg.synth, &meta);
    params.tick_count = tick_count;
    let ticks = generate_ticks(&params);
    let pools = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
    let sweeps = detect_sweeps(&ticks, &pools, &cfg.sweep);
    let plans = build_plans(&ticks, &sweeps, &pools, &cfg.strategy, &cfg.disclaimer.text);
    let window = score_window(&ticks, &cfg.advisory);
    let regime = classify_regime(&ticks, &cfg.advisory);
    let kill = KillSwitch::default();
    let top_conf = plans.first().map_or(0, |p| p.confluence);
    let suit = suitability(top_conf, &cfg.advisory, Some(&kill), &cfg.disclaimer.text);

    let swings = detect_swings(&ticks, &cfg.structure.swings);
    let breaks = detect_structure_breaks(&swings);
    let fvgs = detect_fvgs(&ticks, &cfg.structure.fvg);
    let conf = conf_from_structure_breaks(&breaks);

    let now_ns = ticks.last().map_or(fx_smc_common::TsNanos(0), |t| t.ts_ns);
    let hour_utc = hour_utc_from_ns(now_ns.0);
    let rr_est_milli = plans.first().map_or(0, |p| {
        if p.risk_ticks <= 0 {
            0
        } else {
            p.reward_ticks.saturating_mul(1_000) / p.risk_ticks.max(1)
        }
    });
    let entry = best_entry_window(
        &sweeps,
        &pools,
        rr_est_milli,
        conf,
        regime,
        now_ns,
        hour_utc,
        false,
        matches!(regime, fx_smc_advisory::Regime::Volatile),
        &cfg.window_score,
    );

    AnalyzeResponse {
        disclaimer: cfg.disclaimer.text.clone(),
        tick_count: ticks.len(),
        pool_count: pools.len(),
        sweep_total: sweeps.len(),
        plan_total: plans.len(),
        structure_break_count: breaks.len(),
        fvg_count: fvgs.len(),
        conf_signal: format!("{conf:?}"),
        sweeps: sweeps
            .iter()
            .take(MAX_SWEEPS)
            .map(|s| SweepDto {
                pool_id: s.pool_id.0.clone(),
                side: format!("{:?}", s.side),
                pierce_idx: s.pierce_idx,
                confirm_idx: s.confirm_idx,
                displacement_ticks: s.displacement_ticks,
            })
            .collect(),
        plans: plans
            .iter()
            .take(MAX_PLANS)
            .map(|p| PlanDto {
                id: p.id.clone(),
                side: format!("{:?}", p.side),
                entry_ticks: p.entry.0,
                stop_ticks: p.stop.0,
                target_ticks: p.target.0,
                risk_ticks: p.risk_ticks,
                reward_ticks: p.reward_ticks,
                confluence: p.confluence,
                invalidation: p.invalidation.clone(),
            })
            .collect(),
        regime: RegimeDto {
            label: format!("{regime:?}"),
        },
        window: WindowScoreDto {
            score: window.score,
            window_ticks: window.window_ticks,
        },
        suitability: SuitabilityDto {
            suitable: suit.suitable,
            reasons: suit.reasons,
        },
        window_color: format!("{:?}", entry.color),
        window_raw: entry.raw,
        window_side: format!("{:?}", entry.side),
        facts: entry.facts.iter().map(ToString::to_string).collect(),
    }
}

fn hour_utc_from_ns(ts_ns: i64) -> u8 {
    let secs = ts_ns.div_euclid(1_000_000_000).rem_euclid(86_400);
    u8::try_from(secs.div_euclid(3_600)).unwrap_or(0)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_session(socket, state))
}

async fn ws_session(mut socket: WebSocket, state: AppState) {
    let hello = serde_json::json!({
        "type": "hello",
        "disclaimer": state.cfg.disclaimer.text,
        "note": "Informational stream only — not investment advice."
    });
    if socket.send(Message::Text(hello.to_string())).await.is_err() {
        return;
    }
    while let Some(Ok(msg)) = socket.recv().await {
        if matches!(msg, Message::Close(_)) {
            break;
        }
        let pong = serde_json::json!({
            "type": "pong",
            "disclaimer": state.cfg.disclaimer.text
        });
        if socket.send(Message::Text(pong.to_string())).await.is_err() {
            break;
        }
    }
}
