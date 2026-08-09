//! Fair value gaps (three-candle imbalances) on mid prices.

use fx_smc_common::{FvgConfig, Px, Tick, TsNanos};
use serde::{Deserialize, Serialize};

/// Bullish (demand) vs bearish (supply) gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FvgSide {
    /// Upside imbalance (candle i low above candle i-2 high).
    Bullish,
    /// Downside imbalance.
    Bearish,
}

/// Detected fair value gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FairValueGap {
    /// Gap side.
    pub side: FvgSide,
    /// Upper bound of the gap (ticks).
    pub top: Px,
    /// Lower bound of the gap (ticks).
    pub bottom: Px,
    /// Timestamp of the third candle.
    pub ts_ns: TsNanos,
    /// Index of the third candle in the tick slice.
    pub index: usize,
}

/// Detect FVGs: for each `i >= 2`, compare candle `i-2` and `i` mids±half-range via bid/ask extremes.
///
/// Bullish: `high(i-2) < low(i)` and gap ≥ `min_gap_ticks`.
/// Bearish: `low(i-2) > high(i)` and gap ≥ `min_gap_ticks`.
#[must_use]
pub fn detect_fvgs(ticks: &[Tick], cfg: &FvgConfig) -> Vec<FairValueGap> {
    let min_gap = cfg.min_gap_ticks.max(1);
    let mut out = Vec::new();
    if ticks.len() < 3 {
        return out;
    }
    for i in 2..ticks.len() {
        let a = &ticks[i - 2];
        let c = &ticks[i];
        // Use ask as high proxy, bid as low proxy (TOB).
        let high_a = a.ask.0;
        let low_a = a.bid.0;
        let high_c = c.ask.0;
        let low_c = c.bid.0;

        if low_c > high_a {
            let gap = low_c.saturating_sub(high_a);
            if gap >= min_gap {
                out.push(FairValueGap {
                    side: FvgSide::Bullish,
                    top: Px(low_c),
                    bottom: Px(high_a),
                    ts_ns: c.ts_ns,
                    index: i,
                });
            }
        } else if high_c < low_a {
            let gap = low_a.saturating_sub(high_c);
            if gap >= min_gap {
                out.push(FairValueGap {
                    side: FvgSide::Bearish,
                    top: Px(low_a),
                    bottom: Px(high_c),
                    ts_ns: c.ts_ns,
                    index: i,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{Qty, SymbolId, TsNanos};

    fn tick(bid: i64, ask: i64, i: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(i),
            bid: Px(bid),
            ask: Px(ask),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }
    }

    #[test]
    fn detects_bullish_gap() {
        let ticks = vec![
            tick(100, 101, 0),
            tick(100, 101, 1),
            tick(110, 111, 2), // gap above 101
        ];
        let cfg = FvgConfig { min_gap_ticks: 2 };
        let g = detect_fvgs(&ticks, &cfg);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].side, FvgSide::Bullish);
    }
}
