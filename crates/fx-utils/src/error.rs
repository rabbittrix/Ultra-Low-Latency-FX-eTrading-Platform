//! Error types and result aliases for the FX eTrading platform

use thiserror::Error;

/// Common error types used across the platform
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using the platform error type
pub type Result<T> = std::result::Result<T, Error>;
