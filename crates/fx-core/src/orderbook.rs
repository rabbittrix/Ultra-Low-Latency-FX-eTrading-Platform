//! Lock-free order book implementation

use crate::order::Order;
use fx_utils::{Price, Quantity, Side};
use std::sync::Arc;

/// Price level in the order book
#[derive(Debug, Clone)]
pub struct Level {
    pub price: Price,
    pub total_quantity: Quantity,
    pub orders: Vec<Arc<Order>>,
}

/// Lock-free order book using atomic operations
pub struct OrderBook {
    #[allow(dead_code)]
    instrument: String,
    bids: Vec<Level>, // Sorted descending (highest first)
    asks: Vec<Level>, // Sorted ascending (lowest first)
}

impl OrderBook {
    pub fn new(instrument: String) -> Self {
        Self {
            instrument,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn add_order(&mut self, order: Arc<Order>) {
        let levels = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        if let Some(price) = order.price {
            // Find or create level
            let pos = levels
                .binary_search_by_key(&price.0, |l| {
                    match order.side {
                        Side::Buy => u64::MAX - l.price.0, // Reverse for bids
                        Side::Sell => l.price.0,
                    }
                })
                .unwrap_or_else(|e| e);

            if pos < levels.len() && levels[pos].price == price {
                levels[pos].orders.push(order.clone());
                levels[pos].total_quantity.0 += order.remaining_quantity.0;
            } else {
                levels.insert(
                    pos,
                    Level {
                        price,
                        total_quantity: order.remaining_quantity,
                        orders: vec![order],
                    },
                );
            }
        }
    }

    pub fn best_bid(&self) -> Option<&Level> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&Level> {
        self.asks.first()
    }

    pub fn spread(&self) -> Option<u64> {
        let bid = self.best_bid()?.price.0;
        let ask = self.best_ask()?.price.0;
        Some(ask.saturating_sub(bid))
    }

    pub fn bids_mut(&mut self) -> &mut Vec<Level> {
        &mut self.bids
    }

    pub fn asks_mut(&mut self) -> &mut Vec<Level> {
        &mut self.asks
    }
}
