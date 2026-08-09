//! Tick-level SMC liquidity sweep detector (ADR-0004).
//!
//! Prices are fixed-point ticks. Confirmation never looks past the configured tick window.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod detect;
pub mod event;

pub use detect::{detect_sweeps, detect_sweeps_from_ticks};
pub use event::SweepEvent;
