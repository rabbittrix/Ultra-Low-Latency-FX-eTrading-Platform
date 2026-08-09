//! TOML configuration loading (no secrets in files).

use crate::error::SmcError;
use crate::types::{InstrumentMeta, SymbolId};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Top-level application config.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Default instrument section.
    pub instrument: InstrumentSection,
    /// Logical clock defaults.
    pub clock: ClockConfig,
    /// Synthetic generator defaults.
    pub synth: SynthConfig,
    /// Persistence paths.
    pub store: StoreConfig,
    /// Tracing filter defaults.
    pub tracing: TracingConfig,
    /// User-facing disclaimer (advisory surfaces).
    pub disclaimer: DisclaimerConfig,
    /// Market structure parameters (M1+).
    pub structure: StructureConfig,
    /// Liquidity pool mapping (M2+); scoring lives in `liquidity_score`.
    pub liquidity: LiquidityConfig,
    /// Fixed-point liquidity pool scoring (ADR-0012).
    pub liquidity_score: LiquidityScoreConfig,
    /// Entry-window traffic-light scoring (ADR-0012).
    pub window_score: WindowScoreConfig,
    /// Sweep detector parameters (M3+).
    pub sweep: SweepConfig,
    /// Trade plan / confluence (M4+).
    pub strategy: StrategyConfig,
    /// Backtest / walk-forward / costs (M5+).
    pub backtest: BacktestConfig,
    /// Risk sizing and kill-switch (M6+).
    pub risk: RiskConfig,
    /// Advisory regime / window scoring (M7+).
    pub advisory: AdvisoryConfig,
    /// Journal / paper trading (M8+).
    pub journal: JournalConfig,
    /// Advisory HTTP API defaults (M9+).
    pub api: ApiConfig,
}

/// `[instrument.default]` wrapper for nested TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentSection {
    /// Default traded instrument.
    pub default: InstrumentConfig,
}

/// Instrument parameters (tick size from config).
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentConfig {
    /// Symbol text.
    pub symbol: String,
    /// Human-scale documentation divisor for prices.
    pub price_scale: i64,
    /// Minimum increment in ticks.
    pub tick_size: i64,
    /// Quantity scale.
    pub qty_scale: i64,
}

impl InstrumentConfig {
    /// Convert to runtime metadata.
    #[must_use]
    pub fn to_meta(&self) -> InstrumentMeta {
        InstrumentMeta {
            symbol: SymbolId::new(self.symbol.clone()),
            price_scale: self.price_scale,
            tick_size: self.tick_size.max(1),
            qty_scale: self.qty_scale.max(1),
        }
    }
}

/// Clock section.
#[derive(Debug, Clone, Deserialize)]
pub struct ClockConfig {
    /// Starting epoch in nanos.
    pub epoch_ns: i64,
}

/// Synthetic marketdata section.
#[derive(Debug, Clone, Deserialize)]
pub struct SynthConfig {
    /// PRNG seed (deterministic).
    pub seed: u64,
    /// Number of ticks to emit.
    pub tick_count: usize,
    /// Starting mid in ticks.
    pub base_mid_ticks: i64,
    /// Max random walk step in ticks.
    pub walk_ticks: i64,
    /// Insert a sweep every N ticks (`0` disables).
    pub sweep_every: usize,
    /// Sweep break size in ticks.
    pub sweep_break_ticks: i64,
}

/// Store section.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreConfig {
    /// Directory for Parquet datasets.
    pub parquet_dir: String,
    /// Optional Postgres URL override (prefer `SMC_DATABASE_URL` env in production).
    #[serde(default)]
    pub postgres_url: Option<String>,
}

/// Tracing section.
#[derive(Debug, Clone, Deserialize)]
pub struct TracingConfig {
    /// `RUST_LOG`-style filter.
    pub default_filter: String,
}

