//! Ring-capped in-memory journal.

use crate::entry::{JournalEntry, JournalKind};
use fx_smc_common::{JournalConfig, TsNanos};
use std::collections::VecDeque;

/// FIFO journal capped at `max_entries`.
#[derive(Debug, Clone)]
pub struct Journal {
    entries: VecDeque<JournalEntry>,
    max_entries: usize,
    next_id: u64,
}

impl Journal {
    /// Create from `[journal]` config.
    #[must_use]
    pub fn from_config(cfg: &JournalConfig) -> Self {
        Self::with_capacity(cfg.max_entries)
    }

    /// Create with explicit capacity (`0` ⇒ capacity 1).
    #[must_use]
    pub fn with_capacity(max_entries: usize) -> Self {
        let cap = max_entries.max(1);
        Self {
            entries: VecDeque::with_capacity(cap),
            max_entries: cap,
            next_id: 1,
        }
    }

    /// Append an entry; drops oldest when over capacity. Returns the new entry id.
    pub fn push(
        &mut self,
        ts_ns: TsNanos,
        kind: JournalKind,
        plan_id: Option<String>,
        detail: impl Into<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let entry = JournalEntry {
            id,
            ts_ns,
            kind,
            plan_id,
            detail: detail.into(),
        };
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        id
    }

    /// Borrow retained entries (oldest first).
    #[must_use]
    pub fn entries(&self) -> &VecDeque<JournalEntry> {
        &self.entries
    }

    /// Number of retained entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_at_max() {
        let mut j = Journal::with_capacity(2);
        j.push(TsNanos(1), JournalKind::Note, None, "a");
        j.push(TsNanos(2), JournalKind::Note, None, "b");
        j.push(TsNanos(3), JournalKind::Note, None, "c");
        assert_eq!(j.len(), 2);
        assert_eq!(j.entries()[0].detail, "b");
        assert_eq!(j.entries()[1].detail, "c");
    }
}
