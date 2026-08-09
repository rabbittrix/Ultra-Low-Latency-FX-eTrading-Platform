//! SMC research journal and paper simulator (ADR-0009).
//!
//! Paper stats are for process discipline — not a forecast of live results.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod entry;
pub mod paper;
pub mod ring;

pub use entry::{JournalEntry, JournalKind};
pub use paper::{PaperSimulator, PaperStats};
pub use ring::Journal;
