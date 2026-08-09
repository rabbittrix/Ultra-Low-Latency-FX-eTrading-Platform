//! Typed `OpenAPI` DTOs for the advisory API.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Liveness payload.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Status string.
    pub status: String,
    /// Service name.
    pub service: String,
}

/// Disclaimer payload.
#[derive(Debug, Serialize, ToSchema)]
pub struct DisclaimerResponse {
    /// Full disclaimer text.
    pub text: String,
}

/// Analyze request (optional overrides).
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalyzeRequest {
    /// Override synth tick count.
    pub tick_count: Option<usize>,
}

/// Sweep summary DTO.
#[derive(Debug, Serialize, ToSchema)]
pub struct SweepDto {
    /// Pool id.
    pub pool_id: String,
    /// Side label.
    pub side: String,
    /// Pierce index.
    pub pierce_idx: usize,
    /// Confirm index.
    pub confirm_idx: usize,
    /// Displacement ticks.
    pub displacement_ticks: i64,
}

/// Plan summary DTO.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlanDto {
    /// Plan id.
    pub id: String,
    /// Side label.
    pub side: String,
    /// Entry ticks.
    pub entry_ticks: i64,
    /// Stop ticks.
    pub stop_ticks: i64,
    /// Target ticks.
    pub target_ticks: i64,
    /// Risk ticks.
    pub risk_ticks: i64,
    /// Reward ticks.
    pub reward_ticks: i64,
    /// Confluence.
    pub confluence: i64,
    /// Invalidation text.
    pub invalidation: String,
}

/// Regime DTO.
#[derive(Debug, Serialize, ToSchema)]
pub struct RegimeDto {
    /// Regime label.
    pub label: String,
}

/// Window score DTO.
#[derive(Debug, Serialize, ToSchema)]
pub struct WindowScoreDto {
    /// Score.
    pub score: i64,
    /// Window length.
    pub window_ticks: usize,
}

/// Suitability DTO.
#[derive(Debug, Serialize, ToSchema)]
pub struct SuitabilityDto {
    /// Whether suitable under current gates.
    pub suitable: bool,
    /// Reasons (includes disclaimer).
    pub reasons: Vec<String>,
}

/// Full analyze response.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalyzeResponse {
    /// Always-present disclaimer.
    pub disclaimer: String,
    /// Tick count analyzed.
    pub tick_count: usize,
    /// Pool count (after mapping cap).
    pub pool_count: usize,
    /// Total sweeps before response truncation.
    pub sweep_total: usize,
    /// Total plans before response truncation.
    pub plan_total: usize,
    /// BOS/CHoCH events on the analyzed prefix.
    pub structure_break_count: usize,
    /// Fair value gaps detected.
    pub fvg_count: usize,
    /// Mapped confluence signal (`ChoCh` / `Bos` / …).
    pub conf_signal: String,
    /// Sweeps (truncated for UI).
    pub sweeps: Vec<SweepDto>,
    /// Plans (truncated for UI).
    pub plans: Vec<PlanDto>,
    /// Regime.
    pub regime: RegimeDto,
    /// Window score.
    pub window: WindowScoreDto,
    /// Suitability.
    pub suitability: SuitabilityDto,
    /// Entry-window traffic light (ADR-0012).
    pub window_color: String,
    /// Entry-window raw score (`0..=score_scale`).
    pub window_raw: i64,
    /// Best entry side label.
    pub window_side: String,
    /// Explainable facts (Display strings).
    pub facts: Vec<String>,
}
