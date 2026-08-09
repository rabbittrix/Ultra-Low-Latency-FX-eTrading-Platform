//! Shared SMC/advisory primitives: fixed-point market types, logical clock, event hash.
//!
//! # Safety
//! Hot-path code must treat these as plain data: no heap growth, no locks, no Tokio.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod clock;
pub mod config;
pub mod error;
pub mod hash;
pub mod types;

pub use clock::LogicalClock;
pub use config::{
    AdvisoryConfig, ApiConfig, AppConfig, BacktestConfig, ClockConfig, DisclaimerConfig,
    EqualConfig, FvgConfig, InstrumentConfig, JournalConfig, LiquidityConfig, LiquidityScoreConfig,
    RiskConfig, SessionConfig, SizingMode, StoreConfig, StrategyConfig, StructureConfig,
    SweepConfig, SwingConfig, SynthConfig, TracingConfig, TrendlineConfig, WindowScoreConfig,
};
pub use error::SmcError;
pub use hash::{EventHash, EventHasher};
pub use types::{InstrumentMeta, Px, Qty, Side, SymbolId, Tick, TsNanos};
