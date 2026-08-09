//! Entry-window traffic-light scoring (ADR-0012).

use crate::fact::Fact;
use crate::regime::Regime;
use fx_smc_common::{TsNanos, WindowScoreConfig};
use fx_smc_liquidity::{half_life_decay, LiquidityPool, PoolSide};
use fx_smc_sweep::SweepEvent;
use serde::{Deserialize, Serialize};

/// Traffic-light color after gates + thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowColor {
    /// High conviction window (gates clear, raw ≥ green threshold).
    Green,
    /// Caution / incomplete (or mid score).
    Yellow,
    /// Blocked or low score.
    Red,
}

/// Intended entry side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntrySide {
    /// Long / buy.
    Buy,
    /// Short / sell.
    Sell,
}

/// Confluence / structure confirmation from BOS / `CHoCH` (or HTF bias).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfSignal {
    /// Change of character.
    ChoCh,
    /// Break of structure.
    Bos,
    /// Higher-timeframe bias only.
    HtfBias,
    /// No confluence available.
    None,
}

/// Map the latest structure break to a confluence signal.
#[must_use]
pub fn conf_from_structure_breaks(breaks: &[fx_smc_structure::StructureBreak]) -> ConfSignal {
    use fx_smc_structure::StructureBreakKind;
    match breaks.last() {
        Some(b) if b.kind == StructureBreakKind::ChoCh => ConfSignal::ChoCh,
        Some(b) if b.kind == StructureBreakKind::Bos => ConfSignal::Bos,
        Some(_) => ConfSignal::HtfBias,
        None => ConfSignal::None,
    }
}

/// Scored entry window with explainable facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryWindowScore {
    /// Side scored.
    pub side: EntrySide,
    /// Raw score in `0..=score_scale`.
    pub raw: i64,
    /// Traffic light after gates.
    pub color: WindowColor,
    /// Explainable facts (includes disclaimer).
    pub facts: Vec<Fact>,
    /// Short human summary.
    pub summary: String,
}

/// Score one entry side (ADR-0012).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn score_entry_window(
    side: EntrySide,
    sweeps: &[SweepEvent],
    pools: &[LiquidityPool],
    rr_est_milli: i64,
    conf: ConfSignal,
    regime: Regime,
    now_ns: TsNanos,
    hour_utc: u8,
    news_high_impact: bool,
    vol_above_p95: bool,
    cfg: &WindowScoreConfig,
) -> EntryWindowScore {
    let scale = cfg.score_scale.max(1);
    let mut facts = Vec::new();

    let (s_sweep, sweep_fact) = sweep_component(side, sweeps, pools, now_ns, cfg, scale);
    facts.push(sweep_fact);

    let (s_conf, conf_fact) = conf_component(conf, scale);
    facts.push(conf_fact);
    if matches!(conf, ConfSignal::None) {
        facts.push(Fact::DataDegraded {
            reason: "ConfSignal::None — no BOS/CHoCH on current prefix".into(),
        });
    }

    let (s_rr, rr_fact) = rr_component(rr_est_milli, cfg, scale);
    facts.push(rr_fact);

    let (s_reg, reg_fact) = regime_component(side, regime, scale);
    facts.push(reg_fact);

    let (s_sess, sess_fact) = session_component(hour_utc, cfg, scale);
    facts.push(sess_fact);

    let weighted = cfg
        .w_sweep
        .saturating_mul(s_sweep)
        .saturating_add(cfg.w_conf.saturating_mul(s_conf))
        .saturating_add(cfg.w_rr.saturating_mul(s_rr))
        .saturating_add(cfg.w_regime.saturating_mul(s_reg))
        .saturating_add(cfg.w_session.saturating_mul(s_sess));
    let raw = (weighted / scale).clamp(0, scale);

    let g1 = !has_fresh_opposite_sweep(side, sweeps, pools, now_ns, cfg);
    let g2 = rr_est_milli < cfg.min_rr_milli;
    let g3 = vol_above_p95 || news_high_impact;
    let g4 = g1 && g2;

    if g1 {
        facts.push(Fact::Gate {
            id: "G1",
            reason: "no confirmed opposite-side sweep within sweep_max_age".into(),
        });
    }
    if g2 {
        facts.push(Fact::Gate {
            id: "G2",
            reason: format!(
                "rr_est_milli {rr_est_milli} < min_rr_milli {}",
                cfg.min_rr_milli
            ),
        });
    }
    if g3 {
        let reason = if vol_above_p95 && news_high_impact {
            "vol_above_p95 and news_high_impact".into()
        } else if vol_above_p95 {
            "vol_above_p95".into()
        } else {
            format!(
                "news_high_impact (blackout stub {}m)",
                cfg.news_blackout_min
            )
        };
        facts.push(Fact::Gate { id: "G3", reason });
    }
    if g4 {
        facts.push(Fact::Gate {
            id: "G4",
            reason: "G1 and G2 both active".into(),
        });
    }

    let mut color = threshold_color(raw, cfg);
    if g3 || g4 {
        color = WindowColor::Red;
    } else if (g1 || g2) && matches!(color, WindowColor::Green) {
        color = WindowColor::Yellow;
    }

    facts.push(Fact::Disclaimer {
        text: "Informational only — not investment advice; no returns promised.".into(),
    });

    let summary = format!(
        "{side:?} raw={raw} color={color:?} sweep={s_sweep} conf={s_conf} rr={s_rr} reg={s_reg} sess={s_sess}"
    );

    EntryWindowScore {
        side,
        raw,
        color,
        facts,
        summary,
    }
}

