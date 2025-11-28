//! Time utilities for high-precision timestamps

use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in nanoseconds since epoch
pub fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Get current timestamp in microseconds since epoch
pub fn now_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as u64
}

/// Get current UTC datetime
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}
