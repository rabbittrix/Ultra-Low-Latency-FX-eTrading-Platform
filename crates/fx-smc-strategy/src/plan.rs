//! Trade plan types.

use crate::trace::ReasoningTrace;
use fx_smc_common::{Px, TsNanos};
use fx_smc_liquidity::PoolId;
use serde::{Deserialize, Serialize};

/// Proposed trade direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeSide {
    /// Buy / long bias after buy-side liquidity sweep.
    Long,
    /// Sell / short bias after sell-side liquidity sweep.
    Short,
}

/// Candidate trade plan (not an order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradePlan {
    /// Stable id within a build run.
    pub id: String,
    /// Direction.
    pub side: TradeSide,
    /// Source sweep pool id.
    pub sweep_pool_id: PoolId,
    /// Entry price (ticks).
    pub entry: Px,
    /// Invalidation / stop (ticks).
    pub stop: Px,
    /// Take-profit target (ticks).
    pub target: Px,
    /// Risk distance in ticks.
    pub risk_ticks: i64,
    /// Reward distance in ticks.
    pub reward_ticks: i64,
    /// R:R numerator (reward * den comparable form stored separately).
    pub rr_num: i64,
    /// R:R denominator.
    pub rr_den: i64,
    /// Confluence score.
    pub confluence: i64,
    /// Confirm timestamp.
    pub as_of_ns: TsNanos,
    /// Confirm tick index.
    pub as_of_idx: usize,
    /// Invalidation narrative.
    pub invalidation: String,
    /// Risk disclaimer fragment.
    pub disclaimer: String,
    /// Audit trail.
    pub reasoning: ReasoningTrace,
}
