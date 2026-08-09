//! Sweep state machine over ordered ticks.

use crate::event::SweepEvent;
use fx_smc_common::{
    LiquidityConfig, LiquidityScoreConfig, Px, StructureConfig, SweepConfig, Tick, TsNanos,
};
use fx_smc_liquidity::{map_from_ticks, LiquidityPool, PoolSide};

#[derive(Clone, Copy)]
struct PierceState {
    idx: usize,
    px: i64,
    ts: TsNanos,
}

/// Detect confirmed sweeps for the given pools over `ticks` (no look-ahead).
#[must_use]
pub fn detect_sweeps(
    ticks: &[Tick],
    pools: &[LiquidityPool],
    cfg: &SweepConfig,
) -> Vec<SweepEvent> {
    let pierce_need = cfg.min_pierce_ticks.max(0);
    let reclaim_need = cfg.min_reclaim_ticks.max(0);
    let window = cfg.confirm_max_ticks.max(1);
    let extremes = cfg.use_bid_ask_extremes;

    let mut events = Vec::new();
    for pool in pools {
        if pool.score < cfg.min_pool_score {
            continue;
        }
        if let Some(ev) = scan_pool(ticks, pool, pierce_need, reclaim_need, window, extremes) {
            events.push(ev);
        }
    }
    events.sort_by(|a, b| {
        a.confirm_ts_ns
            .0
            .cmp(&b.confirm_ts_ns.0)
            .then_with(|| a.pool_id.0.cmp(&b.pool_id.0))
            .then_with(|| a.pierce_idx.cmp(&b.pierce_idx))
    });
    events
}

/// Map structure → pools → sweeps from ticks + configs.
#[must_use]
pub fn detect_sweeps_from_ticks(
    ticks: &[Tick],
    structure: &StructureConfig,
    liquidity: &LiquidityConfig,
    score: &LiquidityScoreConfig,
    sweep: &SweepConfig,
) -> Vec<SweepEvent> {
    let pools = map_from_ticks(ticks, structure, liquidity, score);
    detect_sweeps(ticks, &pools, sweep)
}

fn scan_pool(
    ticks: &[Tick],
    pool: &LiquidityPool,
    pierce_need: i64,
    reclaim_need: i64,
    window: usize,
    extremes: bool,
) -> Option<SweepEvent> {
    let mut pending: Option<PierceState> = None;

    for (idx, tick) in ticks.iter().enumerate() {
        if let Some(state) = pending {
            let expired = idx > state.idx.saturating_add(window);
            if expired {
                pending = None;
            } else if idx > state.idx
                && is_reclaim(tick, pool.side, pool.price, reclaim_need, extremes)
            {
                let displacement = (state.px - pool.price.0).abs();
                return Some(SweepEvent {
                    pool_id: pool.id.clone(),
                    side: pool.side,
                    pool_price_ticks: pool.price.0,
                    pierce_price_ticks: state.px,
                    displacement_ticks: displacement,
                    pierce_ts_ns: state.ts,
                    confirm_ts_ns: tick.ts_ns,
                    pierce_idx: state.idx,
                    confirm_idx: idx,
                });
            }
        }

        if pending.is_none() {
            if let Some(px) = pierce_price(tick, pool.side, pool.price, pierce_need, extremes) {
                pending = Some(PierceState {
                    idx,
                    px,
                    ts: tick.ts_ns,
                });
            }
        }
    }
    None
}

fn pierce_price(tick: &Tick, side: PoolSide, pool: Px, need: i64, extremes: bool) -> Option<i64> {
    match side {
        PoolSide::SellSide => {
            let px = if extremes {
                tick.ask.0
            } else {
                tick.mid_ticks().0
            };
            if px >= pool.0.saturating_add(need) {
                Some(px)
            } else {
                None
            }
        }
        PoolSide::BuySide => {
            let px = if extremes {
                tick.bid.0
            } else {
                tick.mid_ticks().0
            };
            if px <= pool.0.saturating_sub(need) {
                Some(px)
            } else {
                None
            }
        }
    }
}