/// Disclaimer section — never promise returns.
#[derive(Debug, Clone, Deserialize)]
pub struct DisclaimerConfig {
    /// Full disclaimer text for UI/API.
    pub text: String,
}

/// `[structure]` aggregate.
#[derive(Debug, Clone, Deserialize)]
pub struct StructureConfig {
    /// Swing pivot detection.
    pub swings: SwingConfig,
    /// Equal highs / lows clustering.
    pub equal: EqualConfig,
    /// Trendline liquidity anchors.
    pub trendline: TrendlineConfig,
    /// Session level tracking.
    pub sessions: SessionConfig,
    /// Fair value gap detection.
    pub fvg: FvgConfig,
}

/// Fair value gap parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct FvgConfig {
    /// Minimum gap size in ticks.
    pub min_gap_ticks: i64,
}

/// Swing pivot parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct SwingConfig {
    /// Bars to the left that must be dominated.
    pub left_strength: usize,
    /// Bars to the right that must be dominated.
    pub right_strength: usize,
}

/// Equal high/low parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct EqualConfig {
    /// Minimum tolerance in ticks.
    pub pips_min_ticks: i64,
    /// ATR multiplier numerator.
    pub k_atr_num: i64,
    /// ATR multiplier denominator (`>= 1`).
    pub k_atr_den: i64,
    /// Lookback length for ATR proxy.
    pub atr_lookback: usize,
}

/// Trendline parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct TrendlineConfig {
    /// Minimum anchors / touches required.
    pub min_touches: usize,
    /// Max vertical distance (ticks) to count a touch.
    pub touch_tolerance_ticks: i64,
}

/// Session window parameters (UTC hours).
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    /// Asia session start hour `[0, 23]` UTC inclusive start.
    pub asia_start_hour_utc: u8,
    /// Asia session end hour UTC (exclusive); may be `< start` for wrap.
    pub asia_end_hour_utc: u8,
    /// Track previous-day high / low.
    pub track_pdh_pdl: bool,
    /// Track week high / low (UTC week starting Monday).
    pub track_wh_wl: bool,
}

/// `[liquidity]` mapping-only parameters (ADR-0012).
#[derive(Debug, Clone, Deserialize)]
pub struct LiquidityConfig {
    /// Include equal high/low clusters with at least this many members.
    pub min_equal_members: usize,
    /// Include trendlines with at least this many touches.
    pub min_trendline_touches: usize,
    /// After scoring, keep at most this many pools (`0` = unlimited).
    pub max_mapped_pools: usize,
}

/// `[liquidity_score]` fixed-point pool scoring (ADR-0012).
#[derive(Debug, Clone, Deserialize)]
pub struct LiquidityScoreConfig {
    /// Output score scale (max score); `10_000` ≡ `1.0`.
    pub score_scale: i64,
    /// Cap for touch contribution.
    pub touches_cap: u32,
    /// Recency half-life in nanoseconds (default 24h).
    pub half_life_ns: i64,
    /// Distance decay λ in milli-ATR (default `2000` = 2.0 ATR).
    pub lambda_atr_milli: i64,
    /// Equality std tolerance in ticks for `s_eq`.
    pub equality_tol_ticks: i64,
    /// Weight: touches (must sum with other weights to `score_scale`).
    pub w_touches: i64,
    /// Weight: equality tightness.
    pub w_equality: i64,
    /// Weight: recency.
    pub w_recency: i64,
    /// Weight: distance to mid / ATR.
    pub w_distance: i64,
    /// Weight: origin / round-number context.
    pub w_context: i64,
    /// Context score for Asia session extremes.
    pub ctx_session: i64,
    /// Context score for PDH/PDL/WH/WL.
    pub ctx_pdh_pdl_wh_wl: i64,
    /// Context score bump for round-number prices.
    pub ctx_round: i64,
    /// Context score when origin has no session premium (equal / trendline).
    pub ctx_none: i64,
    /// Round-number grid in ticks (`price % round_number_ticks == 0`).
    pub round_number_ticks: i64,
}

