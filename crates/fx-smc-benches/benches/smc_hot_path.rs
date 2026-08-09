//! Criterion benches for SMC scoring / hashing / sweeps (M10).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fx_smc_common::{AppConfig, EventHasher};
use fx_smc_liquidity::{map_from_ticks, score_pool, PoolOrigin, PoolScoreInput};
use fx_smc_marketdata::{generate_ticks, SynthParams};
use fx_smc_sweep::detect_sweeps;

fn load_cfg() -> AppConfig {
    AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config")
}

fn bench_event_hash(c: &mut Criterion) {
    let cfg = load_cfg();
    let meta = cfg.instrument.default.to_meta();
    let mut p = SynthParams::from_config(&cfg.synth, &meta);
    p.tick_count = 2_000;
    let ticks = generate_ticks(&p);
    c.bench_function("event_hash_2k", |b| {
        b.iter(|| {
            let mut h = EventHasher::new();
            for t in &ticks {
                h.absorb_tick(t);
            }
            black_box(h.finalize())
        });
    });
}

fn bench_liquidity_map(c: &mut Criterion) {
    let cfg = load_cfg();
    let meta = cfg.instrument.default.to_meta();
    let mut p = SynthParams::from_config(&cfg.synth, &meta);
    p.tick_count = 2_000;
    let ticks = generate_ticks(&p);
    c.bench_function("map_liquidity_2k", |b| {
        b.iter(|| {
            black_box(map_from_ticks(
                &ticks,
                &cfg.structure,
                &cfg.liquidity,
                &cfg.liquidity_score,
            ))
        });
    });
}

fn bench_score_pool(c: &mut Criterion) {
    let cfg = load_cfg();
    c.bench_function("score_pool", |b| {
        b.iter(|| {
            black_box(score_pool(
                &PoolScoreInput {
                    price: fx_smc_common::Px(11_000),
                    touches: 4,
                    last_touch_ns: fx_smc_common::TsNanos(1_000),
                    origin: PoolOrigin::Equal,
                    mid: fx_smc_common::Px(11_050),
                    now_ns: fx_smc_common::TsNanos(2_000),
                    equality_std_ticks: Some(0),
                    atr_ticks: 10,
                },
                &cfg.liquidity_score,
            ))
        });
    });
}

fn bench_detect_sweeps(c: &mut Criterion) {
    let cfg = load_cfg();
    let meta = cfg.instrument.default.to_meta();
    let mut p = SynthParams::from_config(&cfg.synth, &meta);
    p.tick_count = 2_000;
    let ticks = generate_ticks(&p);
    let pools = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
    c.bench_function("detect_sweeps_2k", |b| {
        b.iter(|| black_box(detect_sweeps(&ticks, &pools, &cfg.sweep)));
    });
}

criterion_group!(
    benches,
    bench_event_hash,
    bench_score_pool,
    bench_liquidity_map,
    bench_detect_sweeps
);
criterion_main!(benches);
