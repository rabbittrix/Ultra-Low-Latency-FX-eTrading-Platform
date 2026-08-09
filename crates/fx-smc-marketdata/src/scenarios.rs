//! Named deterministic synthetic scenarios for research / detector harnesses.

use fx_smc_common::{InstrumentMeta, Px, Qty, Side, Tick, TsNanos};

/// Named synthetic market paths (deterministic from `seed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SynthScenario {
    /// Flat Asian-style range, then downside pierce + reclaim (London-style sweep).
    AsianRangeLondonSweep,
    /// Repeated pierces of the same level without sustained reclaim (fake sweeps).
    FakeoutChop,
    /// Rising HH/HL pattern with small gaps (FVG-friendly structure).
    CleanTrendBos,
}

/// Tiny deterministic LCG (Numerical Recipes constants).
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

fn push_tick(
    out: &mut Vec<Tick>,
    meta: &InstrumentMeta,
    ts: i64,
    mid: i64,
    half: i64,
    book_qty: i64,
    aggressor: Option<Side>,
) {
    let bid = mid.saturating_sub(half);
    let ask = mid.saturating_add(half);
    out.push(Tick {
        symbol: meta.symbol.clone(),
        ts_ns: TsNanos(ts),
        bid: Px(bid),
        ask: Px(ask),
        bid_qty: Qty(book_qty),
        ask_qty: Qty(book_qty),
        aggressor,
    });
}

/// Generate a named scenario tick series (deterministic for `(scenario, meta, seed)`).
#[must_use]
pub fn generate_scenario(scenario: SynthScenario, meta: &InstrumentMeta, seed: u64) -> Vec<Tick> {
    match scenario {
        SynthScenario::AsianRangeLondonSweep => asian_range_london_sweep(meta, seed),
        SynthScenario::FakeoutChop => fakeout_chop(meta, seed),
        SynthScenario::CleanTrendBos => clean_trend_bos(meta, seed),
    }
}

/// ~80-tick flat range, then downside pierce below range low + reclaim (Sell then Buy).
fn asian_range_london_sweep(meta: &InstrumentMeta, seed: u64) -> Vec<Tick> {
    let mut rng = Lcg(seed | 1);
    let half = meta.tick_size.max(1);
    let book_qty = 1_000_000_i64;
    let step_ns = 1_000_000_i64;
    let mut ts = 1_700_000_000_000_000_000_i64;
    let base = 11_000_i64;
    let range_half = 3_i64;
    let range_low = base.saturating_sub(range_half);
    let mut out = Vec::with_capacity(100);

    // Asian range: ~80 ticks oscillating inside [base-range_half, base+range_half].
    for _ in 0..80 {
        let wobble = rng.next_i64_range(range_half);
        let mid = base.saturating_add(wobble).clamp(
            range_low.saturating_add(half),
            base.saturating_add(range_half),
        );
        push_tick(&mut out, meta, ts, mid, half, book_qty, None);
        ts = ts.saturating_add(step_ns);
    }

    // Downside pierce below range low (aggressor Sell).
    let pierce_mid = range_low.saturating_sub(6);
    push_tick(
        &mut out,
        meta,
        ts,
        pierce_mid,
        half,
        book_qty,
        Some(Side::Sell),
    );
    ts = ts.saturating_add(step_ns);

    // Reclaim back into range (aggressor Buy).
    let reclaim_mid = base;
    push_tick(
        &mut out,
        meta,
        ts,
        reclaim_mid,
        half,
        book_qty,
        Some(Side::Buy),
    );
    ts = ts.saturating_add(step_ns);

    // A few post-reclaim ticks inside the prior range.
    for _ in 0..8 {
        let wobble = rng.next_i64_range(range_half);
        let mid = base.saturating_add(wobble);
        push_tick(&mut out, meta, ts, mid, half, book_qty, None);
        ts = ts.saturating_add(step_ns);
    }
    out
}

