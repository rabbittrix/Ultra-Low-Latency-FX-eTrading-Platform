//! Reasoning trace for auditability.

use serde::{Deserialize, Serialize};

/// One reasoning step with a stable machine code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasonStep {
    /// Stable code (`SWEEP`, `STOP`, `TARGET`, …).
    pub code: String,
    /// Human-readable detail (may include risk / invalidation wording).
    pub detail: String,
}

/// Ordered audit trail for a trade plan.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReasoningTrace {
    /// Steps in emission order.
    pub steps: Vec<ReasonStep>,
}

impl ReasoningTrace {
    /// Append a step.
    pub fn push(&mut self, code: impl Into<String>, detail: impl Into<String>) {
        self.steps.push(ReasonStep {
            code: code.into(),
            detail: detail.into(),
        });
    }
}
