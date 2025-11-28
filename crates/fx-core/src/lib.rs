//! Core matching engine logic for the FX eTrading platform
//!
//! This crate provides the ultra-low-latency matching engine implementation
//! with lock-free data structures and zero-allocation hot paths.

pub mod audit_log;
pub mod matching;
pub mod order;
pub mod orderbook;
pub mod trade_log;

pub use audit_log::{AuditEvent, AuditEventType, AuditLog};
pub use matching::{MatchResult, MatchingEngine, Trade};
pub use order::Order;
pub use orderbook::OrderBook;
pub use trade_log::TradeLog;
