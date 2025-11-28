//! Order representation and types

use fx_utils::{OrderId, OrderType, Price, Quantity, Side};
use serde::{Deserialize, Serialize};

/// Internal order representation optimized for matching engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub instrument: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>, // None for market orders
    pub timestamp_ns: u64,
    pub remaining_quantity: Quantity,
}

impl Order {
    pub fn new(
        id: OrderId,
        instrument: String,
        side: Side,
        order_type: OrderType,
        quantity: Quantity,
        price: Option<Price>,
    ) -> Self {
        let timestamp_ns = fx_utils::time::now_nanos();
        Self {
            id,
            instrument,
            side,
            order_type,
            quantity,
            price,
            timestamp_ns,
            remaining_quantity: quantity,
        }
    }

    pub fn is_filled(&self) -> bool {
        self.remaining_quantity.0 == 0
    }

    pub fn fill(&mut self, quantity: Quantity) {
        if quantity.0 > self.remaining_quantity.0 {
            self.remaining_quantity.0 = 0;
        } else {
            self.remaining_quantity.0 -= quantity.0;
        }
    }
}