/// Score both sides and return the better one (higher `raw`; Buy wins ties).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn best_entry_window(
    sweeps: &[SweepEvent],
    pools: &[LiquidityPool],
    rr_est_milli: i64,
    conf: ConfSignal,
    regime: Regime,
    now_ns: TsNanos,
    hour_utc: u8,
    news_high_impact: bool,
    vol_above_p95: bool,
    cfg: &WindowScoreConfig,
) -> EntryWindowScore {
    let buy = score_entry_window(
        EntrySide::Buy,
        sweeps,
        pools,
        rr_est_milli,
        conf,
        regime,
        now_ns,
        hour_utc,
        news_high_impact,
        vol_above_p95,
        cfg,
    );
    let sell = score_entry_window(
        EntrySide::Sell,
        sweeps,
        pools,
        rr_est_milli,
        conf,
        regime,
        now_ns,
        hour_utc,
        news_high_impact,
        vol_above_p95,
        cfg,
    );
    if sell.raw > buy.raw {
        sell
    } else {
        buy
    }
}

fn threshold_color(raw: i64, cfg: &WindowScoreConfig) -> WindowColor {
    if raw >= cfg.thr_green {
        WindowColor::Green
    } else if raw >= cfg.thr_yellow {
        WindowColor::Yellow
    } else {
        WindowColor::Red
    }
}

fn opposite_pool_side(side: EntrySide) -> PoolSide {
    // Buy after sell-side liquidity swept; Sell after buy-side swept.
    match side {
        EntrySide::Buy => PoolSide::SellSide,
        EntrySide::Sell => PoolSide::BuySide,
    }
}

fn has_fresh_opposite_sweep(
    side: EntrySide,
    sweeps: &[SweepEvent],
    pools: &[LiquidityPool],
    now_ns: TsNanos,
    cfg: &WindowScoreConfig,
) -> bool {
    let want = opposite_pool_side(side);
    let max_age = cfg.sweep_max_age_ns.max(0);
    for sw in sweeps {
        let age = now_ns.0.saturating_sub(sw.confirm_ts_ns.0).max(0);
        if age > max_age {
            continue;
        }
        if let Some(pool) = pools.iter().find(|p| p.id == sw.pool_id) {
            if pool.side == want {
                return true;
            }
        } else if sw.side == want {
            return true;
        }
    }
    false
}

fn sweep_component(
    side: EntrySide,
    sweeps: &[SweepEvent],
    pools: &[LiquidityPool],
    now_ns: TsNanos,
    cfg: &WindowScoreConfig,
    scale: i64,
) -> (i64, Fact) {
    let want = opposite_pool_side(side);
    let max_age = cfg.sweep_max_age_ns.max(0);
    let hl = cfg.sweep_half_life_ns.max(1);
    let mut best = 0i64;
    let mut best_id: Option<String> = None;
    for sw in sweeps {
        let age = now_ns.0.saturating_sub(sw.confirm_ts_ns.0).max(0);
        if age > max_age {
            continue;
        }
        let pool = pools.iter().find(|p| p.id == sw.pool_id);
        let pool_side = pool.map_or(sw.side, |p| p.side);
        if pool_side != want {
            continue;
        }
        let pool_score = pool.map_or(scale / 2, |p| p.score.clamp(0, scale));
        let decay = half_life_decay(age, hl, scale);
        let contrib = pool_score.saturating_mul(decay) / scale.max(1);
        if contrib >= best {
            best = contrib;
            best_id = Some(sw.pool_id.0.clone());
        }
    }
    (
        best.clamp(0, scale),
        Fact::Sweep {
            pool_id: best_id,
            score: best.clamp(0, scale),
        },
    )
}