/// `[window_score]` entry-window traffic light (ADR-0012).
#[derive(Debug, Clone, Deserialize)]
pub struct WindowScoreConfig {
    /// Output score scale (`10_000` ≡ `1.0`).
    pub score_scale: i64,
    /// Half-life for sweep contribution decay (nanoseconds).
    pub sweep_half_life_ns: i64,
    /// Ignore sweeps older than this for `s_sweep` / G1 (nanoseconds).
    pub sweep_max_age_ns: i64,
    /// R:R floor in milli-R for mapping onto the score scale.
    pub rr_floor_milli: i64,
    /// R:R cap in milli-R for mapping onto the score scale.
    pub rr_cap_milli: i64,
    /// Minimum R:R (milli-R) for G2 / G4 gates.
    pub min_rr_milli: i64,
    /// News blackout window in minutes (stub; calendar not wired).
    pub news_blackout_min: i64,
    /// Weight: sweep contribution.
    pub w_sweep: i64,
    /// Weight: confluence signal.
    pub w_conf: i64,
    /// Weight: R:R estimate.
    pub w_rr: i64,
    /// Weight: regime alignment.
    pub w_regime: i64,
    /// Weight: session / kill-zone.
    pub w_session: i64,
    /// Raw score ≥ this → Green (subject to gates).
    pub thr_green: i64,
    /// Raw score ≥ this → Yellow (subject to gates).
    pub thr_yellow: i64,
    /// Kill-zone start hour UTC inclusive.
    pub killzone_start_hour_utc: u8,
    /// Kill-zone end hour UTC exclusive.
    pub killzone_end_hour_utc: u8,
}

/// `[sweep]` pierce / pre-close confirmation parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct SweepConfig {
    /// Minimum ticks beyond the pool to count as a pierce.
    pub min_pierce_ticks: i64,
    /// Minimum ticks back across the pool to count as reclaim.
    pub min_reclaim_ticks: i64,
    /// Max ticks after pierce for confirmation (exclusive of pierce tick).
    pub confirm_max_ticks: usize,
    /// Use bid/ask extremes instead of mid for pierce/reclaim.
    pub use_bid_ask_extremes: bool,
    /// Skip pools with score strictly below this threshold.
    pub min_pool_score: i64,
}

/// `[strategy]` trade builder parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    /// Minimum R:R numerator (e.g. 15 with den 10 ⇒ 1.5R).
    pub min_rr_num: i64,
    /// Minimum R:R denominator (must be `> 0`).
    pub min_rr_den: i64,
    /// Minimum confluence score (`0..=score_scale`).
    pub min_confluence: i64,
    /// Confluence score scale.
    pub score_scale: i64,
    /// Legacy stop buffer beyond pierce (ticks); combined via `max` with ATR formula.
    pub stop_buffer_ticks: i64,
    /// Pip/tick floor component of anti-sweep stop buffer.
    pub stop_buffer_pips_ticks: i64,
    /// ATR buffer numerator: `atr * num / den`.
    pub stop_atr_num: i64,
    /// ATR buffer denominator (`>= 1`).
    pub stop_atr_den: i64,
    /// Lookback for ATR proxy at confirm (ticks).
    pub stop_atr_lookback: usize,
    /// Fallback target distance in R units (num/den) when no opposing pool.
    pub fallback_rr_num: i64,
    /// Fallback target R denominator.
    pub fallback_rr_den: i64,
    /// Points awarded for a confirmed sweep factor.
    pub pts_sweep: i64,
    /// Points for equal-origin pool.
    pub pts_equal: i64,
    /// Points for session-origin pool.
    pub pts_session: i64,
    /// Points for trendline-origin pool.
    pub pts_trendline: i64,
    /// Points when displacement ≥ `strong_displacement_ticks`.
    pub pts_strong_displace: i64,
    /// Displacement threshold for strong-displace points.
    pub strong_displacement_ticks: i64,
}

