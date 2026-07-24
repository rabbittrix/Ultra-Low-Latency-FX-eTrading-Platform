use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use prometheus::{default_registry, Encoder, Histogram, HistogramOpts, IntCounter, TextEncoder};

pub struct Metrics {
    pub risk_checks: IntCounter,
    pub risk_passed: IntCounter,
    pub risk_failed: IntCounter,
    pub risk_check_time: Histogram,
    pub graph_plan_time: Histogram,
    pub dispatch_parallel_time: Histogram,
    pub exec_latency: Histogram,
    pub exec_success: IntCounter,
    pub exec_failures: IntCounter,
    pub ai_calls: IntCounter,
    pub ai_failures: IntCounter,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = default_registry();
        let risk_checks = IntCounter::new("exec_risk_checks_total", "Fast-path risk checks")?;
        let risk_passed = IntCounter::new("exec_risk_passed_total", "Risk checks passed")?;
        let risk_failed = IntCounter::new("exec_risk_failed_total", "Risk checks failed")?;
        let risk_check_time = Histogram::with_opts(
            HistogramOpts::new("exec_risk_check_seconds", "Risk check duration")
                .buckets(vec![0.000_01, 0.000_05, 0.000_1, 0.000_5, 0.001]),
        )?;
        let graph_plan_time = Histogram::with_opts(
            HistogramOpts::new("exec_graph_plan_seconds", "Graph planning duration")
                .buckets(vec![0.000_05, 0.000_1, 0.000_5, 0.001, 0.005]),
        )?;
        let dispatch_parallel_time = Histogram::with_opts(
            HistogramOpts::new("exec_dispatch_parallel_seconds", "Parallel venue dispatch")
                .buckets(vec![0.000_1, 0.000_5, 0.001, 0.005, 0.01]),
        )?;
        let exec_latency = Histogram::with_opts(
            HistogramOpts::new(
                "exec_end_to_end_seconds",
                "End-to-end execution latency (client request to response)",
            )
            .buckets(vec![0.000_1, 0.000_5, 0.001, 0.002, 0.005, 0.01, 0.05]),
        )?;
        let exec_success = IntCounter::new("exec_success_total", "Successful executions")?;
        let exec_failures = IntCounter::new("exec_failure_total", "Failed executions")?;
        let ai_calls = IntCounter::new("exec_ai_calls_total", "AI inference calls")?;
        let ai_failures = IntCounter::new("exec_ai_failures_total", "AI inference failures")?;

        registry.register(Box::new(risk_checks.clone()))?;
        registry.register(Box::new(risk_passed.clone()))?;
        registry.register(Box::new(risk_failed.clone()))?;
        registry.register(Box::new(exec_success.clone()))?;
        registry.register(Box::new(exec_failures.clone()))?;
        registry.register(Box::new(ai_calls.clone()))?;
        registry.register(Box::new(ai_failures.clone()))?;
        registry.register(Box::new(risk_check_time.clone()))?;
        registry.register(Box::new(graph_plan_time.clone()))?;
        registry.register(Box::new(dispatch_parallel_time.clone()))?;
        registry.register(Box::new(exec_latency.clone()))?;

        Ok(Self {
            risk_checks,
            risk_passed,
            risk_failed,
            risk_check_time,
            graph_plan_time,
            dispatch_parallel_time,
            exec_latency,
            exec_success,
            exec_failures,
            ai_calls,
            ai_failures,
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
            .body(Body::from(format!("encode: {}", e)))
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}
