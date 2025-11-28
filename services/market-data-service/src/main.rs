//! Market Data Service
//!
//! Ingests FX market data feeds, normalizes quotes, and publishes
//! L2 + L3 order books via WebSocket and REST APIs.

use axum::{
    body::Body,
    extract::{State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{Json, Response},
    routing::get,
    Router,
};
use fx_md::{MarketDataFeed, Quote};
use fx_utils::{Price, Quantity, Result};
use prometheus::{Encoder, Gauge, IntCounter, Registry, TextEncoder};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, Level};

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    feed: Arc<MarketDataFeed>,
    latest_quote: Arc<RwLock<Option<Quote>>>,
    quote_rx: Arc<broadcast::Receiver<Quote>>,
    metrics: Arc<Metrics>,
}

/// Prometheus metrics for market data service
struct Metrics {
    quotes_published: IntCounter,
    websocket_connections: Gauge,
    active_subscribers: Gauge,
    quote_latency_ns: Gauge,
}

impl Metrics {
    fn new(registry: &Registry) -> Result<Self> {
        let quotes_published = IntCounter::new(
            "market_data_quotes_published_total",
            "Total number of quotes published",
        )
        .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        let websocket_connections = Gauge::new(
            "market_data_websocket_connections",
            "Current number of WebSocket connections",
        )
        .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        let active_subscribers = Gauge::new(
            "market_data_active_subscribers",
            "Current number of active subscribers",
        )
        .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        let quote_latency_ns = Gauge::new(
            "market_data_quote_latency_ns",
            "Quote processing latency in nanoseconds",
        )
        .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;

        registry
            .register(Box::new(quotes_published.clone()))
            .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        registry
            .register(Box::new(websocket_connections.clone()))
            .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        registry
            .register(Box::new(active_subscribers.clone()))
            .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;
        registry
            .register(Box::new(quote_latency_ns.clone()))
            .map_err(|e| fx_utils::Error::Prometheus(e.to_string()))?;

        Ok(Self {
            quotes_published,
            websocket_connections,
            active_subscribers,
            quote_latency_ns,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Market Data Service");

    // Initialize Prometheus registry
    let registry = Registry::new();
    let metrics = Arc::new(Metrics::new(&registry)?);

    // Initialize market data feed
    let (feed, quote_rx) = MarketDataFeed::new("EURUSD".to_string());
    let feed = Arc::new(feed);
    let quote_rx = Arc::new(quote_rx);
    let latest_quote = Arc::new(RwLock::new(None));

    // Start quote subscriber to update latest quote and metrics
    let latest_quote_clone = latest_quote.clone();
    let metrics_clone = metrics.clone();
    let feed_for_metrics = feed.clone();
    let mut quote_rx_subscriber = quote_rx.resubscribe();
    tokio::spawn(async move {
        while let Ok(quote) = quote_rx_subscriber.recv().await {
            let publish_time = fx_utils::time::now_nanos();
            let latency = publish_time.saturating_sub(quote.timestamp_ns);

            *latest_quote_clone.write().await = Some(quote.clone());
            metrics_clone.quotes_published.inc();
            metrics_clone.quote_latency_ns.set(latency as f64);
            metrics_clone
                .active_subscribers
                .set(feed_for_metrics.receiver_count() as f64);
        }
    });

    // Start mock feed generator
    let feed_clone = feed.clone();
    let metrics_feed = metrics.clone();
    tokio::spawn(async move {
        generate_mock_feed(feed_clone, metrics_feed).await;
    });

    // Setup HTTP server
    let state = AppState {
        feed,
        latest_quote,
        quote_rx,
        metrics,
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .route("/quote", get(get_latest_quote))
        .route("/ws", get(websocket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .map_err(fx_utils::Error::Io)?;

    info!("Market Data Service listening on http://0.0.0.0:8081");
    info!("WebSocket endpoint available at ws://0.0.0.0:8081/ws");
    axum::serve(listener, app)
        .await
        .map_err(|e| fx_utils::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy", "service": "market-data-service" }))
}

async fn metrics_handler() -> Response<Body> {
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

async fn get_latest_quote(State(state): State<AppState>) -> Json<serde_json::Value> {
    let quote_guard = state.latest_quote.read().await;
    if let Some(quote) = quote_guard.as_ref() {
        Json(json!({
            "instrument": quote.instrument,
            "bid": quote.bid_price.to_decimal(4),
            "ask": quote.ask_price.to_decimal(4),
            "bid_size": quote.bid_size.0,
            "ask_size": quote.ask_size.0,
            "spread": quote.spread(),
            "mid_price": quote.mid_price().to_decimal(4),
            "timestamp": quote.timestamp_ns
        }))
    } else {
        Json(json!({
            "instrument": "EURUSD",
            "bid": 1.0850,
            "ask": 1.0852,
            "bid_size": 1000000,
            "ask_size": 1000000,
            "spread": 2,
            "mid_price": 1.0851,
            "timestamp": fx_utils::time::now_nanos(),
            "note": "No quotes received yet, returning default values"
        }))
    }
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response<Body> {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut quote_rx = state.quote_rx.resubscribe();

    state.metrics.websocket_connections.inc();
    state.metrics.active_subscribers.inc();

    // Spawn task to send quotes to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(quote) = quote_rx.recv().await {
            let quote_json = json!({
                "instrument": quote.instrument,
                "bid": quote.bid_price.to_decimal(4),
                "ask": quote.ask_price.to_decimal(4),
                "bid_size": quote.bid_size.0,
                "ask_size": quote.ask_size.0,
                "spread": quote.spread(),
                "mid_price": quote.mid_price().to_decimal(4),
                "timestamp": quote.timestamp_ns
            });

            if let Err(e) = sender
                .send(Message::Text(serde_json::to_string(&quote_json).unwrap()))
                .await
            {
                tracing::error!(error = %e, "Failed to send WebSocket message");
                break;
            }
        }
    });

    // Spawn task to receive messages from client (ping/pong, close, etc.)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    break;
                }
                Message::Ping(_) => {
                    // Ping is handled automatically by axum, just continue
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    state.metrics.websocket_connections.dec();
    state.metrics.active_subscribers.dec();
}

async fn generate_mock_feed(feed: Arc<MarketDataFeed>, _metrics: Arc<Metrics>) {
    let mut counter = 0u64;
    let instruments = ["EURUSD", "GBPUSD", "USDJPY", "AUDUSD"];
    let base_prices = [1.0850, 1.2650, 150.50, 0.6550];

    loop {
        for (idx, instrument) in instruments.iter().enumerate() {
            let base_price = base_prices[idx];
            let variation = (counter as f64 * 0.0001 + idx as f64).sin() * 0.001;
            let current_price = base_price + variation;

            let quote = Quote {
                instrument: instrument.to_string(),
                bid_price: Price::from_decimal(current_price - 0.0001, 4),
                ask_price: Price::from_decimal(current_price + 0.0001, 4),
                bid_size: Quantity(1_000_000),
                ask_size: Quantity(1_000_000),
                timestamp_ns: fx_utils::time::now_nanos(),
            };

            if let Err(e) = feed.publish(quote) {
                tracing::error!(error = %e, "Failed to publish quote");
            }
        }

        counter += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
