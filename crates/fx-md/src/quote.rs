//! Quote representation

use fx_utils::{Price, Quantity};
use serde::{Deserialize, Serialize};

/// Market quote (bid/ask)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub instrument: String,
    pub bid_price: Price,
    pub ask_price: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub timestamp_ns: u64,
}

impl Quote {
    pub fn spread(&self) -> u64 {
        self.ask_price.0.saturating_sub(self.bid_price.0)
    }

    pub fn mid_price(&self) -> Price {
        Price((self.bid_price.0 + self.ask_price.0) / 2)
    }
}
