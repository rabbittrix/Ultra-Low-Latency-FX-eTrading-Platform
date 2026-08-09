//! SMC risk sizing and kill-switch (ADR-0007).
//!
//! Sizing helpers do not place orders and never imply expected returns.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod kill;
pub mod size;

pub use kill::KillSwitch;
pub use size::{size_qty, size_qty_fixed_bps, size_qty_guarded, size_qty_kelly, RiskReject};
