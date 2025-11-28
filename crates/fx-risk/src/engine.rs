//! Risk engine implementation

use crate::limits::RiskLimits;
use fx_utils::{OrderId, Quantity, Result, Side};
use dashmap::DashMap;
use std::sync::Arc;

/// Current position for an instrument
#[derive(Debug, Clone)]
struct Position {
    instrument: String,
    quantity: i64, // Positive for long, negative for short
}

/// Risk engine for pre-trade validation
pub struct RiskEngine {
    limits: Arc<RiskLimits>,
    positions: DashMap<String, Position>,
    open_orders: DashMap<OrderId, Quantity>,
}

impl RiskEngine {
    pub fn new(limits: RiskLimits) -> Self {
        Self {
            limits: Arc::new(limits),
            positions: DashMap::new(),
            open_orders: DashMap::new(),
        }
    }

    pub fn check_order(
        &self,
        instrument: &str,
        side: Side,
        quantity: Quantity,
        order_id: OrderId,
    ) -> Result<()> {
        // Check order size limit
        if quantity.0 > self.limits.max_order_size.0 {
            return Err(fx_utils::Error::InvalidInput(format!(
                "Order size {} exceeds limit {}",
                quantity.0, self.limits.max_order_size.0
            )));
        }

        // Check open orders limit
        if self.open_orders.len() >= self.limits.max_open_orders {
            return Err(fx_utils::Error::InvalidInput(
                "Maximum open orders limit reached".to_string(),
            ));
        }

        // Check position limit
        let current_position = self
            .positions
            .get(instrument)
            .map(|p| p.quantity)
            .unwrap_or(0);

        let new_position = match side {
            Side::Buy => current_position + quantity.0 as i64,
            Side::Sell => current_position - quantity.0 as i64,
        };

        if new_position.abs() as u64 > self.limits.max_position_size.0 {
            return Err(fx_utils::Error::InvalidInput(format!(
                "Position limit would be exceeded: {}",
                new_position
            )));
        }

        // Register open order
        self.open_orders.insert(order_id, quantity);

        Ok(())
    }

    pub fn update_position(&self, instrument: &str, side: Side, quantity: Quantity) {
        let mut position = self
            .positions
            .entry(instrument.to_string())
            .or_insert_with(|| Position {
                instrument: instrument.to_string(),
                quantity: 0,
            });

        match side {
            Side::Buy => position.quantity += quantity.0 as i64,
            Side::Sell => position.quantity -= quantity.0 as i64,
        }
    }

    pub fn remove_order(&self, order_id: OrderId) {
        self.open_orders.remove(&order_id);
    }
}