/// Repeated pierces of the same level without sustained reclaim.
fn fakeout_chop(meta: &InstrumentMeta, seed: u64) -> Vec<Tick> {
    let mut rng = Lcg(seed.wrapping_add(0x9e37_79b9) | 1);
    let half = meta.tick_size.max(1);
    let book_qty = 1_000_000_i64;
    let step_ns = 1_000_000_i64;
    let mut ts = 1_700_000_000_000_000_000_i64;
    let level = 11_000_i64;
    let mut out = Vec::with_capacity(120);

    for _ in 0..20 {
        let wobble = rng.next_i64_range(2);
        push_tick(
            &mut out,
            meta,
            ts,
            level.saturating_add(wobble),
            half,
            book_qty,
            None,
        );
        ts = ts.saturating_add(step_ns);
    }

    // Multiple fake sweeps: pierce then fail to hold reclaim.
    for cycle in 0..5 {
        let pierce = level.saturating_sub(5 + i64::from(cycle % 2));
        push_tick(&mut out, meta, ts, pierce, half, book_qty, Some(Side::Sell));
        ts = ts.saturating_add(step_ns);
        // Weak bounce that does not sustain above level.
        let bounce = level.saturating_sub(1);
        push_tick(&mut out, meta, ts, bounce, half, book_qty, Some(Side::Buy));
        ts = ts.saturating_add(step_ns);
        for _ in 0..4 {
            let wobble = rng.next_i64_range(2);
            push_tick(
                &mut out,
                meta,
                ts,
                level.saturating_add(wobble).saturating_sub(2),
                half,
                book_qty,
                None,
            );
            ts = ts.saturating_add(step_ns);
        }
    }
    out
}

/// Rising HH/HL pattern with small gaps for FVG-friendly structure.
fn clean_trend_bos(meta: &InstrumentMeta, seed: u64) -> Vec<Tick> {
    let mut rng = Lcg(seed.wrapping_mul(0x85eb_ca6b) | 1);
    let half = meta.tick_size.max(1);
    let book_qty = 1_000_000_i64;
    let step_ns = 1_000_000_i64;
    let mut ts = 1_700_000_000_000_000_000_i64;
    let mut mid = 11_000_i64;
    let mut out = Vec::with_capacity(100);

    // Impulsive legs with small gaps, then shallow HL pullbacks.
    for leg in 0..6 {
        // Impulse up with a 2–3 tick gap (FVG-friendly).
        let gap = 2 + i64::from(leg % 2);
        mid = mid.saturating_add(gap);
        push_tick(&mut out, meta, ts, mid, half, book_qty, Some(Side::Buy));
        ts = ts.saturating_add(step_ns);
        for _ in 0..6 {
            let step = 1 + rng.next_i64_range(1).abs();
            mid = mid.saturating_add(step);
            push_tick(&mut out, meta, ts, mid, half, book_qty, None);
            ts = ts.saturating_add(step_ns);
        }
        // Higher-low pullback (shallow — leave room above prior impulse base).
        let pull = 2 + rng.next_i64_range(1).abs().min(2);
        for _ in 0..4 {
            mid = mid.saturating_sub(1).max(mid.saturating_sub(pull));
            push_tick(&mut out, meta, ts, mid, half, book_qty, None);
            ts = ts.saturating_add(step_ns);
        }
        mid = mid.saturating_add(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;

    fn meta() -> InstrumentMeta {
        let cfg =
            AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config");
        cfg.instrument.default.to_meta()
    }

    #[test]
    fn same_seed_same_ticks() {
        let m = meta();
        for scenario in [
            SynthScenario::AsianRangeLondonSweep,
            SynthScenario::FakeoutChop,
            SynthScenario::CleanTrendBos,
        ] {
            let a = generate_scenario(scenario, &m, 42);
            let b = generate_scenario(scenario, &m, 42);
            assert_eq!(a, b);
            assert!(!a.is_empty());
        }
    }

    #[test]
    fn asian_has_clear_low_excursion() {
        let m = meta();
        let ticks = generate_scenario(SynthScenario::AsianRangeLondonSweep, &m, 7);
        assert!(ticks.len() >= 80);
        let range_slice = &ticks[..80];
        let range_low = range_slice
            .iter()
            .map(|t| t.bid.0.min(t.mid_ticks().0))
            .min()
            .expect("range");
        let pierce = &ticks[80];
        assert!(
            pierce.mid_ticks().0 < range_low,
            "pierce mid {} must be below asian range low {range_low}",
            pierce.mid_ticks().0
        );
        assert_eq!(pierce.aggressor, Some(Side::Sell));
        assert_eq!(ticks[81].aggressor, Some(Side::Buy));
    }
}