fn is_reclaim(tick: &Tick, side: PoolSide, pool: Px, need: i64, extremes: bool) -> bool {
    match side {
        PoolSide::SellSide => {
            let px = if extremes {
                tick.bid.0
            } else {
                tick.mid_ticks().0
            };
            px <= pool.0.saturating_sub(need)
        }
        PoolSide::BuySide => {
            let px = if extremes {
                tick.ask.0
            } else {
                tick.mid_ticks().0
            };
            px >= pool.0.saturating_add(need)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{AppConfig, Qty, Side, SymbolId, Tick};
    use fx_smc_liquidity::{LiquidityPool, PoolId, PoolOrigin, PoolSide};

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

    fn pool(id: &str, side: PoolSide, price: i64) -> LiquidityPool {
        LiquidityPool {
            id: PoolId::new(id),
            side,
            price: Px(price),
            touches: 2,
            last_touch_ns: TsNanos(0),
            score: 5_000,
            origin: PoolOrigin::Equal,
        }
    }

    fn cfg() -> SweepConfig {
        SweepConfig {
            min_pierce_ticks: 1,
            min_reclaim_ticks: 0,
            confirm_max_ticks: 5,
            use_bid_ask_extremes: true,
            min_pool_score: 0,
        }
    }

    #[test]
    fn buy_side_sweep_confirms() {
        // Pool at 100 buy-side: pierce bid <= 99, reclaim ask >= 100.
        let pools = [pool("p1", PoolSide::BuySide, 100)];
        let ticks = [
            tick(1, 101, 103),
            tick(2, 98, 100), // pierce
            tick(3, 99, 101), // reclaim via ask >= 100
        ];
        let ev = detect_sweeps(&ticks, &pools, &cfg());
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].pierce_idx, 1);
        assert_eq!(ev[0].confirm_idx, 2);
        assert!(ev[0].displacement_ticks >= 1);
    }

    #[test]
    fn sell_side_sweep_confirms() {
        let pools = [pool("p2", PoolSide::SellSide, 100)];
        let ticks = [
            tick(1, 97, 99),
            tick(2, 100, 102), // pierce ask >= 101
            tick(3, 99, 101),  // reclaim bid <= 100
        ];
        let ev = detect_sweeps(&ticks, &pools, &cfg());
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].side, PoolSide::SellSide);
    }

    #[test]
    fn fake_sweep_no_reclaim_emits_nothing() {
        let pools = [pool("p3", PoolSide::BuySide, 100)];
        let ticks = [
            tick(1, 101, 103),
            tick(2, 90, 92), // pierce deep
            tick(3, 88, 90),
            tick(4, 87, 89),
            tick(5, 86, 88),
            tick(6, 85, 87),
            tick(7, 84, 86), // window expired (confirm_max=5 after pierce)
        ];
        let ev = detect_sweeps(&ticks, &pools, &cfg());
        assert!(ev.is_empty());
    }

    #[test]
    fn deterministic_on_synth() {
        let app = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let meta = app.instrument.default.to_meta();
        let mut params = fx_smc_marketdata::SynthParams::from_config(&app.synth, &meta);
        params.tick_count = 3_000;
        let ticks = fx_smc_marketdata::generate_ticks(&params);
        let a = detect_sweeps_from_ticks(
            &ticks,
            &app.structure,
            &app.liquidity,
            &app.liquidity_score,
            &app.sweep,
        );
        let b = detect_sweeps_from_ticks(
            &ticks,
            &app.structure,
            &app.liquidity,
            &app.liquidity_score,
            &app.sweep,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn aggressor_field_unused_but_ticks_ok() {
        let mut t = tick(1, 100, 102);
        t.aggressor = Some(Side::Sell);
        assert!(t.aggressor.is_some());
    }
}
