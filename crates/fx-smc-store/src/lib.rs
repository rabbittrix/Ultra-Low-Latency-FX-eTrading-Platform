//! Persistence backends for SMC: Parquet ticks + optional Postgres metadata.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod parquet_store;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod traits;

pub use parquet_store::ParquetTickStore;
#[cfg(feature = "postgres")]
pub use postgres::PostgresStore;
pub use traits::{PaperStatsSnapshot, ResearchProfile, TickStore};
