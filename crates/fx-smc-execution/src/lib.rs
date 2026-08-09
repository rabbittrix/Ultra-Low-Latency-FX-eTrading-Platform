//! SMC hot-path execution: SPSC rings and Copy event slots (ADR-0014).
//!
//! No Tokio, no locks, no heap allocation on the push/pop path. Research intents only —
//! not a promise of fills or returns.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod engine;
pub mod ring;
pub mod slot;

pub use engine::{HotPathEngine, HotPathError};
pub use ring::{spsc_pair, SpscPair};
pub use slot::{ExecIntent, ExecSide, TickSlot};
