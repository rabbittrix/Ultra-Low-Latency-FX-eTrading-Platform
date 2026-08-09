//! SMC advisory: regimes, window scores, suitability (ADR-0008 / ADR-0012).
//!
//! Outputs are informational only — not investment advice and not a promise of returns.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod entry_window;
pub mod fact;
pub mod regime;
pub mod suitability;
pub mod window;

pub use entry_window::{
    best_entry_window, conf_from_structure_breaks, score_entry_window, ConfSignal, EntrySide,
    EntryWindowScore, WindowColor,
};
pub use fact::Fact;
pub use regime::{classify_regime, Regime};
pub use suitability::{suitability, Suitability};
pub use window::{rank_symbols, rank_windows, score_window, WindowScore};
