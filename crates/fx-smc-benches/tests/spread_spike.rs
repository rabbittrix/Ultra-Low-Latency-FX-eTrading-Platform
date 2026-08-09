//! Spread / spike resilience tests (M10).

use fx_smc_common::{AppConfig, Px, Qty, SymbolId, Tick, TsNanos};
use fx_smc_liquidity::map_from_ticks;
use fx_smc_marketdata::{generate_ticks, SynthParams};
use fx_smc_sweep::detect_sweeps;

fn spike_series() -> Vec<Tick> {
    let mut ticks = Vec::new();
    let mut mid = 11_000i64;
    for i in 0..500 {
        if i == 250 {
            mid = mid.saturating_add(200); // spike
        } else if i == 251 {
            mid = mid.saturating_sub(200); // revert
        } else if i % 17 == 0 {
            mid = mid.saturating_add(if i % 2 == 0 { 3 } else { -3 });
        }
        let spread = if i == 250 { 40 } else { 2 }; // wide spread on spike
        ticks.push(Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(i),
            bid: Px(mid),
            ask: Px(mid.saturating_add(spread)),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        });
    }
    ticks
}

#[test]
fn spike_and_wide_spread_do_not_panic_and_stay_deterministic() {
    let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
    let ticks = spike_series();
    let a_pools = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
    let b_pools = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
    assert_eq!(a_pools, b_pools);
    let a = detect_sweeps(&ticks, &a_pools, &cfg.sweep);
    let b = detect_sweeps(&ticks, &b_pools, &cfg.sweep);
    assert_eq!(a, b);
    for t in &ticks {
        assert!(t.spread_ticks() >= 0);
        assert!(t.ask.0 >= t.bid.0);
    }
}

#[test]
fn synth_with_sweeps_hashes_stable_under_rerun() {
    let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
    let meta = cfg.instrument.default.to_meta();
    let mut p = SynthParams::from_config(&cfg.synth, &meta);
    p.tick_count = 1_500;
    p.sweep_every = 100;
    p.sweep_break_ticks = 12;
    let a = generate_ticks(&p);
    let b = generate_ticks(&p);
    assert_eq!(a, b);
    let mut max_spread = 0i64;
    for t in &a {
        max_spread = max_spread.max(t.spread_ticks());
    }
    assert!(max_spread >= 0);
}
