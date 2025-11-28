//! Audit log for tracking order events

use crate::order::Order;
use fx_utils::{OrderId, OrderType, Side};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    OrderSubmitted,
    OrderFilled,
    OrderPartiallyFilled,
    OrderCancelled,
    OrderRejected,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub order_id: OrderId,
    pub instrument: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<u64>,
    pub timestamp_ns: u64,
    pub message: Option<String>,
}

impl AuditEvent {
    pub fn from_order(event_type: AuditEventType, order: &Order, message: Option<String>) -> Self {
        Self {
            event_type,
            order_id: order.id,
            instrument: order.instrument.clone(),
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity.0,
            price: order.price.map(|p| p.0),
            timestamp_ns: fx_utils::time::now_nanos(),
            message,
        }
    }
}

/// Audit log for tracking all order events
pub struct AuditLog {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    max_size: usize,
}

impl AuditLog {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add an audit event
    pub fn add_event(&self, event: AuditEvent) {
        let mut events = self.events.write();
        events.push(event);

        // Keep only the most recent events if we exceed max_size
        if events.len() > self.max_size {
            events.remove(0);
        }
    }

    /// Get all audit events
    pub fn get_events(&self) -> Vec<AuditEvent> {
        self.events.read().clone()
    }

    /// Get events for a specific order
    pub fn get_events_for_order(&self, order_id: OrderId) -> Vec<AuditEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.order_id == order_id)
            .cloned()
            .collect()
    }

    /// Get recent events (last N events)
    pub fn get_recent_events(&self, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read();
        let start = events.len().saturating_sub(limit);
        events[start..].to_vec()
    }
}
