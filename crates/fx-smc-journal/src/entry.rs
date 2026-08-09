//! Journal entry types.

use fx_smc_common::TsNanos;
use serde::{Deserialize, Serialize};

/// Kind of journal event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JournalKind {
    /// Plan accepted into paper book.
    PlanOpen,
    /// Plan closed (stop / target / manual).
    PlanClose,
    /// Free-form note.
    Note,
    /// Risk / invalidation reminder.
    RiskNote,
}

/// One capped journal row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Monotonic id within the journal instance.
    pub id: u64,
    /// Event time (nanos).
    pub ts_ns: TsNanos,
    /// Entry kind.
    pub kind: JournalKind,
    /// Related plan id when applicable.
    pub plan_id: Option<String>,
    /// Human detail (may include invalidation / risk language).
    pub detail: String,
}
