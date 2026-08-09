//! Store traits (Parquet ticks; Postgres profiles/journal/stats behind feature).

use fx_smc_common::{SmcError, Tick, TsNanos};
use serde::{Deserialize, Serialize};

/// Read/write tick sequences.
pub trait TickStore {
    /// Replace or create a named dataset with the given ticks (ordered).
    ///
    /// # Errors
    /// Backend-specific I/O or schema errors.
    fn write_ticks(&self, dataset: &str, ticks: &[Tick]) -> Result<(), SmcError>;

    /// Read all ticks from a named dataset in stored order.
    ///
    /// # Errors
    /// Backend-specific I/O or schema errors.
    fn read_ticks(&self, dataset: &str) -> Result<Vec<Tick>, SmcError>;
}

/// Research user profile (sizing preferences — not a returns promise).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchProfile {
    /// Stable profile id.
    pub id: String,
    /// Preferred risk per trade in basis points.
    pub risk_bps: i64,
    /// Free-form notes (may include risk reminders).
    pub notes: String,
    /// Last update time (ns UTC).
    pub updated_ns: TsNanos,
}

/// Persisted paper stats snapshot (fixed-point fields only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperStatsSnapshot {
    /// Closed trades.
    pub trades: u64,
    /// Winning closes.
    pub wins: u64,
    /// Losing closes.
    pub losses: u64,
    /// Net `PnL` in ticks.
    pub net_pnl_ticks: i64,
    /// Win rate in basis points.
    pub win_rate_bps: i64,
    /// Snapshot time (ns UTC).
    pub updated_ns: TsNanos,
}
