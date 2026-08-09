//! Logical clock driven by event input (deterministic live and replay).

use crate::types::TsNanos;

/// Monotonic logical clock. Advances only when fed event timestamps.
#[derive(Debug, Clone, Default)]
pub struct LogicalClock {
    now: TsNanos,
}

impl LogicalClock {
    /// Create a clock at the given epoch.
    #[must_use]
    pub const fn new(epoch: TsNanos) -> Self {
        Self { now: epoch }
    }

    /// Current logical time.
    #[must_use]
    pub const fn now(&self) -> TsNanos {
        self.now
    }

    /// Observe an event timestamp: clock becomes `max(now, ts)`.
    pub fn observe(&mut self, ts: TsNanos) -> TsNanos {
        if ts > self.now {
            self.now = ts;
        }
        self.now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_is_monotonic() {
        let mut c = LogicalClock::new(TsNanos(10));
        assert_eq!(c.observe(TsNanos(5)), TsNanos(10));
        assert_eq!(c.observe(TsNanos(20)), TsNanos(20));
        assert_eq!(c.now(), TsNanos(20));
    }
}
