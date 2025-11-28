//! Matching engine algorithm

use crate::order::Order;
use crate::orderbook::OrderBook;
use fx_utils::{Price, Quantity, Side, TradeId};
use std::sync::Arc;
use uuid::Uuid;

/// Result of matching an order
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub order: Arc<Order>,
}

/// Trade execution
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: TradeId,
    pub buy_order_id: fx_utils::OrderId,
    pub sell_order_id: fx_utils::OrderId,
    pub instrument: String,
    pub quantity: Quantity,
    pub price: Price,
    pub timestamp_ns: u64,
}

/// Matching engine implementation
pub struct MatchingEngine {
    orderbook: OrderBook,
}

impl MatchingEngine {
    pub fn new(instrument: String) -> Self {
        Self {
            orderbook: OrderBook::new(instrument),
        }
    }

    pub fn match_order(&mut self, order: Arc<Order>) -> MatchResult {
        let mut trades = Vec::new();
        let mut remaining_order = order.clone();

        // Match against opposite side
        let opposite_levels = match order.side {
            Side::Buy => self.orderbook.asks_mut(),
            Side::Sell => self.orderbook.bids_mut(),
        };

        while !remaining_order.is_filled() && !opposite_levels.is_empty() {
            let level = &mut opposite_levels[0];

            // Check if we can match
            let can_match = match remaining_order.price {
                Some(limit_price) => match order.side {
                    Side::Buy => limit_price.0 >= level.price.0,
                    Side::Sell => limit_price.0 <= level.price.0,
                },
                None => true, // Market order
            };

            if !can_match {
                break;
            }

            // Match against orders in this level
            let mut level_idx = 0;
            while level_idx < level.orders.len() && !remaining_order.is_filled() {
                let counter_order = &level.orders[level_idx];
                let trade_qty = Quantity(
                    remaining_order
                        .remaining_quantity
                        .0
                        .min(counter_order.remaining_quantity.0),
                );

                // Create trade
                let trade = Trade {
                    id: Uuid::new_v4(),
                    buy_order_id: match order.side {
                        Side::Buy => order.id,
                        Side::Sell => counter_order.id,
                    },
                    sell_order_id: match order.side {
                        Side::Buy => counter_order.id,
                        Side::Sell => order.id,
                    },
                    instrument: order.instrument.clone(),
                    quantity: trade_qty,
                    price: level.price,
                    timestamp_ns: fx_utils::time::now_nanos(),
                };

                trades.push(trade);
                remaining_order.fill(trade_qty);

                // Update counter order
                let mut updated_counter = (*counter_order).clone();
                updated_counter.fill(trade_qty);
                level.orders[level_idx] = Arc::new(updated_counter);

                if level.orders[level_idx].is_filled() {
                    level.orders.remove(level_idx);
                } else {
                    level_idx += 1;
                }
            }

            // Update level quantity
            level.total_quantity.0 = level.orders.iter().map(|o| o.remaining_quantity.0).sum();

            // Remove empty levels
            if level.orders.is_empty() {
                opposite_levels.remove(0);
            }
        }

        // Add remaining order to book if not fully filled
        if !remaining_order.is_filled() {
            if let Some(price) = remaining_order.price {
                self.orderbook.add_order(Arc::new(remaining_order.clone()));
            }
        }

        MatchResult {
            trades,
            order: Arc::new(remaining_order),
        }
    }
}
