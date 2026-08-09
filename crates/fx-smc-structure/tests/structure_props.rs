//! Property tests and end-to-end structure pipeline checks.

use fx_smc_common::AppConfig;
use fx_smc_marketdata::{generate_ticks, SynthParams};
use fx_smc_structure::equal::{cluster_equal_levels, EqualKind};
use fx_smc_structure::geom::{atr_proxy_ticks, equal_tolerance_ticks};
use fx_smc_structure::session::scan_session_levels;
use fx_smc_structure::swing::{detect_swings, SwingKind};
use fx_smc_structure::trendline::detect_trendlines;
use proptest::prelude::*;

#[test]
fn pipeline_on_synth_is_deterministic() {
    let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
    let meta = cfg.instrument.default.to_meta();
    let mut params = SynthParams::from_config(&cfg.synth, &meta);
    params.tick_count = 3_000;
    let ticks = generate_ticks(&params);

    let swings_a = detect_swings(&ticks, &cfg.structure.swings);
    let swings_b = detect_swings(&ticks, &cfg.structure.swings);
    assert_eq!(swings_a, swings_b);

    let atr = atr_proxy_ticks(&ticks, cfg.structure.equal.atr_lookback);
    let tol = equal_tolerance_ticks(&cfg.structure.equal, atr);
    let eq_a = cluster_equal_levels(&swings_a, tol);
    let eq_b = cluster_equal_levels(&swings_b, tol);
    assert_eq!(eq_a, eq_b);

    let lines_a = detect_trendlines(&swings_a, &cfg.structure.trendline);
    let lines_b = detect_trendlines(&swings_b, &cfg.structure.trendline);
    assert_eq!(lines_a, lines_b);

    let sess = scan_session_levels(&ticks, &cfg.structure.sessions);
    if let (Some(h), Some(l)) = (sess.wh, sess.wl) {
        assert!(h.0 >= l.0);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Equal clusters never mix highs and lows; members stay within tolerance of seed.
    #[test]
    fn equal_cluster_invariants(
        mids in prop::collection::vec(1_000i64..2_000i64, 30..80),
        tol in 0i64..20i64,
    ) {
        use fx_smc_common::{Px, Qty, SymbolId, Tick, TsNanos};
        let ticks: Vec<Tick> = mids.iter().enumerate().map(|(i, m)| Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(i64::try_from(i).unwrap_or(0) * 1_000_000),
            bid: Px(*m),
            ask: Px(*m + 1),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }).collect();
        let cfg = fx_smc_common::SwingConfig { left_strength: 1, right_strength: 1 };
        let swings = detect_swings(&ticks, &cfg);
        let clusters = cluster_equal_levels(&swings, tol);
        for c in clusters {
            for &idx in &c.members {
                let s = &swings[idx];
                match c.kind {
                    EqualKind::Highs => prop_assert_eq!(s.kind, SwingKind::High),
                    EqualKind::Lows => prop_assert_eq!(s.kind, SwingKind::Low),
                }
                prop_assert!((s.price.0 - c.price.0).abs() <= tol);
            }
        }
    }

    /// Trendlines always have dt != 0 and touch_count >= min_touches.
    #[test]
    fn trendline_invariants(
        mids in prop::collection::vec(1_000i64..2_000i64, 40..100),
    ) {
        use fx_smc_common::{Px, Qty, SymbolId, Tick, TsNanos, TrendlineConfig};
        let ticks: Vec<Tick> = mids.iter().enumerate().map(|(i, m)| Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(i64::try_from(i).unwrap_or(0) * 1_000_000),
            bid: Px(*m),
            ask: Px(*m + 1),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }).collect();
        let swing_cfg = fx_smc_common::SwingConfig { left_strength: 1, right_strength: 1 };
        let swings = detect_swings(&ticks, &swing_cfg);
        let tl_cfg = TrendlineConfig { min_touches: 2, touch_tolerance_ticks: 5 };
        for line in detect_trendlines(&swings, &tl_cfg) {
            prop_assert!(line.dt_ns != 0);
            prop_assert!(line.touch_count() >= 2);
        }
    }
}
