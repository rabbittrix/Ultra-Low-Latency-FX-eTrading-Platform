//! Break of structure (BOS) and change of character (`CHoCH`) from swings.

use crate::swing::{SwingKind, SwingPoint};
use fx_smc_common::{Px, TsNanos};
use serde::{Deserialize, Serialize};

/// Bullish vs bearish structure event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureBias {
    /// Upside break / bullish character.
    Bullish,
    /// Downside break / bearish character.
    Bearish,
}

/// BOS vs `CHoCH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureBreakKind {
    /// Continuation break with the prior trend.
    Bos,
    /// First break against the prior trend (change of character).
    ChoCh,
}

/// Detected structure break.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureBreak {
    /// BOS or `CHoCH`.
    pub kind: StructureBreakKind,
    /// Direction of the break.
    pub bias: StructureBias,
    /// Broken level (swing price).
    pub level: Px,
    /// Time of the confirming swing.
    pub ts_ns: TsNanos,
    /// Index of confirming swing in the input slice.
    pub swing_index: usize,
}

#[derive(Clone, Copy)]
enum Trend {
    Up,
    Down,
    Flat,
}

/// Detect BOS/CHoCH by walking confirmed swings in time order.
///
/// Trend starts flat. Breaking with trend ⇒ BOS; first break against trend ⇒ `CHoCH`.
#[must_use]
pub fn detect_structure_breaks(swings: &[SwingPoint]) -> Vec<StructureBreak> {
    let mut out = Vec::new();
    if swings.len() < 2 {
        return out;
    }

    let mut trend = Trend::Flat;
    let mut last_high: Option<&SwingPoint> = None;
    let mut last_low: Option<&SwingPoint> = None;

    for (i, s) in swings.iter().enumerate() {
        match s.kind {
            SwingKind::High => {
                if let Some(prev) = last_high {
                    if s.price.0 > prev.price.0 {
                        let kind = match trend {
                            Trend::Up | Trend::Flat => StructureBreakKind::Bos,
                            Trend::Down => StructureBreakKind::ChoCh,
                        };
                        out.push(StructureBreak {
                            kind,
                            bias: StructureBias::Bullish,
                            level: prev.price,
                            ts_ns: s.ts_ns,
                            swing_index: i,
                        });
                        trend = Trend::Up;
                    }
                }
                last_high = Some(s);
            }
            SwingKind::Low => {
                if let Some(prev) = last_low {
                    if s.price.0 < prev.price.0 {
                        let kind = match trend {
                            Trend::Down | Trend::Flat => StructureBreakKind::Bos,
                            Trend::Up => StructureBreakKind::ChoCh,
                        };
                        out.push(StructureBreak {
                            kind,
                            bias: StructureBias::Bearish,
                            level: prev.price,
                            ts_ns: s.ts_ns,
                            swing_index: i,
                        });
                        trend = Trend::Down;
                    }
                }
                last_low = Some(s);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swing::{SwingKind, SwingPoint};
    use fx_smc_common::TsNanos;

    fn sw(kind: SwingKind, px: i64, i: usize) -> SwingPoint {
        SwingPoint {
            kind,
            price: Px(px),
            ts_ns: TsNanos(i64::try_from(i).unwrap_or(0)),
            index: i,
            strength: 2,
        }
    }

    #[test]
    fn uptrend_bos_then_choch() {
        let swings = vec![
            sw(SwingKind::Low, 100, 0),
            sw(SwingKind::High, 110, 1),
            sw(SwingKind::Low, 105, 2),
            sw(SwingKind::High, 120, 3),
            sw(SwingKind::Low, 102, 4),
        ];
        let b = detect_structure_breaks(&swings);
        assert!(b.iter().any(|x| x.kind == StructureBreakKind::Bos));
        assert!(b.iter().any(|x| x.kind == StructureBreakKind::ChoCh));
    }
}
