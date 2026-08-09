//! Regime classification from mid drift + ATR proxy.

use fx_smc_common::{AdvisoryConfig, Tick};
use fx_smc_structure::atr_proxy_ticks;
use serde::{Deserialize, Serialize};

/// Coarse market regime for a tick window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Regime {
    /// Net mid drift ≥ `trend_drift_ticks`.
    TrendUp,
    /// Net mid drift ≤ `-trend_drift_ticks`.
    TrendDown,
    /// Low drift, ATR below volatile threshold.
    Range,
    /// ATR proxy ≥ `volatile_atr_ticks`.
    Volatile,
}

/// Classify regime from the last `window_ticks` (or full slice if shorter).
#[must_use]
pub fn classify_regime(ticks: &[Tick], cfg: &AdvisoryConfig) -> Regime {
    let window = take_window(ticks, cfg.window_ticks);
    if window.len() < 2 {
        return Regime::Range;
    }
    let atr = atr_proxy_ticks(window, cfg.window_ticks.max(1));
    if atr >= cfg.volatile_atr_ticks.max(0) {
        return Regime::Volatile;
    }
    let first = window[0].mid_ticks().0;
    let last = window[window.len() - 1].mid_ticks().0;
    let drift = last.saturating_sub(first);
    let thr = cfg.trend_drift_ticks.max(0);
    if drift >= thr {
        Regime::TrendUp
    } else if drift <= -thr {
        Regime::TrendDown
    } else {
        Regime::Range
    }
}

pub(crate) fn take_window(ticks: &[Tick], window_ticks: usize) -> &[Tick] {
    let n = window_ticks.max(1);
    if ticks.len() <= n {
        ticks
    } else {
        let start = ticks.len() - n;
        &ticks[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{AppConfig, Px, Qty, SymbolId, TsNanos};

    fn tick(mid: i64, ts: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(ts),
            bid: Px(mid),
            ask: Px(mid + 1),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }
    }

    #[test]
    fn trend_up_and_volatile() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let up: Vec<_> = (0..50).map(|i| tick(100 + i, i)).collect();
        assert_eq!(classify_regime(&up, &cfg.advisory), Regime::TrendUp);

        let down: Vec<_> = (0..50).map(|i| tick(200 - i, i)).collect();
        assert_eq!(classify_regime(&down, &cfg.advisory), Regime::TrendDown);

        // Large steps → high ATR → Volatile (threshold 40 in defaults)
        let choppy: Vec<_> = (0..50)
            .map(|i| tick(100 + if i % 2 == 0 { 80 } else { 0 }, i))
            .collect();
        assert_eq!(classify_regime(&choppy, &cfg.advisory), Regime::Volatile);
    }
}
