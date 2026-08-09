//! Market data helpers and synthetic tick generation for SMC pipelines.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod scenarios;
pub mod synth;

pub use scenarios::{generate_scenario, SynthScenario};
pub use synth::{generate_ticks, SynthParams};
