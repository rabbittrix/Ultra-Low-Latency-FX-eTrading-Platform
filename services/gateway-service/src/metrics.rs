/**
 * Prometheus metrics for gateway service
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
    pub requests_total: IntCounter,
    #[allow(dead_code)]
    pub request_duration: Histogram,
    #[allow(dead_code)]
    pub websocket_connections: Gauge,
    #[allow(dead_code)]
    pub active_websocket_clients: Gauge,
    #[allow(dead_code)]
    pub backend_errors: IntCounter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let requests_total = IntCounter::new("gateway_requests_total", "Total number of requests")?;
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "gateway_request_duration_seconds",
                "Request duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )?;
        let websocket_connections = Gauge::new(
            "gateway_websocket_connections_total",
            "Total number of WebSocket connections",
        )?;
        let active_websocket_clients = Gauge::new(
            "gateway_active_websocket_clients",
            "Current number of active WebSocket clients",
        )?;
        let backend_errors = IntCounter::new(
            "gateway_backend_errors_total",
            "Total number of backend errors",
        )?;

        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(websocket_connections.clone()))?;
        registry.register(Box::new(active_websocket_clients.clone()))?;
        registry.register(Box::new(backend_errors.clone()))?;

        Ok(Self {
            requests_total,
            request_duration,
            websocket_connections,
            active_websocket_clients,
            backend_errors,
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
