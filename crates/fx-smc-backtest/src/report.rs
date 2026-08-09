//! Aggregated cost / `PnL` report (fixed-point ticks).

use crate::cost::CostModel;
use crate::fill::FillRecord;
use blake3::Hasher;
use fx_smc_common::EventHash;
use serde::{Deserialize, Serialize};

/// Summed cost legs across simulated fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CostReport {
    /// Total spread ticks charged (all sides).
    pub total_spread: i64,
    /// Total commission ticks charged.
    pub commission: i64,
    /// Total slippage ticks charged.
    pub slippage: i64,
    /// Net `PnL` in ticks after costs (research metric only).
    pub net_pnl_ticks: i64,
}

impl CostReport {
    /// Absorb one round-trip (entry + exit) cost legs and add `gross_pnl` after adverse fills.
    #[must_use]
    pub fn with_round_trip(mut self, model: CostModel, gross_pnl_after_adverse: i64) -> Self {
        self.total_spread = self
            .total_spread
            .saturating_add(model.spread_ticks.saturating_mul(2));
        self.commission = self
            .commission
            .saturating_add(model.commission_ticks_per_side.saturating_mul(2));
        self.slippage = self
            .slippage
            .saturating_add(model.slippage_ticks_per_side.saturating_mul(2));
        let costs = model.commission_ticks_per_side.saturating_mul(2);
        self.net_pnl_ticks = self
            .net_pnl_ticks
            .saturating_add(gross_pnl_after_adverse.saturating_sub(costs));
        self
    }

    /// BLAKE3 fingerprint of report fields (deterministic).
    #[must_use]
    pub fn fingerprint(self) -> EventHash {
        let mut h = Hasher::new();
        h.update(&self.total_spread.to_le_bytes());
        h.update(&self.commission.to_le_bytes());
        h.update(&self.slippage.to_le_bytes());
        h.update(&self.net_pnl_ticks.to_le_bytes());
        EventHash(*h.finalize().as_bytes())
    }
}

/// Full backtest summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacktestReport {
    /// Cost / `PnL` totals.
    pub costs: CostReport,
    /// Number of plans simulated.
    pub plans_simulated: u64,
    /// Wins (target hit before stop).
    pub wins: u64,
    /// Losses (stop or time-stop adverse).
    pub losses: u64,
    /// Per-plan fill legs (order = simulation order).
    pub fills: Vec<FillRecord>,
}

impl BacktestReport {
    /// Fingerprint costs + counts + fill curve.
    #[must_use]
    pub fn fingerprint(&self) -> EventHash {
        let mut h = Hasher::new();
        let c = self.costs.fingerprint();
        h.update(&c.0);
        h.update(&self.plans_simulated.to_le_bytes());
        h.update(&self.wins.to_le_bytes());
        h.update(&self.losses.to_le_bytes());
        h.update(&pnl_curve_fingerprint(self));
        EventHash(*h.finalize().as_bytes())
    }
}

/// BLAKE3 over `(plan_id, exit_idx, pnl_ticks)` sequence.
#[must_use]
pub fn pnl_curve_fingerprint(report: &BacktestReport) -> [u8; 32] {
    let mut h = Hasher::new();
    for f in &report.fills {
        h.update(f.plan_id.as_bytes());
        h.update(&[0xff]);
        h.update(&f.exit_idx.to_le_bytes());
        h.update(&f.pnl_ticks.to_le_bytes());
    }
    *h.finalize().as_bytes()
}
