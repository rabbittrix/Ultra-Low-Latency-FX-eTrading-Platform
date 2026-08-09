//! Sweep event types.

use fx_smc_common::TsNanos;
use fx_smc_liquidity::{PoolId, PoolSide};
use serde::{Deserialize, Serialize};

/// Confirmed liquidity sweep (pierce + reclaim within window).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SweepEvent {
    /// Pool that was swept.
    pub pool_id: PoolId,
    /// Pool side (buy-side / sell-side liquidity).
    pub side: PoolSide,
    /// Pool anchor price in ticks.
    pub pool_price_ticks: i64,
    /// Extreme pierce price observed at pierce tick.
    pub pierce_price_ticks: i64,
    /// Displacement beyond the pool at pierce (`|pierce - pool|`).
    pub displacement_ticks: i64,
    /// Pierce timestamp.
    pub pierce_ts_ns: TsNanos,
    /// Confirm (reclaim) timestamp.
    pub confirm_ts_ns: TsNanos,
    /// Tick index of pierce (`0`-based).
    pub pierce_idx: usize,
    /// Tick index of confirm.
    pub confirm_idx: usize,
}
