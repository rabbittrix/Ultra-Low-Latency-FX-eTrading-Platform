use crate::model::{OmsOrder, OmsOrderState};
use dashmap::DashMap;
use fx_utils::{OrderId, Result};

#[derive(Default)]
pub struct OmsEngine {
    orders: DashMap<OrderId, OmsOrder>,
}

impl OmsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_order(&self, mut order: OmsOrder) -> Result<OrderId> {
        order.state = OmsOrderState::Validated;
        let order_id = order.order_id;
        self.orders.insert(order_id, order);
        Ok(order_id)
    }

    pub fn mark_routed(&self, order_id: OrderId) -> Result<()> {
        let mut order = self.orders.get_mut(&order_id).ok_or_else(|| {
            fx_utils::Error::InvalidInput(format!("Order not found: {}", order_id))
        })?;
        order.state = OmsOrderState::Routed;
        Ok(())
    }

    pub fn mark_filled(&self, order_id: OrderId) -> Result<()> {
        let mut order = self.orders.get_mut(&order_id).ok_or_else(|| {
            fx_utils::Error::InvalidInput(format!("Order not found: {}", order_id))
        })?;
        order.state = OmsOrderState::Filled;
        Ok(())
    }

    pub fn get_order(&self, order_id: OrderId) -> Option<OmsOrder> {
        self.orders.get(&order_id).map(|o| o.clone())
    }
}