/// `[backtest]` costs and walk-forward.
#[derive(Debug, Clone, Deserialize)]
pub struct BacktestConfig {
    /// Half-spread cost in ticks applied each side (entry+exit ⇒ ×2 if both).
    pub spread_ticks: i64,
    /// Commission in ticks per side.
    pub commission_ticks_per_side: i64,
    /// Slippage in ticks per side (legacy / base for fill model).
    pub slippage_ticks_per_side: i64,
    /// Walk-forward train window length in ticks.
    pub walk_train_ticks: usize,
    /// Walk-forward test window length in ticks.
    pub walk_test_ticks: usize,
    /// Max bars to hold a plan before time-stop (0 = hold to series end).
    pub max_hold_ticks: usize,
    /// Base market-entry slippage in ticks (before vol / jitter).
    pub fill_base_slippage_ticks: i64,
    /// Local range lookback for vol-factor slippage.
    pub fill_vol_lookback: usize,
    /// Vol-factor slippage numerator (`range * num / den`).
    pub fill_vol_slippage_num: i64,
    /// Vol-factor slippage denominator (`>= 1`).
    pub fill_vol_slippage_den: i64,
    /// Max absolute Xoshiro jitter in ticks for market entries.
    pub fill_jitter_max_ticks: i64,
    /// Distance (ticks) from plan stop/pierce region counted as sweep-proximate.
    pub fill_sweep_proximity_ticks: i64,
    /// Spread widen numerator when sweep-proximate.
    pub fill_sweep_spread_mult_num: i64,
    /// Spread widen denominator when sweep-proximate (`>= 1`).
    pub fill_sweep_spread_mult_den: i64,
}

/// Position sizing mode (`[risk].sizing_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizingMode {
    /// Fixed fraction of equity via `risk_per_trade_bps`.
    FixedBps,
    /// Kelly fraction (milli-scaled), capped by `risk_per_trade_bps`.
    Kelly,
}

/// `[risk]` sizing and guardrails.
#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    /// Risk per trade in basis points of equity (`100` = 1%); also Kelly cap.
    pub risk_per_trade_bps: i64,
    /// Maximum concurrent open plans.
    pub max_open_plans: u32,
    /// Daily loss kill threshold in ticks (absolute `PnL`).
    pub max_daily_loss_ticks: i64,
    /// Minimum stop distance in ticks for sizing.
    pub min_stop_ticks: i64,
    /// Default paper equity in ticks for sizing demos.
    pub default_equity_ticks: i64,
    /// `fixed_bps` or `kelly`.
    pub sizing_mode: SizingMode,
    /// Kelly fraction of full Kelly in milli (`250` = 0.25).
    pub kelly_fraction_milli: i64,
    /// Assumed win probability in milli (`550` = 0.55). Research input only.
    pub kelly_win_prob_milli: i64,
    /// Assumed payoff ratio `b = reward/risk` in milli (`1500` = 1.5).
    pub kelly_payoff_milli: i64,
    /// Trip feed kill when spread (ticks) exceeds this (`0` = disabled).
    pub max_spread_ticks: i64,
    /// Trip feed kill when tick latency (ns) exceeds this (`0` = disabled).
    pub max_tick_latency_ns: i64,
}

/// `[advisory]` regime / window scoring.
#[derive(Debug, Clone, Deserialize)]
pub struct AdvisoryConfig {
    /// Window length in ticks for regime / `WindowScore`.
    pub window_ticks: usize,
    /// Score scale for `WindowScore`.
    pub score_scale: i64,
    /// ATR proxy threshold (ticks) above which regime is Volatile.
    pub volatile_atr_ticks: i64,
    /// Net mid drift (ticks) to label `TrendUp` / `TrendDown`.
    pub trend_drift_ticks: i64,
    /// Minimum suitability confluence (mirrors strategy scale).
    pub min_suitability_confluence: i64,
}

