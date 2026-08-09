//! Fixed-point trading cost model (ticks only).

use fx_smc_common::BacktestConfig;
use serde::{Deserialize, Serialize};

/// Spread / commission / slippage in ticks per side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostModel {
    /// Half-spread (or configured spread) charged each side.
    pub spread_ticks: i64,
    /// Commission charged each side.
    pub commission_ticks_per_side: i64,
    /// Slippage charged each side.
    pub slippage_ticks_per_side: i64,
}

impl CostModel {
    /// Build from `[backtest]` config (non-negative clamps).
    #[must_use]
    pub fn from_config(cfg: &BacktestConfig) -> Self {
        Self {
            spread_ticks: cfg.spread_ticks.max(0),
            commission_ticks_per_side: cfg.commission_ticks_per_side.max(0),
            slippage_ticks_per_side: cfg.slippage_ticks_per_side.max(0),
        }
    }

    /// Adverse price move per side from spread + slippage (commission is PnL-only).
    #[must_use]
    pub fn adverse_ticks_per_side(self) -> i64 {
        self.spread_ticks
            .saturating_add(self.slippage_ticks_per_side)
    }
}
