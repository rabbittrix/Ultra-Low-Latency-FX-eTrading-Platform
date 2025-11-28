//! Shared utilities for the FX eTrading platform
//!
//! This crate provides common types, error handling, and utility functions
//! used across all services in the platform.

pub mod error;
pub mod time;
pub mod types;

pub use error::{Error, Result};
