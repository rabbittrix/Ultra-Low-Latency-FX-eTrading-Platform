//! Window scoring and symbol ranking.

use crate::regime::{classify_regime, take_window, Regime};
use fx_smc_common::{AdvisoryConfig, SymbolId, Tick};
use fx_smc_structure::atr_proxy_ticks;
use serde::{Deserialize, Serialize};

/// Score for one symbol window (fixed-point).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowScore {
    /// Score in `0..=score_scale` (higher = more directional / structured for ranking).
    pub score: i64,
    /// Classified regime.
    pub regime: Regime,
    /// Window length used (ticks).
    pub window_ticks: usize,
}

/// Score the trailing window of `ticks`.
#[must_use]
pub fn score_window(ticks: &[Tick], cfg: &AdvisoryConfig) -> WindowScore {
    let window = take_window(ticks, cfg.window_ticks);
    let regime = classify_regime(ticks, cfg);
    let scale = cfg.score_scale.max(1);
    let atr = atr_proxy_ticks(window, cfg.window_ticks.max(1));
    let drift = if window.len() < 2 {
        0
    } else {
        window[window.len() - 1]
            .mid_ticks()
            .0
            .saturating_sub(window[0].mid_ticks().0)
            .abs()
    };
    // Prefer clear trend with moderate ATR: score ~ scale * min(drift, scale) heuristics in ticks.
    let score = match regime {
        Regime::TrendUp | Regime::TrendDown => {
            // Strong directional windows rank near the top of the scale.
            let strength = drift.min(cfg.trend_drift_ticks.saturating_mul(4).max(1));
            let base =
                strength.saturating_mul(scale) / cfg.trend_drift_ticks.saturating_mul(4).max(1);
            // Floor so clear trends outrank quiet ranges.
            (base.saturating_add(scale / 2)).clamp(0, scale)
        }
        Regime::Range => {
            let quiet = cfg.volatile_atr_ticks.max(1).saturating_sub(atr);
            let raw = quiet.saturating_mul(scale / 2) / cfg.volatile_atr_ticks.max(1);
            raw.clamp(0, scale / 2)
        }
        Regime::Volatile => {
            // Lower rank for chaotic windows; still non-zero for observability.
            (scale / 10).clamp(0, scale)
        }
    };
    WindowScore {
        score,
        regime,
        window_ticks: window.len(),
    }
}

/// Rank symbols by [`WindowScore::score`] descending (stable by symbol on ties).
#[must_use]
pub fn rank_symbols(
    series: &[(SymbolId, &[Tick])],
    cfg: &AdvisoryConfig,
) -> Vec<(SymbolId, WindowScore)> {
    let mut out: Vec<(SymbolId, WindowScore)> = series
        .iter()
        .map(|(sym, ticks)| (sym.clone(), score_window(ticks, cfg)))
        .collect();
    out.sort_by(|a, b| {
        b.1.score
            .cmp(&a.1.score)
            .then_with(|| a.0.as_str().cmp(b.0.as_str()))
    });
    out
}

/// Alias for [`rank_symbols`] (window scores over symbol series).
#[must_use]
pub fn rank_windows(
    series: &[(SymbolId, &[Tick])],
    cfg: &AdvisoryConfig,
) -> Vec<(SymbolId, WindowScore)> {
    rank_symbols(series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{AppConfig, Px, Qty, TsNanos};

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
    fn ranks_trend_above_flat() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let trend: Vec<_> = (0..80).map(|i| tick(100 + i, i)).collect();
        let flat: Vec<_> = (0..80).map(|i| tick(100, i)).collect();
        let ranked = rank_symbols(
            &[
                (SymbolId::new("FLAT"), flat.as_slice()),
                (SymbolId::new("TREND"), trend.as_slice()),
            ],
            &cfg.advisory,
        );
        assert_eq!(ranked[0].0.as_str(), "TREND");
        assert!(ranked[0].1.score >= ranked[1].1.score);
    }
}
