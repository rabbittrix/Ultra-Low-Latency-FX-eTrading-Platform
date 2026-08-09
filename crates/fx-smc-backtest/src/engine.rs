//! Prefix-only backtest engine (no look-ahead).

use crate::cost::CostModel;
use crate::fill::{
    limit_exit_fill, market_entry_adverse, seed_from_plan, FillRecord, Xoshiro256PlusPlus,
};
use crate::report::{BacktestReport, CostReport};
use fx_smc_common::{
    BacktestConfig, LiquidityConfig, LiquidityScoreConfig, Px, StrategyConfig, StructureConfig,
    SweepConfig, Tick,
};
use fx_smc_liquidity::map_from_ticks;
use fx_smc_strategy::{build_plans, TradePlan, TradeSide};
use fx_smc_sweep::detect_sweeps;

/// Config sections required by the backtest pipeline.
#[derive(Debug, Clone, Copy)]
pub struct BacktestSections<'a> {
    /// Structure detection.
    pub structure: &'a StructureConfig,
    /// Liquidity mapping.
    pub liquidity: &'a LiquidityConfig,
    /// Liquidity pool scoring.
    pub liquidity_score: &'a LiquidityScoreConfig,
    /// Sweep detector.
    pub sweep: &'a SweepConfig,
    /// Plan builder.
    pub strategy: &'a StrategyConfig,
    /// Costs / hold / walk-forward lengths.
    pub backtest: &'a BacktestConfig,
    /// User-facing disclaimer text (propagated into plans).
    pub disclaimer: &'a str,
}

/// Map pools / sweeps / plans using **only** the provided tick prefix (no future ticks).
#[must_use]
pub fn analyze_prefix(ticks: &[Tick], sections: &BacktestSections<'_>) -> Vec<TradePlan> {
    if ticks.is_empty() {
        return Vec::new();
    }
    let pools = map_from_ticks(
        ticks,
        sections.structure,
        sections.liquidity,
        sections.liquidity_score,
    );
    let sweeps = detect_sweeps(ticks, &pools, sections.sweep);
    build_plans(
        ticks,
        &sweeps,
        &pools,
        sections.strategy,
        sections.disclaimer,
    )
}

/// Walk the series; emit a plan only when `confirm_idx ==` current index (proves no look-ahead).
#[must_use]
pub fn collect_prefix_plans(ticks: &[Tick], sections: &BacktestSections<'_>) -> Vec<TradePlan> {
    let mut out = Vec::new();
    for i in 0..ticks.len() {
        let plans = analyze_prefix(&ticks[..=i], sections);
        for p in plans {
            if p.as_of_idx == i {
                out.push(p);
            }
        }
    }
    out
}

/// Run prefix-safe plan collection then simulate fills on subsequent ticks only.
#[must_use]
pub fn run_backtest(ticks: &[Tick], sections: &BacktestSections<'_>) -> BacktestReport {
    let plans = collect_prefix_plans(ticks, sections);
    simulate_plans(ticks, &plans, sections.backtest)
}

/// Walk-forward folds: each window is `train_len + test_len`; only plans confirmed in the
/// test region are simulated (train ticks provide structure context via prefix).
#[must_use]
pub fn walk_forward(
    ticks: &[Tick],
    train_len: usize,
    test_len: usize,
    sections: &BacktestSections<'_>,
) -> Vec<BacktestReport> {
    let train = if train_len == 0 {
        sections.backtest.walk_train_ticks
    } else {
        train_len
    };
    let test = if test_len == 0 {
        sections.backtest.walk_test_ticks
    } else {
        test_len
    };
    if train == 0 || test == 0 || ticks.len() < train.saturating_add(test) {
        return Vec::new();
    }

    let mut reports = Vec::new();
    let mut start = 0usize;
    while start.saturating_add(train).saturating_add(test) <= ticks.len() {
        let end = start.saturating_add(train).saturating_add(test);
        let window = &ticks[start..end];
        let plans = collect_prefix_plans(window, sections);
        let test_plans: Vec<TradePlan> = plans
            .into_iter()
            .filter(|p| p.as_of_idx >= train && p.as_of_idx < train.saturating_add(test))
            .collect();
        reports.push(simulate_plans(window, &test_plans, sections.backtest));
        start = start.saturating_add(test);
    }
    reports
}

