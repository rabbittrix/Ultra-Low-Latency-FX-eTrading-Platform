//! Property tests for sweep detection invariants.

use fx_smc_common::{Px, Qty, SweepConfig, SymbolId, Tick, TsNanos};
use fx_smc_liquidity::{LiquidityPool, PoolId, PoolOrigin, PoolSide};
use fx_smc_sweep::detect_sweeps;
use proptest::prelude::*;

fn cfg() -> SweepConfig {
    SweepConfig {
        min_pierce_ticks: 1,
        min_reclaim_ticks: 0,
        confirm_max_ticks: 8,
        use_bid_ask_extremes: true,
        min_pool_score: 0,
    }
}

fn pool(side: PoolSide, price: i64) -> LiquidityPool {
    LiquidityPool {
        id: PoolId::new("prop"),
        side,
        price: Px(price),
        touches: 2,
        last_touch_ns: TsNanos(0),
        score: 1,
        origin: PoolOrigin::Equal,
    }
}

fn mk_tick(i: usize, bid: i64, ask: i64) -> Tick {
    let ask = ask.max(bid);
    let ts = i64::try_from(i).unwrap_or(i64::MAX).saturating_mul(1_000);
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn confirm_after_pierce_indices(
        pool_px in 50i64..150i64,
        pierce_drop in 1i64..20i64,
        reclaim_up in 0i64..20i64,
    ) {
        // BuySide: pierce below, reclaim above.
        let pools = [pool(PoolSide::BuySide, pool_px)];
        let pierce_bid = pool_px - pierce_drop;
        let reclaim_ask = pool_px + reclaim_up;
        let ticks = [
            mk_tick(0, pool_px + 5, pool_px + 7),
            mk_tick(1, pierce_bid, pierce_bid + 2),
            mk_tick(2, reclaim_ask - 1, reclaim_ask.max(reclaim_ask)),
        ];
        let ev = detect_sweeps(&ticks, &pools, &cfg());
        if reclaim_ask >= pool_px {
            prop_assert_eq!(ev.len(), 1);
            prop_assert!(ev[0].confirm_idx > ev[0].pierce_idx);
            prop_assert!(ev[0].displacement_ticks >= 1);
        }
    }

    #[test]
    fn empty_ticks_empty_events(side in prop::sample::select(vec![PoolSide::BuySide, PoolSide::SellSide])) {
        let pools = [pool(side, 100)];
        let ev = detect_sweeps(&[], &pools, &cfg());
        prop_assert!(ev.is_empty());
    }
}
