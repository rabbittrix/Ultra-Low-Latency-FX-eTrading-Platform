//! Equal highs / equal lows clustering.

use crate::geom::abs_ticks;
use crate::swing::{SwingKind, SwingPoint};
use fx_smc_common::{Px, TsNanos};
use serde::{Deserialize, Serialize};

/// Whether the cluster is equal highs or equal lows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EqualKind {
    /// Equal highs.
    Highs,
    /// Equal lows.
    Lows,
}

/// A cluster of swings within tolerance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EqualCluster {
    /// Highs or lows.
    pub kind: EqualKind,
    /// Representative price (first member).
    pub price: Px,
    /// Member swing indices into the input swing slice.
    pub members: Vec<usize>,
    /// Latest touch time among members.
    pub last_touch_ns: TsNanos,
}

/// Cluster swings of the same kind when `|p_a - p_b| <= tolerance_ticks`.
///
/// Greedy: each unassigned swing seeds a cluster; later same-kind swings join if in tolerance
/// of the seed price.
#[must_use]
pub fn cluster_equal_levels(swings: &[SwingPoint], tolerance_ticks: i64) -> Vec<EqualCluster> {
    let tol = tolerance_ticks.max(0);
    let mut used = vec![false; swings.len()];
    let mut out = Vec::new();

    for i in 0..swings.len() {
        if used[i] {
            continue;
        }
        let seed = &swings[i];
        let kind = match seed.kind {
            SwingKind::High => EqualKind::Highs,
            SwingKind::Low => EqualKind::Lows,
        };
        let mut members = vec![i];
        used[i] = true;
        let mut last = seed.ts_ns;
        for j in (i + 1)..swings.len() {
            if used[j] {
                continue;
            }
            let other = &swings[j];
            if other.kind != seed.kind {
                continue;
            }
            if abs_ticks(other.price, seed.price) <= tol {
                used[j] = true;
                members.push(j);
                if other.ts_ns > last {
                    last = other.ts_ns;
                }
            }
        }
        out.push(EqualCluster {
            kind,
            price: seed.price,
            members,
            last_touch_ns: last,
        });
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
    fn clusters_near_highs() {
        let swings = vec![
            sw(SwingKind::High, 100, 1, 0),
            sw(SwingKind::High, 101, 2, 1),
            sw(SwingKind::High, 120, 3, 2),
            sw(SwingKind::Low, 80, 4, 3),
        ];
        let clusters = cluster_equal_levels(&swings, 2);
        let eq_highs: Vec<_> = clusters
            .iter()
            .filter(|c| c.kind == EqualKind::Highs && c.members.len() >= 2)
            .collect();
        assert_eq!(eq_highs.len(), 1);
        assert_eq!(eq_highs[0].members.len(), 2);
    }

    #[test]
    fn tolerance_zero_keeps_singletons() {
        let swings = vec![sw(SwingKind::Low, 50, 1, 0), sw(SwingKind::Low, 51, 2, 1)];
        let clusters = cluster_equal_levels(&swings, 0);
        assert!(clusters.iter().all(|c| c.members.len() == 1));
    }
}
