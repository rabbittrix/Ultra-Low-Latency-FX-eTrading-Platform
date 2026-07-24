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

/// Construction inputs for [`OmsOrder::new`] (avoids a long positional argument list).
#[derive(Debug, Clone)]
pub struct NewOmsOrder {
    pub client_id: Uuid,
    pub instrument: String,
    pub product: FxProduct,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub limit_price: Option<Price>,
    pub time_in_force: TimeInForce,
}

impl OmsOrder {
    pub fn new(params: NewOmsOrder) -> Self {
        Self {
            order_id: Uuid::new_v4(),
            client_id: params.client_id,
            instrument: params.instrument,
            product: params.product,
            side: params.side,
            order_type: params.order_type,
            quantity: params.quantity,
            limit_price: params.limit_price,
            time_in_force: params.time_in_force,
            state: OmsOrderState::New,
            created_at: Utc::now(),
            value_date: None,
        }
    }
}
