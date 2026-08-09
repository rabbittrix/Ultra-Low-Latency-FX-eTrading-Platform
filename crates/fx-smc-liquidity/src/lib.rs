//! SMC liquidity mapping and pool scoring (fixed-point).

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod mapper;
pub mod pool;
pub mod score;

pub use mapper::{map_from_ticks, map_liquidity, StructureFeatures};
pub use pool::{LiquidityPool, PoolId, PoolOrigin, PoolSide};
pub use score::{half_life_decay, rescore, score_pool, PoolScoreInput};
