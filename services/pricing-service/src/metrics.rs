/**
 * Prometheus metrics for pricing service
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use prometheus::{Encoder, Histogram, IntCounter, Registry, TextEncoder};

pub struct Metrics {
    #[allow(dead_code)]
    pub price_calculations: IntCounter,
    #[allow(dead_code)]
    pub price_calculation_duration: Histogram,
    #[allow(dead_code)]
    pub ai_client_requests: IntCounter,
    #[allow(dead_code)]
    pub ai_client_errors: IntCounter,
    #[allow(dead_code)]
    pub risk_adjustments: IntCounter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let price_calculations = IntCounter::new(
            "pricing_calculations_total",
            "Total number of price calculations",
        )?;
        let price_calculation_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "pricing_calculation_duration_seconds",
                "Price calculation duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        let ai_client_requests = IntCounter::new(
            "pricing_ai_client_requests_total",
            "Total number of AI client requests",
        )?;
        let ai_client_errors = IntCounter::new(
            "pricing_ai_client_errors_total",
            "Total number of AI client errors",
        )?;
        let risk_adjustments = IntCounter::new(
            "pricing_risk_adjustments_total",
            "Total number of risk adjustments applied",
        )?;

        registry.register(Box::new(price_calculations.clone()))?;
        registry.register(Box::new(price_calculation_duration.clone()))?;
        registry.register(Box::new(ai_client_requests.clone()))?;
        registry.register(Box::new(ai_client_errors.clone()))?;
        registry.register(Box::new(risk_adjustments.clone()))?;

        Ok(Self {
            price_calculations,
            price_calculation_duration,
            ai_client_requests,
            ai_client_errors,
            risk_adjustments,
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
