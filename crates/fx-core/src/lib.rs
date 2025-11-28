//! Core matching engine logic for the FX eTrading platform
//!
//! This crate provides the ultra-low-latency matching engine implementation
//! with lock-free data structures and zero-allocation hot paths.

pub mod matching;
pub mod order;
pub mod orderbook;

pub use matching::MatchingEngine;
pub use order::Order;
pub use orderbook::OrderBook;
