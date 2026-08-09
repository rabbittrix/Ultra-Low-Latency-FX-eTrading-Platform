//! Fixed-point helpers shared by structure detectors (no `f64`).

use fx_smc_common::{EqualConfig, Px, Tick};

/// Mean absolute mid change over `lookback` steps (ATR proxy in ticks).
#[must_use]
pub fn atr_proxy_ticks(ticks: &[Tick], lookback: usize) -> i64 {
    if ticks.len() < 2 || lookback == 0 {
        return 0;
    }
    let start = ticks.len().saturating_sub(lookback.saturating_add(1));
    let mut sum: i64 = 0;
    let mut n: i64 = 0;
    for i in (start + 1)..ticks.len() {
        let a = ticks[i - 1].mid_ticks().0;
        let b = ticks[i].mid_ticks().0;
        sum = sum.saturating_add((b - a).abs());
        n = n.saturating_add(1);
    }
    if n == 0 {
        0
    } else {
        sum / n
    }
}

/// `max(pips_min, (k_atr_num * atr) / k_atr_den)` in ticks.
#[must_use]
pub fn equal_tolerance_ticks(cfg: &EqualConfig, atr_ticks: i64) -> i64 {
    let den = cfg.k_atr_den.max(1);
    let atr_term = cfg.k_atr_num.saturating_mul(atr_ticks.max(0)) / den;
    cfg.pips_min_ticks.max(0).max(atr_term)
}

/// Project trendline price at `t_ns`: `p0 + dp * (t - t0) / dt` using `i128`.
///
/// Returns `None` when `dt_ns == 0`.
#[must_use]
pub fn project_price(p0: Px, t0_ns: i64, dp: i64, dt_ns: i64, at_ns: i64) -> Option<Px> {
    if dt_ns == 0 {
        return None;
    }
    let num = i128::from(dp).saturating_mul(i128::from(at_ns.saturating_sub(t0_ns)));
    let den = i128::from(dt_ns);
    let delta = num / den;
    let px = i128::from(p0.0).saturating_add(delta);
    i64::try_from(px).ok().map(Px)
}

/// Absolute distance in ticks between two prices.
#[must_use]
pub fn abs_ticks(a: Px, b: Px) -> i64 {
    a.0.saturating_sub(b.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{Qty, SymbolId, TsNanos};

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
    fn atr_and_tolerance_are_integer() {
        let ticks: Vec<Tick> = (0..10).map(|i| tick(100 + i, i)).collect();
        let atr = atr_proxy_ticks(&ticks, 5);
        assert_eq!(atr, 1);
        let cfg = EqualConfig {
            pips_min_ticks: 2,
            k_atr_num: 1,
            k_atr_den: 1,
            atr_lookback: 5,
        };
        assert_eq!(equal_tolerance_ticks(&cfg, atr), 2);
    }

    #[test]
    fn project_is_exact_on_anchor() {
        let p = project_price(Px(100), 0, 10, 10, 0).expect("dt");
        assert_eq!(p, Px(100));
        let p2 = project_price(Px(100), 0, 10, 10, 10).expect("dt");
        assert_eq!(p2, Px(110));
    }
}
