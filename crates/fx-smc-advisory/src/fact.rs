//! Explainable facts for entry-window scoring (ADR-0012).

use std::fmt;

/// Structured explanation attached to an [`super::entry_window::EntryWindowScore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    /// Sweep contribution summary.
    Sweep {
        /// Opposite-side pool id if any.
        pool_id: Option<String>,
        /// Component score used.
        score: i64,
    },
    /// Confluence / structure signal (may be stubbed).
    Conf {
        /// Signal label.
        signal: String,
        /// Component score.
        score: i64,
    },
    /// R:R estimate contribution.
    Rr {
        /// Estimated R:R in milli-R.
        rr_milli: i64,
        /// Component score.
        score: i64,
    },
    /// Session / kill-zone contribution.
    Sess {
        /// Hour UTC used.
        hour_utc: u8,
        /// Component score.
        score: i64,
    },
    /// Regime alignment contribution.
    Reg {
        /// Regime label.
        regime: String,
        /// Component score.
        score: i64,
    },
    /// A soft/hard gate fired.
    Gate {
        /// Gate id (`G1`..`G4`).
        id: &'static str,
        /// Human reason.
        reason: String,
    },
    /// Detectors incomplete / stubbed inputs.
    DataDegraded {
        /// Why degraded.
        reason: String,
    },
    /// Mandatory risk disclaimer.
    Disclaimer {
        /// Disclaimer text.
        text: String,
    },
}

impl fmt::Display for Fact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sweep { pool_id, score } => match pool_id {
                Some(id) => write!(f, "sweep pool={id} s={score}"),
                None => write!(f, "sweep none s={score}"),
            },
            Self::Conf { signal, score } => write!(f, "conf {signal} s={score}"),
            Self::Rr { rr_milli, score } => write!(f, "rr {rr_milli}mR s={score}"),
            Self::Sess { hour_utc, score } => write!(f, "sess hour={hour_utc} s={score}"),
            Self::Reg { regime, score } => write!(f, "reg {regime} s={score}"),
            Self::Gate { id, reason } => write!(f, "gate {id}: {reason}"),
            Self::DataDegraded { reason } => write!(f, "data_degraded: {reason}"),
            Self::Disclaimer { text } => write!(f, "disclaimer: {text}"),
        }
    }
}
