//! Common types used across the platform

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for orders
pub type OrderId = Uuid;

/// Unique identifier for trades
pub type TradeId = Uuid;

/// Unique identifier for instruments
pub type InstrumentId = String;

/// Currency pair (e.g., "EURUSD", "GBPUSD")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurrencyPair {
    pub base: String,
    pub quote: String,
}

impl CurrencyPair {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

/// Price representation (fixed-point for precision)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(pub u64);

impl Price {
    /// Create a price from a decimal representation
    /// Example: Price::from_decimal(1.2345, 4) = 12345
    pub fn from_decimal(value: f64, decimals: u8) -> Self {
        let multiplier = 10_u64.pow(decimals as u32);
        Self((value * multiplier as f64) as u64)
    }

    /// Convert price to decimal
    pub fn to_decimal(self, decimals: u8) -> f64 {
        let divisor = 10_u64.pow(decimals as u32);
        self.0 as f64 / divisor as f64
    }
}

/// Quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Quantity(pub u64);

/// Side of an order (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    IoC, // Immediate or Cancel
    FoK, // Fill or Kill
}
