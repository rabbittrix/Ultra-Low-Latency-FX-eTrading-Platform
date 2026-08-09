//! Deterministic synthetic FX tick series (no OS entropy).

use fx_smc_common::{InstrumentMeta, Px, Qty, Side, SynthConfig, Tick, TsNanos};

/// Parameters for [`generate_ticks`].
#[derive(Debug, Clone)]
pub struct SynthParams {
    /// Instrument metadata.
    pub meta: InstrumentMeta,
    /// PRNG seed.
    pub seed: u64,
    /// Number of ticks.
    pub tick_count: usize,
    /// Starting mid in ticks.
    pub base_mid_ticks: i64,
    /// Max absolute walk step in ticks.
    pub walk_ticks: i64,
    /// Inject sweep every N ticks (`0` = off).
    pub sweep_every: usize,
    /// Sweep excursion in ticks.
    pub sweep_break_ticks: i64,
    /// Starting timestamp (nanos).
    pub start_ts_ns: i64,
    /// Nanoseconds between ticks.
    pub step_ns: i64,
    /// Top-of-book size in qty ticks.
    pub book_qty: i64,
    /// Half-spread in ticks (bid = mid - half, ask = mid + half).
    pub half_spread_ticks: i64,
}

impl SynthParams {
    /// Build from TOML [`SynthConfig`] and instrument meta.
    #[must_use]
    pub fn from_config(cfg: &SynthConfig, meta: &InstrumentMeta) -> Self {
        Self {
            meta: meta.clone(),
            seed: cfg.seed,
            tick_count: cfg.tick_count,
            base_mid_ticks: cfg.base_mid_ticks,
            walk_ticks: cfg.walk_ticks.max(1),
            sweep_every: cfg.sweep_every,
            sweep_break_ticks: cfg.sweep_break_ticks.max(1),
            start_ts_ns: 1_700_000_000_000_000_000,
            step_ns: 1_000_000, // 1 ms
            book_qty: 1_000_000,
            half_spread_ticks: meta.tick_size.max(1),
        }
    }
}

/// Tiny deterministic LCG (Numerical Recipes constants). No `std::collections` growth beyond `Vec` reserve.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0
    }

    fn next_i64_range(&mut self, max_abs: i64) -> i64 {
        let max_abs = max_abs.clamp(0, 1_000_000);
        let width_i = max_abs.saturating_mul(2).saturating_add(1).max(1);
        let Ok(width) = u64::try_from(width_i) else {
            return 0;
        };
        let bucket = self.next_u64() % width;
        match i64::try_from(bucket) {
            Ok(v) => v - max_abs,
            Err(_) => 0,
        }
    }
}

/// Generate a deterministic tick series.
///
/// Allocates exactly once for the output `Vec` (cold-path / test harness).
#[must_use]
pub fn generate_ticks(params: &SynthParams) -> Vec<Tick> {
    let mut out = Vec::with_capacity(params.tick_count);
    let mut rng = Lcg(params.seed | 1);
    let mut mid = params.base_mid_ticks;
    let half = params.half_spread_ticks.max(1);
    let mut ts = params.start_ts_ns;

    for i in 0..params.tick_count {
        let walk = rng.next_i64_range(params.walk_ticks);
        mid = mid.saturating_add(walk);

        let mut bid = mid.saturating_sub(half);
        let mut ask = mid.saturating_add(half);
        let mut aggressor = None;

        if params.sweep_every > 0 && i > 0 && i % params.sweep_every == 0 {
            // Downside sweep then partial recovery — tests later sweep detectors.
            let break_sz = params.sweep_break_ticks;
            bid = bid.saturating_sub(break_sz);
            ask = bid.saturating_add(half * 2);
            mid = bid.saturating_add(half);
            aggressor = Some(Side::Sell);
        }

        out.push(Tick {
            symbol: params.meta.symbol.clone(),
            ts_ns: TsNanos(ts),
            bid: Px(bid),
            ask: Px(ask),
            bid_qty: Qty(params.book_qty),
            ask_qty: Qty(params.book_qty),
            aggressor,
        });
        ts = ts.saturating_add(params.step_ns);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{AppConfig, SymbolId};

    #[test]
    fn deterministic_same_seed() {
        let cfg =
            AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config");
        let meta = cfg.instrument.default.to_meta();
        let p = SynthParams::from_config(&cfg.synth, &meta);
        let a = generate_ticks(&p);
        let b = generate_ticks(&p);
        assert_eq!(a, b);
        assert_eq!(a.len(), cfg.synth.tick_count);
        assert_eq!(a[0].symbol, SymbolId::new("EURUSD"));
    }

    #[test]
    fn injects_sweep_aggressor() {
        let cfg =
            AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config");
        let meta = cfg.instrument.default.to_meta();
        let mut p = SynthParams::from_config(&cfg.synth, &meta);
        p.tick_count = 1_000;
        p.sweep_every = 100;
        let ticks = generate_ticks(&p);
        let sweeps = ticks
            .iter()
            .filter(|t| t.aggressor == Some(Side::Sell))
            .count();
        assert!(sweeps >= 9);
    }
}
