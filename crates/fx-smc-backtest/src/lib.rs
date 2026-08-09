//! SMC backtest: prefix-only analysis, cost model, walk-forward (ADR-0006).
//!
//! Simulations are research tools — not predictions of future returns.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod cost;
pub mod engine;
pub mod fill;
pub mod report;

pub use cost::CostModel;
pub use engine::{
    analyze_prefix, collect_prefix_plans, run_backtest, walk_forward, BacktestSections,
};
pub use fill::{seed_from_plan, FillRecord, Xoshiro256PlusPlus};
pub use report::{pnl_curve_fingerprint, BacktestReport, CostReport};
