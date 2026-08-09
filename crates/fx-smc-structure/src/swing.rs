//! Swing high / low detection (fractal pivots).

use fx_smc_common::{Px, SwingConfig, Tick, TsNanos};
use serde::{Deserialize, Serialize};

/// Swing pivot side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwingKind {
    /// Local high.
    High,
    /// Local low.
    Low,
}

/// Confirmed swing point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwingPoint {
    /// High or low.
    pub kind: SwingKind,
    /// Pivot price (mid ticks at confirmation).
    pub price: Px,
    /// Timestamp of the pivot bar.
    pub ts_ns: TsNanos,
    /// Index into the source tick slice.
    pub index: usize,
    /// Configured left strength used.
    pub strength: usize,
}

/// Detect swing highs and lows on mid prices.
///
/// A bar `i` is a high if it is strictly greater than `left` neighbors and
/// greater-or-equal than `right` neighbors (and symmetrically for lows).
#[must_use]
pub fn detect_swings(ticks: &[Tick], cfg: &SwingConfig) -> Vec<SwingPoint> {
    let left = cfg.left_strength.max(1);
    let right = cfg.right_strength.max(1);
    let strength = left.max(right);
    let mut out = Vec::new();
    if ticks.len() < left + right + 1 {
        return out;
    }
    let last = ticks.len() - right;
    for i in left..last {
        let m = ticks[i].mid_ticks().0;
        let mut is_high = true;
        let mut is_low = true;
        for j in 1..=left {
            let l = ticks[i - j].mid_ticks().0;
            if m <= l {
                is_high = false;
            }
            if m >= l {
                is_low = false;
            }
        }
        for j in 1..=right {
            let r = ticks[i + j].mid_ticks().0;
            if m < r {
                is_high = false;
            }
            if m > r {
                is_low = false;
            }
        }
        if is_high {
            out.push(SwingPoint {
                kind: SwingKind::High,
                price: Px(m),
                ts_ns: ticks[i].ts_ns,
                index: i,
                strength,
            });
        } else if is_low {
            out.push(SwingPoint {
                kind: SwingKind::Low,
                price: Px(m),
                ts_ns: ticks[i].ts_ns,
                index: i,
                strength,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{Qty, SymbolId};

    fn series(mids: &[i64]) -> Vec<Tick> {
        mids.iter()
            .enumerate()
            .map(|(i, m)| Tick {
                symbol: SymbolId::new("EURUSD"),
                ts_ns: TsNanos(i64::try_from(i).unwrap_or(0) * 1_000_000),
                bid: Px(*m),
                ask: Px(*m + 1),
                bid_qty: Qty(1),
                ask_qty: Qty(1),
                aggressor: None,
            })
            .collect()
    }

    #[test]
    fn finds_clear_high_and_low() {
        // left=1,right=1: peak at 5, trough at 1
        let ticks = series(&[1, 2, 5, 2, 1, 3, 0, 2]);
        let cfg = SwingConfig {
            left_strength: 1,
            right_strength: 1,
        };
        let swings = detect_swings(&ticks, &cfg);
        assert!(swings
            .iter()
            .any(|s| s.kind == SwingKind::High && s.price == Px(5)));
        assert!(swings
            .iter()
            .any(|s| s.kind == SwingKind::Low && s.price == Px(0)));
    }

    #[test]
    fn swings_are_time_ordered() {
        let ticks = series(&[1, 3, 1, 4, 1, 5, 1]);
        let cfg = SwingConfig {
            left_strength: 1,
            right_strength: 1,
        };
        let swings = detect_swings(&ticks, &cfg);
        for w in swings.windows(2) {
            assert!(w[0].ts_ns <= w[1].ts_ns);
            assert!(w[0].index < w[1].index);
        }
    }
}
