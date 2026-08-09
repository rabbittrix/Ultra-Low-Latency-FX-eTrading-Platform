//! Position sizing from equity and stop distance (fixed-point ticks).

use fx_smc_common::{Qty, RiskConfig, SizingMode};
use thiserror::Error;

use crate::kill::KillSwitch;

/// Why sizing refused a quantity.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RiskReject {
    /// Kill switch is tripped (daily loss or feed guard).
    #[error("kill switch tripped — new size blocked")]
    KillTripped,
    /// Open plan count at or above configured max.
    #[error("max open plans reached ({open}>={max})")]
    MaxOpenPlans {
        /// Current open plans.
        open: u32,
        /// Configured maximum.
        max: u32,
    },
    /// Stop distance below minimum.
    #[error("stop distance {stop} < min_stop_ticks {min}")]
    MinStop {
        /// Requested stop distance.
        stop: i64,
        /// Configured minimum.
        min: i64,
    },
    /// Non-positive inputs or zero quantity after division.
    #[error("invalid sizing inputs (equity={equity}, stop={stop}, qty would be {qty})")]
    InvalidInputs {
        /// Equity in ticks.
        equity: i64,
        /// Stop distance.
        stop: i64,
        /// Computed qty before reject.
        qty: i64,
    },
}

/// Fixed-bps size: `risk_ticks = equity * bps / 10000`, `qty = risk_ticks / stop_distance`.
#[must_use]
pub fn size_qty_fixed_bps(equity_ticks: i64, stop_distance: i64, cfg: &RiskConfig) -> Qty {
    if equity_ticks <= 0 || stop_distance <= 0 {
        return Qty(0);
    }
    let bps = cfg.risk_per_trade_bps.max(0);
    let risk_ticks = equity_ticks.saturating_mul(bps) / 10_000;
    if risk_ticks <= 0 {
        return Qty(0);
    }
    Qty(risk_ticks / stop_distance)
}

/// Kelly fraction in milli of equity, then qty; capped by `risk_per_trade_bps` max risk.
///
/// `f* = p - (1-p)/b` with milli inputs; applied fraction = `f* * kelly_fraction_milli / 1000`.
/// Research sizing only — does not imply expected returns.
#[must_use]
pub fn size_qty_kelly(equity_ticks: i64, stop_distance: i64, cfg: &RiskConfig) -> Qty {
    if equity_ticks <= 0 || stop_distance <= 0 {
        return Qty(0);
    }
    let p = cfg.kelly_win_prob_milli.clamp(0, 1000);
    let b = cfg.kelly_payoff_milli.max(1);
    // f_milli = p - (1000-p)*1000 / b
    let one_minus_p = 1000_i64.saturating_sub(p);
    let edge = p.saturating_sub(one_minus_p.saturating_mul(1000) / b);
    if edge <= 0 {
        return Qty(0);
    }
    let frac = cfg.kelly_fraction_milli.clamp(0, 1000);
    let f_eff_milli = edge.saturating_mul(frac) / 1000;
    let kelly_risk = equity_ticks.saturating_mul(f_eff_milli) / 1000;
    let bps_cap = equity_ticks.saturating_mul(cfg.risk_per_trade_bps.max(0)) / 10_000;
    let risk_ticks = kelly_risk.min(bps_cap);
    if risk_ticks <= 0 {
        return Qty(0);
    }
    Qty(risk_ticks / stop_distance)
}

/// Core size dispatch on [`SizingMode`].
#[must_use]
pub fn size_qty(equity_ticks: i64, stop_distance: i64, cfg: &RiskConfig) -> Qty {
    match cfg.sizing_mode {
        SizingMode::FixedBps => size_qty_fixed_bps(equity_ticks, stop_distance, cfg),
        SizingMode::Kelly => size_qty_kelly(equity_ticks, stop_distance, cfg),
    }
}

/// Apply kill switch, `max_open_plans`, and `min_stop_ticks` before sizing.
///
/// # Errors
/// Returns [`RiskReject`] when a guardrail blocks sizing.
pub fn size_qty_guarded(
    equity_ticks: i64,
    stop_distance: i64,
    open_plans: u32,
    kill: &KillSwitch,
    cfg: &RiskConfig,
) -> Result<Qty, RiskReject> {
    if kill.is_tripped() {
        return Err(RiskReject::KillTripped);
    }
    if open_plans >= cfg.max_open_plans {
        return Err(RiskReject::MaxOpenPlans {
            open: open_plans,
            max: cfg.max_open_plans,
        });
    }
    let min_stop = cfg.min_stop_ticks.max(1);
    if stop_distance < min_stop {
        return Err(RiskReject::MinStop {
            stop: stop_distance,
            min: min_stop,
        });
    }
    let qty = size_qty(equity_ticks, stop_distance, cfg);
    if qty.0 <= 0 {
        return Err(RiskReject::InvalidInputs {
            equity: equity_ticks,
            stop: stop_distance,
            qty: qty.0,
        });
    }
    Ok(qty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;

    #[test]
    fn size_qty_integer_math() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        // equity 100_000, bps 50 → risk 500; stop 10 → qty 50
        let q = size_qty(100_000, 10, &cfg.risk);
        assert_eq!(q, Qty(50));
    }

    #[test]
    fn kelly_positive_and_capped() {
        let mut cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        cfg.risk.sizing_mode = SizingMode::Kelly;
        // p=0.55, b=1.5 → f* = 0.55 - 0.45/1.5 = 0.55 - 0.3 = 0.25 → 250 milli
        // half-kelly fraction 250 → f_eff = 250*250/1000 = 62 milli of equity
        // equity 100_000 → risk 6200; bps cap 500 → risk 500; stop 10 → qty 50
        let q = size_qty_kelly(100_000, 10, &cfg.risk);
        assert!(q.0 > 0);
        let capped = size_qty_fixed_bps(100_000, 10, &cfg.risk);
        assert_eq!(q, capped, "kelly risk must be capped by risk_per_trade_bps");

        // Raise bps cap so kelly can exceed fixed size
        cfg.risk.risk_per_trade_bps = 10_000; // 100%
        let q2 = size_qty_kelly(100_000, 10, &cfg.risk);
        assert!(q2.0 > capped.0);
    }

    #[test]
    fn guardrails_and_kill() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut kill = KillSwitch::default();
        assert!(size_qty_guarded(100_000, 10, 0, &kill, &cfg.risk).is_ok());

        assert_eq!(
            size_qty_guarded(100_000, 1, 0, &kill, &cfg.risk),
            Err(RiskReject::MinStop {
                stop: 1,
                min: cfg.risk.min_stop_ticks.max(1)
            })
        );

        assert!(matches!(
            size_qty_guarded(100_000, 10, cfg.risk.max_open_plans, &kill, &cfg.risk),
            Err(RiskReject::MaxOpenPlans { .. })
        ));

        kill.record_pnl(-cfg.risk.max_daily_loss_ticks, &cfg.risk);
        assert!(kill.is_tripped());
        assert_eq!(
            size_qty_guarded(100_000, 10, 0, &kill, &cfg.risk),
            Err(RiskReject::KillTripped)
        );
    }
}
