//! Direct in-app observability: scrape loopback `/health` + `/metrics`, keep ring history.
//! No Prometheus / Grafana required.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::State;

const HTTP_TIMEOUT: Duration = Duration::from_secs(2);
const RING_CAP: usize = 180; // ~15m at 5s poll

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProbe {
    pub id: String,
    pub name: String,
    pub url: String,
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: u64,
    pub error: Option<String>,
    /// Whether this service is expected in the default `dev:stack`.
    pub expected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartPoint {
    pub t: i64,
    pub v: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartSeries {
    pub id: String,
    pub title: String,
    pub unit: String,
    pub points: Vec<ChartPoint>,
    pub latest: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObsSnapshot {
    pub probes: Vec<ServiceProbe>,
    pub charts: Vec<ChartSeries>,
    pub collected_at_ns: i64,
    pub source: String,
}

struct Target {
    id: &'static str,
    name: &'static str,
    health: &'static str,
    metrics: Option<&'static str>,
    expected: bool,
}

const TARGETS: &[Target] = &[
    Target {
        id: "gateway",
        name: "Gateway",
        health: "http://127.0.0.1:8080/health",
        metrics: Some("http://127.0.0.1:8080/metrics"),
        expected: true,
    },
    Target {
        id: "matching",
        name: "Matching Engine",
        health: "http://127.0.0.1:8083/health",
        metrics: Some("http://127.0.0.1:8083/metrics"),
        expected: true,
    },
    Target {
        id: "risk",
        name: "Risk",
        health: "http://127.0.0.1:8084/health",
        metrics: Some("http://127.0.0.1:8084/metrics"),
        expected: true,
    },
    Target {
        id: "liquidity",
        name: "Liquidity Graph",
        health: "http://127.0.0.1:8091/health",
        metrics: Some("http://127.0.0.1:8091/metrics"),
        expected: true,
    },
    Target {
        id: "execution",
        name: "Execution Engine",
        health: "http://127.0.0.1:8092/health",
        metrics: Some("http://127.0.0.1:8092/metrics"),
        expected: true,
    },
    Target {
        id: "smc",
        name: "fx-smc Advisory",
        health: "http://127.0.0.1:8094/health",
        metrics: None,
        expected: true,
    },
    Target {
        id: "market-data",
        name: "Market Data",
        health: "http://127.0.0.1:8081/health",
        metrics: Some("http://127.0.0.1:8081/metrics"),
        expected: false,
    },
    Target {
        id: "pricing",
        name: "Pricing",
        health: "http://127.0.0.1:8082/health",
        metrics: Some("http://127.0.0.1:8082/metrics"),
        expected: false,
    },
];

struct ChartDef {
    id: &'static str,
    title: &'static str,
    unit: &'static str,
    /// Metric name in Prometheus text exposition (prefix match).
    metric: &'static str,
    /// true = treat as counter (rate from delta); false = gauge (raw).
    is_counter: bool,
}

const CHARTS: &[ChartDef] = &[
    ChartDef {
        id: "gateway-rps",
        title: "Gateway requests (rate)",
        unit: "/s",
        metric: "gateway_requests_total",
        is_counter: true,
    },
    ChartDef {
        id: "gateway-ws",
        title: "Gateway WebSocket clients",
        unit: "",
        metric: "gateway_active_websocket_clients",
        is_counter: false,
    },
    ChartDef {
        id: "matching-orders",
        title: "Matching orders submitted (rate)",
        unit: "/s",
        metric: "matching_engine_orders_submitted_total",
        is_counter: true,
    },
    ChartDef {
        id: "matching-trades",
        title: "Matching trades executed (rate)",
        unit: "/s",
        metric: "matching_engine_trades_executed_total",
        is_counter: true,
    },
    ChartDef {
        id: "risk-checks",
        title: "Risk checks (rate)",
        unit: "/s",
        metric: "risk_checks_total",
        is_counter: true,
    },
    ChartDef {
        id: "exec-success",
        title: "Execution success (rate)",
        unit: "/s",
        metric: "exec_success_total",
        is_counter: true,
    },
    ChartDef {
        id: "liq-recompute",
        title: "Liquidity graph recomputes (rate)",
        unit: "/s",
        metric: "liquidity_graph_recomputes_total",
        is_counter: true,
    },
];

#[derive(Default)]
struct SeriesRing {
    points: VecDeque<ChartPoint>,
    last_raw: Option<(i64, f64)>,
}

pub struct ObsCollector {
    inner: Mutex<CollectorInner>,
}

struct CollectorInner {
    series: HashMap<String, SeriesRing>,
    client: reqwest::Client,
}

impl ObsCollector {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let mut series = HashMap::new();
        for c in CHARTS {
            series.insert(c.id.to_string(), SeriesRing::default());
        }
        // latency series per expected service
        for t in TARGETS.iter().filter(|t| t.expected) {
            series.insert(format!("lat-{}", t.id), SeriesRing::default());
        }
        Ok(Self {
            inner: Mutex::new(CollectorInner { series, client }),
        })
    }
}

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn millis_since(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Parse a Prometheus text exposition counter/gauge sample by metric name prefix.
fn parse_metric_value(body: &str, metric: &str) -> Option<f64> {
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !(line.starts_with(metric)
            && (line.as_bytes().get(metric.len()) == Some(&b'{')
                || line.as_bytes().get(metric.len()) == Some(&b' ')
                || line.as_bytes().get(metric.len()) == Some(&b'\t')))
        {
            continue;
        }
        let Some(val_str) = line.split_whitespace().last() else {
            continue;
        };
        if let Ok(v) = val_str.parse::<f64>() {
            return Some(v);
        }
    }
    None
}

fn push_point(ring: &mut SeriesRing, t: i64, v: f64) {
    ring.points.push_back(ChartPoint { t, v });
    while ring.points.len() > RING_CAP {
        ring.points.pop_front();
    }
}

fn push_rate_or_gauge(ring: &mut SeriesRing, t: i64, raw: f64, is_counter: bool) {
    if is_counter {
        if let Some((prev_t, prev_v)) = ring.last_raw {
            let dt = (t - prev_t).max(1) as f64;
            let dv = (raw - prev_v).max(0.0);
            push_point(ring, t, dv / dt);
        }
        // first sample seeds only
        ring.last_raw = Some((t, raw));
    } else {
        ring.last_raw = Some((t, raw));
        push_point(ring, t, raw);
    }
}

/// One collect cycle: probe health, scrape `/metrics`, update rings, return snapshot.
#[tauri::command]
pub async fn obs_collect(state: State<'_, ObsCollector>) -> Result<ObsSnapshot, String> {
    let client = {
        let g = state
            .inner
            .lock()
            .map_err(|_| "observability lock poisoned".to_string())?;
        g.client.clone()
    };

    let t = now_secs();
    let mut probes = Vec::with_capacity(TARGETS.len());
    let mut scraped: HashMap<String, f64> = HashMap::new();
    let mut latencies: HashMap<String, f64> = HashMap::new();

    for target in TARGETS {
        let start = Instant::now();
        let probe = match client.get(target.health).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let ok = resp.status().is_success();
                ServiceProbe {
                    id: target.id.into(),
                    name: target.name.into(),
                    url: target.health.into(),
                    ok,
                    status: Some(status),
                    latency_ms: millis_since(start),
                    error: if ok {
                        None
                    } else {
                        Some(format!("HTTP {status}"))
                    },
                    expected: target.expected,
                }
            }
            Err(e) => ServiceProbe {
                id: target.id.into(),
                name: target.name.into(),
                url: target.health.into(),
                ok: false,
                status: None,
                latency_ms: millis_since(start),
                error: Some(short_err(&e.to_string())),
                expected: target.expected,
            },
        };
        if target.expected {
            latencies.insert(target.id.into(), probe.latency_ms as f64);
        }
        probes.push(probe);

        if let Some(metrics_url) = target.metrics {
            if let Ok(resp) = client.get(metrics_url).send().await {
                if resp.status().is_success() {
                    if let Ok(body) = resp.text().await {
                        for c in CHARTS {
                            if let Some(v) = parse_metric_value(&body, c.metric) {
                                scraped.insert(c.id.to_string(), v);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut charts = Vec::new();
    {
        let mut g = state
            .inner
            .lock()
            .map_err(|_| "observability lock poisoned".to_string())?;

        for c in CHARTS {
            let ring = g
                .series
                .entry(c.id.to_string())
                .or_insert_with(SeriesRing::default);
            let note = if let Some(raw) = scraped.get(c.id) {
                push_rate_or_gauge(ring, t, *raw, c.is_counter);
                None
            } else {
                Some("metric not published by a reachable service yet".into())
            };
            let latest = ring.points.back().map(|p| p.v);
            charts.push(ChartSeries {
                id: c.id.into(),
                title: c.title.into(),
                unit: c.unit.into(),
                points: ring.points.iter().cloned().collect(),
                latest,
                note,
            });
        }

        for (id, lat) in &latencies {
            let key = format!("lat-{id}");
            let ring = g.series.entry(key.clone()).or_insert_with(SeriesRing::default);
            push_point(ring, t, *lat);
            let name = TARGETS
                .iter()
                .find(|x| x.id == id.as_str())
                .map(|x| x.name)
                .unwrap_or(id.as_str());
            charts.push(ChartSeries {
                id: key,
                title: format!("{name} health latency"),
                unit: "ms".into(),
                points: ring.points.iter().cloned().collect(),
                latest: ring.points.back().map(|p| p.v),
                note: None,
            });
        }
    }

    // Put latency charts after main metric charts is fine; UI can group.
    // Prefer metric charts first already done; latencies appended.

    Ok(ObsSnapshot {
        probes,
        charts,
        collected_at_ns: now_ns(),
        source: "tauri-direct".into(),
    })
}

fn short_err(s: &str) -> String {
    // Drop noisy reqwest URL tails for the UI.
    if s.contains("error sending request") {
        "unreachable".into()
    } else if s.len() > 120 {
        format!("{}…", &s[..117])
    } else {
        s.to_string()
    }
}