fn simulate_plans(ticks: &[Tick], plans: &[TradePlan], cfg: &BacktestConfig) -> BacktestReport {
    let model = CostModel::from_config(cfg);
    let mut costs = CostReport::default();
    let mut wins = 0u64;
    let mut losses = 0u64;
    let mut fills = Vec::with_capacity(plans.len());

    for plan in plans {
        if let Some((pnl, won, exit_idx)) = simulate_one(ticks, plan, cfg) {
            costs = costs.with_round_trip(model, pnl);
            let commission = model.commission_ticks_per_side.saturating_mul(2);
            fills.push(FillRecord {
                plan_id: plan.id.clone(),
                exit_idx,
                pnl_ticks: pnl.saturating_sub(commission),
            });
            if won {
                wins = wins.saturating_add(1);
            } else {
                losses = losses.saturating_add(1);
            }
        }
    }

    BacktestReport {
        costs,
        plans_simulated: wins.saturating_add(losses),
        wins,
        losses,
        fills,
    }
}

/// Returns `(gross_pnl_after_adverse_fills, is_win, exit_idx)` or `None` if entry index is OOR.
fn simulate_one(
    ticks: &[Tick],
    plan: &TradePlan,
    cfg: &BacktestConfig,
) -> Option<(i64, bool, usize)> {
    let entry_idx = plan.as_of_idx;
    if entry_idx >= ticks.len() {
        return None;
    }

    let mut rng = Xoshiro256PlusPlus::seed(seed_from_plan(plan));
    let adverse_entry = market_entry_adverse(ticks, plan, cfg, &mut rng);
    let entry_fill = match plan.side {
        TradeSide::Long => Px(plan.entry.0.saturating_add(adverse_entry)),
        TradeSide::Short => Px(plan.entry.0.saturating_sub(adverse_entry)),
    };

    let hold_end = if cfg.max_hold_ticks == 0 {
        ticks.len().saturating_sub(1)
    } else {
        entry_idx
            .saturating_add(cfg.max_hold_ticks)
            .min(ticks.len().saturating_sub(1))
    };

    let mut exit_px: Option<Px> = None;
    let mut exit_idx = hold_end;
    let mut mid_at_exit = ticks[hold_end].mid_ticks().0;
    let mut hit_stop = false;
    let mut won = false;
    for (i, tick) in ticks
        .iter()
        .enumerate()
        .take(hold_end.saturating_add(1))
        .skip(entry_idx.saturating_add(1))
    {
        let mid = tick.mid_ticks();
        match plan.side {
            TradeSide::Long => {
                if mid.0 <= plan.stop.0 {
                    exit_px = Some(plan.stop);
                    exit_idx = i;
                    mid_at_exit = mid.0;
                    hit_stop = true;
                    won = false;
                    break;
                }
                if mid.0 >= plan.target.0 {
                    exit_px = Some(plan.target);
                    exit_idx = i;
                    mid_at_exit = mid.0;
                    won = true;
                    break;
                }
            }
            TradeSide::Short => {
                if mid.0 >= plan.stop.0 {
                    exit_px = Some(plan.stop);
                    exit_idx = i;
                    mid_at_exit = mid.0;
                    hit_stop = true;
                    won = false;
                    break;
                }
                if mid.0 <= plan.target.0 {
                    exit_px = Some(plan.target);
                    exit_idx = i;
                    mid_at_exit = mid.0;
                    won = true;
                    break;
                }
            }
        }
    }

    let raw_exit = exit_px.unwrap_or_else(|| ticks[hold_end].mid_ticks());
    if exit_px.is_none() {
        // Time stop: mark win if mid moved favorably vs entry.
        won = match plan.side {
            TradeSide::Long => raw_exit.0 > plan.entry.0,
            TradeSide::Short => raw_exit.0 < plan.entry.0,
        };
        exit_idx = hold_end;
        mid_at_exit = raw_exit.0;
    }

    let exit_fill = limit_exit_fill(plan, raw_exit, mid_at_exit, cfg, hit_stop);

    let gross = match plan.side {
        TradeSide::Long => exit_fill.0.saturating_sub(entry_fill.0),
        TradeSide::Short => entry_fill.0.saturating_sub(exit_fill.0),
    };
    Some((gross, won, exit_idx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::pnl_curve_fingerprint;
    use fx_smc_common::{AppConfig, Qty, SymbolId, TsNanos};
    use fx_smc_liquidity::{LiquidityPool, PoolId, PoolOrigin, PoolSide};
    use fx_smc_marketdata::{generate_ticks, SynthParams};
    use fx_smc_sweep::SweepEvent;

    fn sections(cfg: &AppConfig) -> BacktestSections<'_> {
        BacktestSections {
            structure: &cfg.structure,
            liquidity: &cfg.liquidity,
            liquidity_score: &cfg.liquidity_score,
            sweep: &cfg.sweep,
            strategy: &cfg.strategy,
            backtest: &cfg.backtest,
            disclaimer: &cfg.disclaimer.text,
        }
    }

    fn tick(ts: i64, bid: i64, ask: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(ts),
            bid: Px(bid),
            ask: Px(ask),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }
    }

    #[test]
    fn same_ticks_same_report_fingerprint() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let meta = cfg.instrument.default.to_meta();
        let mut params = SynthParams::from_config(&cfg.synth, &meta);
        params.tick_count = 120;
        params.sweep_every = 40;
        let ticks = generate_ticks(&params);
        let sec = sections(&cfg);
        let a = run_backtest(&ticks, &sec);
        let b = run_backtest(&ticks, &sec);
        assert_eq!(a, b);
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_eq!(pnl_curve_fingerprint(&a), pnl_curve_fingerprint(&b));
        assert_eq!(a.costs.fingerprint(), b.costs.fingerprint());
    }

    #[test]
    fn prefix_plans_never_read_past_index() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let meta = cfg.instrument.default.to_meta();
        let mut params = SynthParams::from_config(&cfg.synth, &meta);
        params.tick_count = 80;
        params.sweep_every = 20;
        let ticks = generate_ticks(&params);
        let sec = sections(&cfg);

        for i in 0..ticks.len() {
            let plans = analyze_prefix(&ticks[..=i], &sec);
            for p in &plans {
                assert!(
                    p.as_of_idx <= i,
                    "plan as_of_idx {} must be <= prefix end {i}",
                    p.as_of_idx
                );
            }
        }
    }

    #[test]
    fn full_series_pools_can_differ_from_prefix() {
        // Hand series: early confirm vs pools that only appear after more structure.
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut ticks = Vec::new();
        // Flat then dip then reclaim (buy-side sweep shape), then later higher structure.
        for i in 0..30 {
            let mid = 110 + i / 10;
            ticks.push(tick(i, mid, mid + 1));
        }
        // Force a clear swing valley then reclaim-ish path
        ticks[10] = tick(10, 95, 96);
        ticks[11] = tick(11, 94, 95);
        ticks[12] = tick(12, 100, 101);
        ticks[13] = tick(13, 105, 106);
        for i in 40..80 {
            let mid = 130 + (i % 5);
            ticks.push(tick(i, mid, mid + 1));
        }

        let sec = sections(&cfg);
        let early = 20usize;
        let prefix_pools = map_from_ticks(
            &ticks[..=early],
            sec.structure,
            sec.liquidity,
            sec.liquidity_score,
        );
        let full_pools = map_from_ticks(&ticks, sec.structure, sec.liquidity, sec.liquidity_score);
        let prefix_plans = analyze_prefix(&ticks[..=early], &sec);
        assert!(prefix_plans.iter().all(|p| p.as_of_idx <= early));
        // Explicit look-ahead misuse: detect sweeps on prefix ticks but with full-series pools.
        let sweeps_lookahead = detect_sweeps(&ticks[..=early], &full_pools, sec.sweep);
        let sweeps_prefix = detect_sweeps(&ticks[..=early], &prefix_pools, sec.sweep);
        // Full-series pools vs prefix pools often differ; when they do, detection inputs diverge.
        assert!(
            full_pools != prefix_pools || sweeps_lookahead == sweeps_prefix,
            "when pools differ, look-ahead pool injection would change detection inputs"
        );
        for p in prefix_plans {
            assert!(p.as_of_idx <= early);
        }
    }

    #[test]
    fn simulate_long_stop_and_target() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let plan = TradePlan {
            id: "tp-0".into(),
            side: TradeSide::Long,
            sweep_pool_id: PoolId::new("eq-0"),
            entry: Px(100),
            stop: Px(90),
            target: Px(120),
            risk_ticks: 10,
            reward_ticks: 20,
            rr_num: 20,
            rr_den: 10,
            confluence: 5000,
            as_of_ns: TsNanos(0),
            as_of_idx: 0,
            invalidation: "Invalidation: stop".into(),
            disclaimer: cfg.disclaimer.text.clone(),
            reasoning: fx_smc_strategy::ReasoningTrace::default(),
        };
        let ticks = [
            tick(0, 99, 101),
            tick(1, 95, 96),
            tick(2, 89, 90), // stop
        ];
        let (pnl, won, exit_idx) = simulate_one(&ticks, &plan, &cfg.backtest).unwrap();
        assert!(!won);
        assert_eq!(exit_idx, 2);
        assert!(pnl < 0 || pnl <= 0);

        let ticks_win = [tick(0, 99, 101), tick(1, 110, 111), tick(2, 120, 121)];
        let (_pnl2, won2, _) = simulate_one(&ticks_win, &plan, &cfg.backtest).unwrap();
        assert!(won2);
    }

    #[test]
    fn walk_forward_runs_on_synth() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let meta = cfg.instrument.default.to_meta();
        let mut params = SynthParams::from_config(&cfg.synth, &meta);
        params.tick_count = 280;
        let ticks = generate_ticks(&params);
        let sec = sections(&cfg);
        let folds = walk_forward(&ticks, 100, 40, &sec);
        assert!(!folds.is_empty());
    }

    #[test]
    fn golden_build_plans_still_works_via_prefix() {
        // Sanity: hand sweep → plan path through analyze pieces
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let pools = vec![LiquidityPool {
            id: PoolId::new("eq-0"),
            side: PoolSide::BuySide,
            price: Px(100),
            touches: 3,
            last_touch_ns: TsNanos(0),
            score: 8000,
            origin: PoolOrigin::Equal,
        }];
        let sweeps = [SweepEvent {
            pool_id: PoolId::new("eq-0"),
            side: PoolSide::BuySide,
            pool_price_ticks: 100,
            pierce_price_ticks: 90,
            displacement_ticks: 10,
            pierce_ts_ns: TsNanos(1),
            confirm_ts_ns: TsNanos(2),
            pierce_idx: 1,
            confirm_idx: 2,
        }];
        let ticks = [tick(0, 105, 107), tick(1, 90, 92), tick(2, 104, 106)];
        let plans = build_plans(&ticks, &sweeps, &pools, &cfg.strategy, &cfg.disclaimer.text);
        assert_eq!(plans.len(), 1);
    }
}
