use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use prometheus::{default_registry, Encoder, Histogram, HistogramOpts, IntCounter, TextEncoder};

pub struct Metrics {
    pub graph_recomputes: IntCounter,
    pub graph_recompute_time: Histogram,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = default_registry();
        let graph_recomputes =
            IntCounter::new("liquidity_graph_recomputes_total", "Graph mock recomputes")?;
        let graph_recompute_time = Histogram::with_opts(
            HistogramOpts::new(
                "liquidity_graph_recompute_seconds",
                "Time to rebuild liquidity graph",
            )
            .buckets(vec![0.000_05, 0.000_1, 0.000_5, 0.001, 0.005, 0.01]),
        )?;
        registry.register(Box::new(graph_recomputes.clone()))?;
        registry.register(Box::new(graph_recompute_time.clone()))?;
        Ok(Self {
            graph_recomputes,
            graph_recompute_time,
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