/// `[journal]` paper / stats.
#[derive(Debug, Clone, Deserialize)]
pub struct JournalConfig {
    /// Max journal entries retained in-memory.
    pub max_entries: usize,
    /// Paper fill uses mid ± this many ticks.
    pub paper_slippage_ticks: i64,
}

/// `[api]` advisory service bind defaults (secrets via env).
#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    /// HTTP listen port.
    pub http_port: u16,
    /// Enable Telegram alert attempts when env token present.
    pub telegram_enabled: bool,
}

impl AppConfig {
    /// Load TOML from a filesystem path.
    ///
    /// # Errors
    /// Returns [`SmcError::Config`] on read/parse failure.
    pub fn load_path(path: impl AsRef<Path>) -> Result<Self, SmcError> {
        let raw = fs::read_to_string(path.as_ref())
            .map_err(|e| SmcError::Config(format!("read {}: {e}", path.as_ref().display())))?;
        Self::parse_toml(&raw)
    }

    /// Parse TOML text.
    ///
    /// # Errors
    /// Returns [`SmcError::Config`] on parse failure or weight-sum validation errors.
    pub fn parse_toml(raw: &str) -> Result<Self, SmcError> {
        let cfg: Self = toml::from_str(raw).map_err(|e| SmcError::Config(format!("toml: {e}")))?;
        cfg.validate_score_weights()?;
        Ok(cfg)
    }

    fn validate_score_weights(&self) -> Result<(), SmcError> {
        let ls = &self.liquidity_score;
        let liq_sum = ls
            .w_touches
            .saturating_add(ls.w_equality)
            .saturating_add(ls.w_recency)
            .saturating_add(ls.w_distance)
            .saturating_add(ls.w_context);
        if liq_sum != ls.score_scale {
            return Err(SmcError::Config(format!(
                "liquidity_score weights sum {liq_sum} != score_scale {}",
                ls.score_scale
            )));
        }
        let ws = &self.window_score;
        let win_sum = ws
            .w_sweep
            .saturating_add(ws.w_conf)
            .saturating_add(ws.w_rr)
            .saturating_add(ws.w_regime)
            .saturating_add(ws.w_session);
        if win_sum != ws.score_scale {
            return Err(SmcError::Config(format!(
                "window_score weights sum {win_sum} != score_scale {}",
                ws.score_scale
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_defaults() {
        let raw = include_str!("../../../config/default.toml");
        let cfg = AppConfig::parse_toml(raw).expect("default.toml must parse");
        assert_eq!(cfg.instrument.default.symbol, "EURUSD");
        assert!(cfg.instrument.default.tick_size >= 1);
        assert!(!cfg.disclaimer.text.is_empty());
        assert!(cfg.disclaimer.text.to_ascii_lowercase().contains("risk"));
        assert_eq!(cfg.liquidity_score.score_scale, 10_000);
        assert_eq!(cfg.window_score.score_scale, 10_000);
        assert_eq!(cfg.strategy.stop_buffer_pips_ticks, 2);
        assert_eq!(cfg.strategy.stop_atr_den, 4);
        assert_eq!(cfg.backtest.fill_jitter_max_ticks, 2);
        assert!(matches!(
            cfg.risk.sizing_mode,
            crate::config::SizingMode::FixedBps
        ));
        assert_eq!(cfg.risk.kelly_fraction_milli, 250);
    }

    #[test]
    fn rejects_liquidity_weight_mismatch() {
        let raw = include_str!("../../../config/default.toml");
        let mut bad = raw.to_string();
        bad = bad.replace("w_touches = 2500", "w_touches = 2501");
        let err = AppConfig::parse_toml(&bad).expect_err("weights must match scale");
        assert!(err.to_string().contains("liquidity_score weights"));
    }
}
