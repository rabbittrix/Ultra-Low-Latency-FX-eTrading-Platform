//! Matching engine algorithm

use crate::audit_log::{AuditEvent, AuditEventType, AuditLog};
use crate::order::Order;
use crate::orderbook::OrderBook;
use crate::trade_log::TradeLog;
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
    trade_log: Arc<TradeLog>,
    audit_log: Arc<AuditLog>,
}

impl MatchingEngine {
    pub fn new(instrument: String) -> Self {
        Self {
            orderbook: OrderBook::new(instrument),
            trade_log: Arc::new(TradeLog::new(10_000)),
            audit_log: Arc::new(AuditLog::new(10_000)),
        }
    }

    /// Get reference to trade log
    pub fn trade_log(&self) -> &Arc<TradeLog> {
        &self.trade_log
    }

    /// Get reference to audit log
    pub fn audit_log(&self) -> &Arc<AuditLog> {
        &self.audit_log
    }

    /// Get mutable reference to orderbook (for testing)
    pub fn orderbook_mut(&mut self) -> &mut OrderBook {
        &mut self.orderbook
    }

    /// Get reference to orderbook (for testing and internal use)
    pub fn orderbook(&self) -> &OrderBook {
        &self.orderbook
    }

    pub fn match_order(&mut self, order: Arc<Order>) -> MatchResult {
        // Log order submission
        self.audit_log.add_event(AuditEvent::from_order(
            AuditEventType::Submitted,
            &order,
            None,
        ));

        let mut trades = Vec::new();
        let mut remaining_order = (*order).clone();

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

                trades.push(trade.clone());
                self.trade_log.add_trade(trade);
                remaining_order.fill(trade_qty);

                // Update counter order
                let mut updated_counter = (**counter_order).clone();
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
        let remaining_order_arc = Arc::new(remaining_order.clone());
        if !remaining_order.is_filled() {
            if let Some(_price) = remaining_order.price {
                self.orderbook.add_order(remaining_order_arc.clone());
                // Log partial fill
                if !trades.is_empty() {
                    self.audit_log.add_event(AuditEvent::from_order(
                        AuditEventType::PartiallyFilled,
                        &remaining_order,
                        Some(format!("Filled {} trades", trades.len())),
                    ));
                }
            } else {
                // Market order not fully filled - reject remaining
                self.audit_log.add_event(AuditEvent::from_order(
                    AuditEventType::Rejected,
                    &remaining_order,
                    Some("Market order not fully filled".to_string()),
                ));
            }
        } else {
            // Order fully filled
            self.audit_log.add_event(AuditEvent::from_order(
                AuditEventType::Filled,
                &remaining_order,
                Some(format!("Filled {} trades", trades.len())),
            ));
        }

        MatchResult {
            trades,
            order: remaining_order_arc,
        }
    }

    /// Cancel a resting order. Returns `true` if the order was found and removed.
    pub fn cancel_order(&mut self, order_id: fx_utils::OrderId) -> bool {
        let mut removed: Option<(Side, Option<Price>, u64)> = None;
        'outer: for side in [Side::Buy, Side::Sell] {
            let levels = match side {
                Side::Buy => self.orderbook.bids_mut(),
                Side::Sell => self.orderbook.asks_mut(),
            };
            let mut li = 0;
            while li < levels.len() {
                if let Some(oi) = levels[li].orders.iter().position(|o| o.id == order_id) {
                    let ord = levels[li].orders.remove(oi);
                    levels[li].total_quantity.0 = levels[li]
                        .orders
                        .iter()
                        .map(|o| o.remaining_quantity.0)
                        .sum();
                    if levels[li].orders.is_empty() {
                        levels.remove(li);
                    }
                    removed = Some((ord.side, ord.price, ord.remaining_quantity.0));
                    break 'outer;
                }
                li += 1;
            }
        }

        match removed {
            Some((side, price, quantity)) => {
                self.audit_log.add_event(AuditEvent {
                    event_type: AuditEventType::Cancelled,
                    order_id,
                    instrument: self.orderbook.instrument().to_string(),
                    side,
                    order_type: fx_utils::OrderType::Limit,
                    quantity,
                    price: price.map(|p| p.0),
                    timestamp_ns: fx_utils::time::now_nanos(),
                    message: Some("Order cancelled".to_string()),
                });
                true
            }
            None => false,
        }
    }
}
