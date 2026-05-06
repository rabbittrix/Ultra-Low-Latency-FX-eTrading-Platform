use chrono::{DateTime, Utc};
use fx_utils::{OrderId, OrderType, Price, Quantity, Side};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FxProduct {
    Spot,
    Forward,
    Ndf,
    Swap,
    Option,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OmsOrderState {
    New,
    Validated,
    Routed,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmsOrder {
    pub order_id: OrderId,
    pub client_id: Uuid,
    pub instrument: String,
    pub product: FxProduct,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub limit_price: Option<Price>,
    pub time_in_force: TimeInForce,
    pub state: OmsOrderState,
    pub created_at: DateTime<Utc>,
    pub value_date: Option<DateTime<Utc>>,
}

impl OmsOrder {
    pub fn new(
        client_id: Uuid,
        instrument: String,
        product: FxProduct,
        side: Side,
        order_type: OrderType,
        quantity: Quantity,
        limit_price: Option<Price>,
        time_in_force: TimeInForce,
    ) -> Self {
        Self {
            order_id: Uuid::new_v4(),
            client_id,
            instrument,
            product,
            side,
            order_type,
            quantity,
            limit_price,
            time_in_force,
            state: OmsOrderState::New,
            created_at: Utc::now(),
            value_date: None,
        }
    }
}
