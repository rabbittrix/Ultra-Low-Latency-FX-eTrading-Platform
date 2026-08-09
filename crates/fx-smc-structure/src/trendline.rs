//! Trendline liquidity (≥ min touches) with rational slope.

use crate::geom::{abs_ticks, project_price};
use crate::swing::{SwingKind, SwingPoint};
use fx_smc_common::{Px, TrendlineConfig, TsNanos};
use serde::{Deserialize, Serialize};

/// Resistance (highs) or support (lows) trendline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrendlineSide {
    /// Connecting swing highs.
    Resistance,
    /// Connecting swing lows.
    Support,
}

/// A trendline with rational slope `(dp_ticks / dt_ns)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trendline {
    /// Resistance or support.
    pub side: TrendlineSide,
    /// First anchor price.
    pub p0: Px,
    /// First anchor time.
    pub t0_ns: TsNanos,
    /// Price delta between anchors (ticks).
    pub dp_ticks: i64,
    /// Time delta between anchors (nanos); never zero for valid lines.
    pub dt_ns: i64,
    /// Indices into the input swing slice that touched the line.
    pub touch_indices: Vec<usize>,
    /// Latest touch timestamp.
    pub last_touch_ns: TsNanos,
}

impl Trendline {
    /// Number of touches (≥ `min_touches` when emitted by [`detect_trendlines`]).
    #[must_use]
    pub fn touch_count(&self) -> usize {
        self.touch_indices.len()
    }
}

/// Detect trendlines by pairing early swings with later ones of the same kind.
///
/// For each ordered pair `(i, j)` with `j > i`, build a line and count same-kind swings
/// within `touch_tolerance_ticks` of the projected price. Keep lines with
/// `touch_count >= min_touches`.
#[must_use]
pub fn detect_trendlines(swings: &[SwingPoint], cfg: &TrendlineConfig) -> Vec<Trendline> {
    let min_touches = cfg.min_touches.max(2);
    let tol = cfg.touch_tolerance_ticks.max(0);
    let mut out = Vec::new();

    for i in 0..swings.len() {
        for j in (i + 1)..swings.len() {
            if swings[i].kind != swings[j].kind {
                continue;
            }
            let dt = swings[j].ts_ns.0.saturating_sub(swings[i].ts_ns.0);
            if dt == 0 {
                continue;
            }
            let dp = swings[j].price.0.saturating_sub(swings[i].price.0);
            let side = match swings[i].kind {
                SwingKind::High => TrendlineSide::Resistance,
                SwingKind::Low => TrendlineSide::Support,
            };
            let mut touches = Vec::new();
            let mut last = swings[i].ts_ns;
            for (k, s) in swings.iter().enumerate() {
                if s.kind != swings[i].kind {
                    continue;
                }
                let Some(proj) =
                    project_price(swings[i].price, swings[i].ts_ns.0, dp, dt, s.ts_ns.0)
                else {
                    continue;
                };
                if abs_ticks(proj, s.price) <= tol {
                    touches.push(k);
                    if s.ts_ns > last {
                        last = s.ts_ns;
                    }
                }
            }
            if touches.len() >= min_touches {
                out.push(Trendline {
                    side,
                    p0: swings[i].price,
                    t0_ns: swings[i].ts_ns,
                    dp_ticks: dp,
                    dt_ns: dt,
                    touch_indices: touches,
                    last_touch_ns: last,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swing::SwingKind;

    fn sw(kind: SwingKind, px: i64, ts: i64, idx: usize) -> SwingPoint {
        SwingPoint {
            kind,
            price: Px(px),
            ts_ns: TsNanos(ts),
            index: idx,
            strength: 1,
        }
    }

    #[test]
    fn three_colinear_highs_form_line() {
        let swings = vec![
            sw(SwingKind::High, 100, 0, 0),
            sw(SwingKind::High, 110, 10, 1),
            sw(SwingKind::High, 120, 20, 2),
            sw(SwingKind::Low, 50, 15, 3),
        ];
        let cfg = TrendlineConfig {
            min_touches: 2,
            touch_tolerance_ticks: 0,
        };
        let lines = detect_trendlines(&swings, &cfg);
        assert!(lines
            .iter()
            .any(|l| { l.side == TrendlineSide::Resistance && l.touch_count() >= 3 }));
    }

    #[test]
    fn rejects_zero_dt() {
        let swings = vec![sw(SwingKind::Low, 100, 5, 0), sw(SwingKind::Low, 90, 5, 1)];
        let cfg = TrendlineConfig {
            min_touches: 2,
            touch_tolerance_ticks: 5,
        };
        let lines = detect_trendlines(&swings, &cfg);
        assert!(lines.is_empty());
    }
}