fn conf_component(conf: ConfSignal, scale: i64) -> (i64, Fact) {
    let score = match conf {
        ConfSignal::ChoCh => scale,
        ConfSignal::Bos => (scale * 8) / 10,
        ConfSignal::HtfBias => (scale * 6) / 10,
        ConfSignal::None => scale / 5,
    };
    (
        score,
        Fact::Conf {
            signal: format!("{conf:?}"),
            score,
        },
    )
}

fn rr_component(rr_est_milli: i64, cfg: &WindowScoreConfig, scale: i64) -> (i64, Fact) {
    let floor = cfg.rr_floor_milli;
    let cap = cfg.rr_cap_milli.max(floor.saturating_add(1));
    let clamped = rr_est_milli.clamp(floor, cap);
    let span = cap.saturating_sub(floor).max(1);
    let score = (clamped.saturating_sub(floor)).saturating_mul(scale) / span;
    (
        score.clamp(0, scale),
        Fact::Rr {
            rr_milli: rr_est_milli,
            score: score.clamp(0, scale),
        },
    )
}

fn regime_component(side: EntrySide, regime: Regime, scale: i64) -> (i64, Fact) {
    let score = match (side, regime) {
        (EntrySide::Buy, Regime::TrendUp) | (EntrySide::Sell, Regime::TrendDown) => scale,
        (_, Regime::Range) => scale / 2,
        (EntrySide::Buy, Regime::TrendDown) | (EntrySide::Sell, Regime::TrendUp) => scale / 5,
        (_, Regime::Volatile) => scale / 4,
    };
    (
        score,
        Fact::Reg {
            regime: format!("{regime:?}"),
            score,
        },
    )
}

