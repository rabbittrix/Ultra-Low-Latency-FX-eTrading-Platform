/**
 * Prometheus metrics for risk service
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
    pub risk_checks: IntCounter,
    #[allow(dead_code)]
    pub risk_checks_passed: IntCounter,
    #[allow(dead_code)]
    pub risk_checks_failed: IntCounter,
    #[allow(dead_code)]
    pub risk_check_duration: Histogram,
    #[allow(dead_code)]
    pub total_positions: Gauge,
    #[allow(dead_code)]
    pub total_exposure: Gauge,
    #[allow(dead_code)]
    pub position_limit_utilization: Gauge,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let risk_checks =
            IntCounter::new("risk_checks_total", "Total number of risk checks performed")?;
        let risk_checks_passed = IntCounter::new(
            "risk_checks_passed_total",
            "Total number of risk checks that passed",
        )?;
        let risk_checks_failed = IntCounter::new(
            "risk_checks_failed_total",
            "Total number of risk checks that failed",
        )?;
        let risk_check_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "risk_check_duration_seconds",
                "Risk check duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        let total_positions = Gauge::new("risk_total_positions", "Total number of positions")?;
        let total_exposure = Gauge::new(
            "risk_total_exposure",
            "Total exposure across all instruments",
        )?;
        let position_limit_utilization = Gauge::new(
            "risk_position_limit_utilization",
            "Position limit utilization percentage",
        )?;

        registry.register(Box::new(risk_checks.clone()))?;
        registry.register(Box::new(risk_checks_passed.clone()))?;
        registry.register(Box::new(risk_checks_failed.clone()))?;
        registry.register(Box::new(risk_check_duration.clone()))?;
        registry.register(Box::new(total_positions.clone()))?;
        registry.register(Box::new(total_exposure.clone()))?;
        registry.register(Box::new(position_limit_utilization.clone()))?;

        Ok(Self {
            risk_checks,
            risk_checks_passed,
            risk_checks_failed,
            risk_check_duration,
            total_positions,
            total_exposure,
            position_limit_utilization,
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
