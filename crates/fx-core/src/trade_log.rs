//! Trade log for storing executed trades

use crate::matching::Trade;
use fx_utils::TradeId;
use parking_lot::RwLock;
use std::sync::Arc;

/// Trade log that stores all executed trades
pub struct TradeLog {
    trades: Arc<RwLock<Vec<Trade>>>,
    max_size: usize,
}

impl TradeLog {
    pub fn new(max_size: usize) -> Self {
        Self {
            trades: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add a trade to the log
    pub fn add_trade(&self, trade: Trade) {
        let mut trades = self.trades.write();
        trades.push(trade);

        // Keep only the most recent trades if we exceed max_size
        if trades.len() > self.max_size {
            trades.remove(0);
        }
    }

    /// Get all trades
    pub fn get_trades(&self) -> Vec<Trade> {
        self.trades.read().clone()
    }

    /// Get trades for a specific order
    pub fn get_trades_for_order(&self, order_id: fx_utils::OrderId) -> Vec<Trade> {
        self.trades
            .read()
            .iter()
            .filter(|t| t.buy_order_id == order_id || t.sell_order_id == order_id)
            .cloned()
            .collect()
    }

    /// Get trade by ID
    pub fn get_trade(&self, trade_id: TradeId) -> Option<Trade> {
        self.trades
            .read()
            .iter()
            .find(|t| t.id == trade_id)
            .cloned()
    }

    /// Get recent trades (last N trades)
    pub fn get_recent_trades(&self, limit: usize) -> Vec<Trade> {
        let trades = self.trades.read();
        let start = trades.len().saturating_sub(limit);
        trades[start..].to_vec()
    }
}
