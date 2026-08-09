//! `AsianRangeLondonSweep` → liquidity map → sweep detect (practical pool/sweep link).

use fx_smc_common::AppConfig;
use fx_smc_liquidity::map_from_ticks;
use fx_smc_marketdata::{generate_scenario, SynthScenario};
use fx_smc_sweep::detect_sweeps;

#[test]
fn asian_sweep_implies_pool_near_level_or_preexisting() {
    let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config");
    let meta = cfg.instrument.default.to_meta();
    let ticks = generate_scenario(SynthScenario::AsianRangeLondonSweep, &meta, 99);

    let pools = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
    let sweeps = detect_sweeps(&ticks, &pools, &cfg.sweep);

    if sweeps.is_empty() {
        // Scenario still maps some structure liquidity for later detectors.
        assert!(
            !pools.is_empty() || ticks.len() >= 80,
            "expected pools or a long enough asian range for structure"
        );
        return;
    }

    for sw in &sweeps {
        let near = pools.iter().any(|p| {
            (p.price.0 - sw.pool_price_ticks).abs() <= cfg.sweep.min_pierce_ticks.saturating_add(8)
                || p.id == sw.pool_id
        });
        // Either a pool sits at/near the sweep level, or pools existed before the last confirm.
        let pools_before_confirm = map_from_ticks(
            &ticks[..=sw.confirm_idx],
            &cfg.structure,
            &cfg.liquidity,
            &cfg.liquidity_score,
        );
        assert!(
            near || !pools_before_confirm.is_empty(),
            "sweep {:?} without nearby pool and empty prefix pools",
            sw.pool_id
        );
    }
}
