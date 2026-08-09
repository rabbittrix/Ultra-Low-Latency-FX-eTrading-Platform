//! Property tests for liquidity scoring / mapping.

use fx_smc_common::{AppConfig, LiquidityScoreConfig, Px, TsNanos};
use fx_smc_liquidity::pool::PoolOrigin;
use fx_smc_liquidity::{score_pool, PoolScoreInput};
use proptest::prelude::*;

fn base_cfg() -> LiquidityScoreConfig {
    AppConfig::parse_toml(include_str!("../../../config/default.toml"))
        .unwrap()
        .liquidity_score
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn score_in_range(
        price in -5_000i64..5_000i64,
        mid in -5_000i64..5_000i64,
        touches in 0u32..20u32,
        age in 0i64..5_000_000i64,
        session in prop::bool::ANY,
        atr in 1i64..200i64,
    ) {
        let cfg = base_cfg();
        let now = TsNanos(10_000_000);
        let last = TsNanos(now.0.saturating_sub(age));
        let origin = if session { PoolOrigin::Asia } else { PoolOrigin::Equal };
        let s = score_pool(
            &PoolScoreInput {
                price: Px(price),
                touches,
                last_touch_ns: last,
                origin,
                mid: Px(mid),
                now_ns: now,
                equality_std_ticks: None,
                atr_ticks: atr,
            },
            &cfg,
        );
        prop_assert!(s >= 0);
        prop_assert!(s <= cfg.score_scale);
    }

    #[test]
    fn more_touches_not_lower_when_other_equal(
        touches_a in 1u32..4u32,
        delta in 1u32..4u32,
    ) {
        let cfg = base_cfg();
        let now = TsNanos(1_000);
        let last = TsNanos(1_000);
        let a = score_pool(
            &PoolScoreInput {
                price: Px(100),
                touches: touches_a,
                last_touch_ns: last,
                origin: PoolOrigin::Equal,
                mid: Px(100),
                now_ns: now,
                equality_std_ticks: Some(0),
                atr_ticks: 10,
            },
            &cfg,
        );
        let b = score_pool(
            &PoolScoreInput {
                price: Px(100),
                touches: touches_a.saturating_add(delta).min(4),
                last_touch_ns: last,
                origin: PoolOrigin::Equal,
                mid: Px(100),
                now_ns: now,
                equality_std_ticks: Some(0),
                atr_ticks: 10,
            },
            &cfg,
        );
        prop_assert!(b >= a);
    }
}
