//! Deterministic fill simulation (Xoshiro jitter, limit exits, sweep-proximate spread).

use fx_smc_common::{BacktestConfig, Px, Tick};
use fx_smc_strategy::{TradePlan, TradeSide};
use serde::{Deserialize, Serialize};

/// In-crate Xoshiro256++ (wrapping ops only — no external RNG crate).
#[derive(Debug, Clone)]
pub struct Xoshiro256PlusPlus {
    s: [u64; 4],
}

impl Xoshiro256PlusPlus {
    /// Seed from a single `u64` (splits via SplitMix64-style expansion).
    #[must_use]
    pub fn seed(seed: u64) -> Self {
        let mut sm = seed | 1;
        let mut s = [0u64; 4];
        for slot in &mut s {
            sm = sm.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            *slot = z ^ (z >> 31);
        }
        // Ensure non-zero state.
        if s.iter().all(|&x| x == 0) {
            s[0] = 0xdead_beef_cafe_babe;
        }
        Self { s }
    }

    /// Next `u64`.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[0]
            .wrapping_add(self.s[3])
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Signed jitter in `[-max_abs, max_abs]`.
    pub fn next_i64_range(&mut self, max_abs: i64) -> i64 {
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

/// Seed RNG from confirm timestamp (native-endian) XOR plan-id hash.
#[must_use]
pub fn seed_from_plan(plan: &TradePlan) -> u64 {
    let ts = u64::from_ne_bytes(plan.as_of_ns.0.to_ne_bytes());
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in plan.id.as_bytes() {
        h = h.wrapping_mul(0x0100_0000_01b3).wrapping_add(u64::from(*b));
    }
    let idx = u64::try_from(plan.as_of_idx).unwrap_or(0);
    ts ^ h ^ idx
}

/// Local mid range over lookback ending at `idx` (inclusive).
fn local_range_ticks(ticks: &[Tick], idx: usize, lookback: usize) -> i64 {
    if ticks.is_empty() || lookback == 0 {
        return 0;
    }
    let end = idx.min(ticks.len().saturating_sub(1));
    let start = end.saturating_sub(lookback.saturating_sub(1));
    let mut lo = i64::MAX;
    let mut hi = i64::MIN;
    for t in &ticks[start..=end] {
        let m = t.mid_ticks().0;
        lo = lo.min(m);
        hi = hi.max(m);
    }
    if lo == i64::MAX {
        0
    } else {
        hi.saturating_sub(lo)
    }
}

fn effective_spread_ticks(cfg: &BacktestConfig, sweep_proximate: bool) -> i64 {
    let base = cfg.spread_ticks.max(0);
    if !sweep_proximate {
        return base;
    }
    let den = cfg.fill_sweep_spread_mult_den.max(1);
    base.saturating_mul(cfg.fill_sweep_spread_mult_num.max(0)) / den
}

fn is_sweep_proximate(plan: &TradePlan, mid: i64, cfg: &BacktestConfig) -> bool {
    let prox = cfg.fill_sweep_proximity_ticks.max(0);
    (mid - plan.stop.0).abs() <= prox
}

/// Market-entry adverse ticks: base + vol factor + Xoshiro jitter, with optional spread widen.
#[must_use]
pub fn market_entry_adverse(
    ticks: &[Tick],
    plan: &TradePlan,
    cfg: &BacktestConfig,
    rng: &mut Xoshiro256PlusPlus,
) -> i64 {
    let range = local_range_ticks(ticks, plan.as_of_idx, cfg.fill_vol_lookback.max(1));
    let den = cfg.fill_vol_slippage_den.max(1);
    let vol = range.saturating_mul(cfg.fill_vol_slippage_num.max(0)) / den;
    let jitter = rng.next_i64_range(cfg.fill_jitter_max_ticks.max(0)).abs();
    let mid = ticks
        .get(plan.as_of_idx)
        .map_or(plan.entry.0, |t| t.mid_ticks().0);
    let spread = effective_spread_ticks(cfg, is_sweep_proximate(plan, mid, cfg));
    let half_spread = spread / 2;
    cfg.fill_base_slippage_ticks
        .max(0)
        .max(cfg.slippage_ticks_per_side.max(0))
        .saturating_add(vol)
        .saturating_add(jitter)
        .saturating_add(half_spread)
}

/// Limit-exit fill when mid crosses target/stop; adverse = effective `spread/2`.
#[must_use]
pub fn limit_exit_fill(
    plan: &TradePlan,
    raw_exit: Px,
    mid_at_exit: i64,
    cfg: &BacktestConfig,
    hit_stop: bool,
) -> Px {
    let spread =
        effective_spread_ticks(cfg, is_sweep_proximate(plan, mid_at_exit, cfg) || hit_stop);
    let half = spread / 2;
    match plan.side {
        TradeSide::Long => Px(raw_exit.0.saturating_sub(half)),
        TradeSide::Short => Px(raw_exit.0.saturating_add(half)),
    }
}

/// One simulated fill leg for `PnL` curve hashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FillRecord {
    /// Trade plan id.
    pub plan_id: String,
    /// Exit tick index.
    pub exit_idx: usize,
    /// Net `PnL` in ticks after fills / commission for this plan.
    pub pnl_ticks: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xoshiro_deterministic() {
        let mut a = Xoshiro256PlusPlus::seed(42);
        let mut b = Xoshiro256PlusPlus::seed(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
