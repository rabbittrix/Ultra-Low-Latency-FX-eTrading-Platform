/**
 * Prometheus metrics for router service
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

pub struct Metrics {
    #[allow(dead_code)]
    pub orders_routed: IntCounter,
    #[allow(dead_code)]
    pub routing_duration: Histogram,
    #[allow(dead_code)]
    pub venue_errors: IntCounter,
    #[allow(dead_code)]
    pub active_venues: Gauge,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let orders_routed = IntCounter::new(
            "router_orders_routed_total",
            "Total number of orders routed",
        )?;
        let routing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "router_routing_duration_seconds",
                "Order routing duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        let venue_errors =
            IntCounter::new("router_venue_errors_total", "Total number of venue errors")?;
        let active_venues = Gauge::new("router_active_venues", "Current number of active venues")?;

        registry.register(Box::new(orders_routed.clone()))?;
        registry.register(Box::new(routing_duration.clone()))?;
        registry.register(Box::new(venue_errors.clone()))?;
        registry.register(Box::new(active_venues.clone()))?;

        Ok(Self {
            orders_routed,
            routing_duration,
            venue_errors,
            active_venues,
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
