//! SMC trade plans: R:R, confluence, `ReasoningTrace` (ADR-0005).

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod builder;
pub mod plan;
pub mod trace;

pub use builder::build_plans;
pub use plan::{TradePlan, TradeSide};
pub use trace::{ReasonStep, ReasoningTrace};