fn session_component(hour_utc: u8, cfg: &WindowScoreConfig, scale: i64) -> (i64, Fact) {
    let start = cfg.killzone_start_hour_utc;
    let end = cfg.killzone_end_hour_utc;
    let in_kz = if start <= end {
        hour_utc >= start && hour_utc < end
    } else {
        hour_utc >= start || hour_utc < end
    };
    // Low-liquidity stub: Asia deep hours 0..5 when killzone is London.
    let low = hour_utc < 5;
    let score = if in_kz {
        scale
    } else if low {
        scale / 3
    } else {
        (scale * 7) / 10
    };
    (score, Fact::Sess { hour_utc, score })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;
    use fx_smc_liquidity::{LiquidityPool, PoolId, PoolOrigin, PoolSide};

    fn cfg() -> WindowScoreConfig {
        AppConfig::parse_toml(include_str!("../../../config/default.toml"))
            .unwrap()
            .window_score
    }

    fn pool(id: &str, side: PoolSide, score: i64) -> LiquidityPool {
        LiquidityPool {
            id: PoolId::new(id),
            side,
            price: fx_smc_common::Px(100),
            touches: 3,
            last_touch_ns: TsNanos(0),
            score,
            origin: PoolOrigin::Equal,
        }
    }

    fn sweep(id: &str, side: PoolSide, confirm_ns: i64) -> SweepEvent {
        SweepEvent {
            pool_id: PoolId::new(id),
            side,
            pool_price_ticks: 100,
            pierce_price_ticks: 105,
            displacement_ticks: 5,
            pierce_ts_ns: TsNanos(confirm_ns.saturating_sub(1)),
            confirm_ts_ns: TsNanos(confirm_ns),
            pierce_idx: 0,
            confirm_idx: 1,
        }
    }

    /// ADR §3 pattern sanity: injected inputs → raw ≈ 0.865·scale → Green.
    #[test]
    fn golden_pattern_approx_865_green() {
        let c = cfg();
        let now = TsNanos(1_000);
        let pools = [pool("eq-0", PoolSide::SellSide, 10_000)];
        let sweeps = [sweep("eq-0", PoolSide::SellSide, 900)];
        // Components (scale=10000): sweep=10000, HtfBias=6000, rr 3900→7250,
        // TrendUp=10000, killzone=10000 → raw = 8650.
        let scored = score_entry_window(
            EntrySide::Buy,
            &sweeps,
            &pools,
            3_900,
            ConfSignal::HtfBias,
            Regime::TrendUp,
            now,
            8,
            false,
            false,
            &c,
        );
        assert!(
            (scored.raw - 8_650).abs() <= 50,
            "raw={} expected ~8650",
            scored.raw
        );
        assert_eq!(scored.color, WindowColor::Green);
    }

    #[test]
    fn g3_forces_red() {
        let c = cfg();
        let scored = score_entry_window(
            EntrySide::Buy,
            &[],
            &[],
            5_000,
            ConfSignal::ChoCh,
            Regime::TrendUp,
            TsNanos(0),
            8,
            true,
            false,
            &c,
        );
        assert_eq!(scored.color, WindowColor::Red);
        assert!(scored
            .facts
            .iter()
            .any(|f| matches!(f, Fact::Gate { id: "G3", .. })));
    }

    #[test]
    fn g1_caps_yellow() {
        let c = cfg();
        // No sweeps → G1; high other components still cannot be Green.
        let scored = score_entry_window(
            EntrySide::Buy,
            &[],
            &[],
            5_000,
            ConfSignal::ChoCh,
            Regime::TrendUp,
            TsNanos(0),
            8,
            false,
            false,
            &c,
        );
        assert!(matches!(
            scored.color,
            WindowColor::Yellow | WindowColor::Red
        ));
        assert_ne!(scored.color, WindowColor::Green);
        assert!(scored
            .facts
            .iter()
            .any(|f| matches!(f, Fact::Gate { id: "G1", .. })));
    }

    #[test]
    fn g2_caps_yellow() {
        let c = cfg();
        let pools = [pool("eq-0", PoolSide::SellSide, 10_000)];
        let sweeps = [sweep("eq-0", PoolSide::SellSide, 0)];
        let scored = score_entry_window(
            EntrySide::Buy,
            &sweeps,
            &pools,
            1_500, // below min_rr 3000
            ConfSignal::ChoCh,
            Regime::TrendUp,
            TsNanos(100),
            8,
            false,
            false,
            &c,
        );
        assert_ne!(scored.color, WindowColor::Green);
        assert!(scored
            .facts
            .iter()
            .any(|f| matches!(f, Fact::Gate { id: "G2", .. })));
    }

    #[test]
    fn g1_and_g2_red() {
        let c = cfg();
        let scored = score_entry_window(
            EntrySide::Buy,
            &[],
            &[],
            1_000,
            ConfSignal::ChoCh,
            Regime::TrendUp,
            TsNanos(0),
            8,
            false,
            false,
            &c,
        );
        assert_eq!(scored.color, WindowColor::Red);
        assert!(scored
            .facts
            .iter()
            .any(|f| matches!(f, Fact::Gate { id: "G4", .. })));
    }

    #[test]
    fn no_sweep_never_green() {
        let c = cfg();
        let scored = score_entry_window(
            EntrySide::Buy,
            &[],
            &[],
            5_000,
            ConfSignal::ChoCh,
            Regime::TrendUp,
            TsNanos(0),
            8,
            false,
            false,
            &c,
        );
        assert_ne!(scored.color, WindowColor::Green);
    }

    #[test]
    fn mirror_symmetry() {
        let c = cfg();
        let now = TsNanos(1_000);
        let buy_pools = [pool("s", PoolSide::SellSide, 9_000)];
        let buy_sweeps = [sweep("s", PoolSide::SellSide, 900)];
        let sell_pools = [pool("b", PoolSide::BuySide, 9_000)];
        let sell_sweeps = [sweep("b", PoolSide::BuySide, 900)];
        let buy = score_entry_window(
            EntrySide::Buy,
            &buy_sweeps,
            &buy_pools,
            4_000,
            ConfSignal::Bos,
            Regime::TrendUp,
            now,
            8,
            false,
            false,
            &c,
        );
        let sell = score_entry_window(
            EntrySide::Sell,
            &sell_sweeps,
            &sell_pools,
            4_000,
            ConfSignal::Bos,
            Regime::TrendDown,
            now,
            8,
            false,
            false,
            &c,
        );
        assert_eq!(buy.raw, sell.raw);
        assert_eq!(buy.color, sell.color);
    }
}
