//! Error type for SMC libraries (`thiserror`).

use thiserror::Error;

/// Library-level SMC errors.
#[derive(Debug, Error)]
pub enum SmcError {
    /// Configuration could not be loaded or parsed.
    #[error("config: {0}")]
    Config(String),
    /// I/O failure (files, Parquet, etc.).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Invalid tick or invariant violation.
    #[error("invalid data: {0}")]
    InvalidData(String),
    /// Store backend failure.
    #[error("store: {0}")]
    Store(String),
}
