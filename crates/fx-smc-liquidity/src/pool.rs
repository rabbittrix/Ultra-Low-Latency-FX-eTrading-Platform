//! Liquidity pool types.

use fx_smc_common::{Px, TsNanos};
use serde::{Deserialize, Serialize};

/// Stable pool identifier within a mapping run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoolId(pub String);

impl PoolId {
    /// Construct an id.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Buy-side vs sell-side resting liquidity (SMC convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolSide {
    /// Liquidity below price (sell stops / buy-side liquidity).
    BuySide,
    /// Liquidity above price (buy stops / sell-side liquidity).
    SellSide,
}

/// Where the pool was derived from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolOrigin {
    /// Equal highs/lows cluster.
    Equal,
    /// Trendline liquidity.
    Trendline,
    /// Asia session extreme.
    Asia,
    /// Previous day high/low.
    PrevDay,
    /// Week high/low.
    Week,
}

/// Mapped liquidity pool with fixed-point score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquidityPool {
    /// Run-local identifier.
    pub id: PoolId,
    /// Buy-side or sell-side.
    pub side: PoolSide,
    /// Anchor price in ticks.
    pub price: Px,
    /// Touch / member count.
    pub touches: u32,
    /// Most recent touch time.
    pub last_touch_ns: TsNanos,
    /// Relative score in `[0, score_scale]`.
    pub score: i64,
    /// Feature origin.
    pub origin: PoolOrigin,
}
