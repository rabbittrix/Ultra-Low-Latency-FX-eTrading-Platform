/**
 * Prometheus metrics for matching engine service
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use prometheus::{Encoder, Gauge, Histogram, IntCounter, Registry, TextEncoder};
use std::sync::Arc;

pub struct Metrics {
    pub orders_submitted: IntCounter,
    pub orders_cancelled: IntCounter,
    pub orders_rejected: IntCounter,
    pub trades_executed: IntCounter,
    pub order_matching_duration: Histogram,
    pub orderbook_depth: Gauge,
    pub active_orders: Gauge,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let orders_submitted = IntCounter::new(
            "matching_engine_orders_submitted_total",
            "Total number of orders submitted",
        )?;
        let orders_cancelled = IntCounter::new(
            "matching_engine_orders_cancelled_total",
            "Total number of orders cancelled",
        )?;
        let orders_rejected = IntCounter::new(
            "matching_engine_orders_rejected_total",
            "Total number of orders rejected",
        )?;
        let trades_executed = IntCounter::new(
            "matching_engine_trades_executed_total",
            "Total number of trades executed",
        )?;
        let order_matching_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "matching_engine_order_matching_duration_seconds",
                "Order matching duration in seconds",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        )?;
        let orderbook_depth = Gauge::new(
            "matching_engine_orderbook_depth",
            "Current order book depth",
        )?;
        let active_orders = Gauge::new(
            "matching_engine_active_orders",
            "Current number of active orders",
        )?;

        registry.register(Box::new(orders_submitted.clone()))?;
        registry.register(Box::new(orders_cancelled.clone()))?;
        registry.register(Box::new(orders_rejected.clone()))?;
        registry.register(Box::new(trades_executed.clone()))?;
        registry.register(Box::new(order_matching_duration.clone()))?;
        registry.register(Box::new(orderbook_depth.clone()))?;
        registry.register(Box::new(active_orders.clone()))?;

        Ok(Self {
            orders_submitted,
            orders_cancelled,
            orders_rejected,
            trades_executed,
            order_matching_duration,
            orderbook_depth,
            active_orders,
        })
    }
}

pub async fn metrics_handler() -> Response {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Failed to encode metrics: {}", e)))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}
