//! HTTP handlers for the matching engine service

use axum::extract::State;
use axum::response::Json;
use fx_core::{MatchingEngine, Order};
use fx_utils::{OrderType, Price, Quantity, Side};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitOrderRequest {
    pub instrument: String,
    pub side: String,       // "Buy" or "Sell"
    pub order_type: String, // "Market", "Limit", "Stop", "IoC", "FoK"
    pub quantity: u64,
    pub price: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitOrderResponse {
    pub success: bool,
    pub message: String,
    pub order_id: String,
    pub trades: Vec<TradeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResponse {
    pub trade_id: String,
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub instrument: String,
    pub quantity: u64,
    pub price: u64,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTradesResponse {
    pub trades: Vec<TradeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAuditEventsResponse {
    pub events: Vec<AuditEventResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventResponse {
    pub event_type: String,
    pub order_id: String,
    pub instrument: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: Option<u64>,
    pub timestamp_ns: u64,
    pub message: Option<String>,
}

pub async fn submit_order(
    State(engine): State<Arc<parking_lot::Mutex<MatchingEngine>>>,
    Json(req): Json<SubmitOrderRequest>,
) -> Json<SubmitOrderResponse> {
    let order_id = Uuid::new_v4();
    let side = match req.side.as_str() {
        "Buy" => Side::Buy,
        "Sell" => Side::Sell,
        _ => {
            return Json(SubmitOrderResponse {
                success: false,
                message: format!("Invalid side: {}", req.side),
                order_id: order_id.to_string(),
                trades: vec![],
            });
        }
    };

    let order_type = match req.order_type.as_str() {
        "Market" => OrderType::Market,
        "Limit" => OrderType::Limit,
        "Stop" => OrderType::Stop,
        "IoC" => OrderType::IoC,
        "FoK" => OrderType::FoK,
        _ => {
            return Json(SubmitOrderResponse {
                success: false,
                message: format!("Invalid order type: {}", req.order_type),
                order_id: order_id.to_string(),
                trades: vec![],
            });
        }
    };

    let price = req.price.map(Price);
    let order = Arc::new(Order::new(
        order_id,
        req.instrument,
        side,
        order_type,
        Quantity(req.quantity),
        price,
    ));

    let mut engine_guard = engine.lock();
    let match_result = engine_guard.match_order(order);

    let trades: Vec<TradeResponse> = match_result
        .trades
        .iter()
        .map(|t| TradeResponse {
            trade_id: t.id.to_string(),
            buy_order_id: t.buy_order_id.to_string(),
            sell_order_id: t.sell_order_id.to_string(),
            instrument: t.instrument.clone(),
            quantity: t.quantity.0,
            price: t.price.0,
            timestamp_ns: t.timestamp_ns,
        })
        .collect();

    Json(SubmitOrderResponse {
        success: true,
        message: if trades.is_empty() {
            "Order placed".to_string()
        } else {
            format!("Order matched with {} trades", trades.len())
        },
        order_id: match_result.order.id.to_string(),
        trades,
    })
}

pub async fn cancel_order(
    State(engine): State<Arc<parking_lot::Mutex<MatchingEngine>>>,
    Json(req): Json<CancelOrderRequest>,
) -> Json<CancelOrderResponse> {
    let order_id = match Uuid::parse_str(&req.order_id) {
        Ok(id) => id,
        Err(_) => {
            return Json(CancelOrderResponse {
                success: false,
                message: format!("Invalid order ID: {}", req.order_id),
            });
        }
    };

    let mut engine_guard = engine.lock();
    let success = engine_guard.cancel_order(order_id);

    Json(CancelOrderResponse {
        success,
        message: if success {
            "Order cancelled".to_string()
        } else {
            "Order not found".to_string()
        },
    })
}

pub async fn get_trades(
    State(engine): State<Arc<parking_lot::Mutex<MatchingEngine>>>,
) -> Json<GetTradesResponse> {
    let engine_guard = engine.lock();
    let trade_log = engine_guard.trade_log();
    let trades = trade_log.get_trades();

    let trade_responses: Vec<TradeResponse> = trades
        .iter()
        .map(|t| TradeResponse {
            trade_id: t.id.to_string(),
            buy_order_id: t.buy_order_id.to_string(),
            sell_order_id: t.sell_order_id.to_string(),
            instrument: t.instrument.clone(),
            quantity: t.quantity.0,
            price: t.price.0,
            timestamp_ns: t.timestamp_ns,
        })
        .collect();

    Json(GetTradesResponse {
        trades: trade_responses,
    })
}

pub async fn get_audit_events(
    State(engine): State<Arc<parking_lot::Mutex<MatchingEngine>>>,
) -> Json<GetAuditEventsResponse> {
    let engine_guard = engine.lock();
    let audit_log = engine_guard.audit_log();
    let events = audit_log.get_events();

    let event_responses: Vec<AuditEventResponse> = events
        .iter()
        .map(|e| AuditEventResponse {
            event_type: format!("{:?}", e.event_type),
            order_id: e.order_id.to_string(),
            instrument: e.instrument.clone(),
            side: format!("{:?}", e.side),
            order_type: format!("{:?}", e.order_type),
            quantity: e.quantity,
            price: e.price,
            timestamp_ns: e.timestamp_ns,
            message: e.message.clone(),
        })
        .collect();

    Json(GetAuditEventsResponse {
        events: event_responses,
    })
}
