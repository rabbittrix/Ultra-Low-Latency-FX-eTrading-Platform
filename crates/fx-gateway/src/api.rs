//! Gateway API definitions

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health,
        crate::handlers::root,
        crate::openapi_proxy::liquidity_graph_snapshot,
        crate::openapi_proxy::liquidity_graph_recompute,
        crate::openapi_proxy::liquidity_plan,
        crate::openapi_proxy::execution_v1_execute,
    ),
    components(schemas(
        HealthResponse,
        GatewayInfo,
        // Market Data schemas
        QuoteResponse,
        // Pricing schemas
        PriceQuoteRequest,
        PriceQuoteResponse,
        // Matching Engine schemas
        SubmitOrderRequest,
        SubmitOrderResponse,
        CancelOrderRequest,
        CancelOrderResponse,
        TradeResponse,
        GetTradesResponse,
        AuditEventResponse,
        GetAuditEventsResponse,
        // Risk schemas
        CheckOrderRequest,
        CheckOrderResponse,
        PositionResponse,
        InstrumentExposure,
        ExposureSummary,
        RiskLimitsInfo,
        // Liquidity graph + execution (proxied on gateway 8080)
        fx_liquidity_graph::LiquidityGraph,
        fx_liquidity_graph::LiquidityNode,
        fx_liquidity_graph::LiquidityEdge,
        fx_liquidity_graph::VenueClass,
        fx_liquidity_graph::VenueAllocation,
        fx_liquidity_graph::SliceStrategy,
        fx_liquidity_graph::ExecutionPlan,
        crate::proxy_types::LiquidityPlanRequestBody,
        crate::proxy_types::ExecutionSubmitRequest,
        crate::proxy_types::ExecutionFillLeg,
        crate::proxy_types::ExecutionSubmitResponse,
    )),
    tags(
        (name = "gateway", description = "FX eTrading Gateway API"),
        (name = "market-data", description = "Market Data Service endpoints (Port 8081)"),
        (name = "pricing", description = "Pricing Engine endpoints (Port 8082)"),
        (name = "trading", description = "Matching Engine endpoints (Port 8083)"),
        (name = "risk", description = "Risk Management endpoints (Port 8084)"),
        (name = "liquidity", description = "Global liquidity graph (proxied → port 8091)"),
        (name = "execution", description = "AI-assisted execution engine (proxied → port 8092)"),
    ),
    info(
        title = "FX eTrading Platform API",
        description = "Ultra-Low-Latency FX eTrading Platform REST API Documentation

## Service Endpoints

### Gateway Service (Port 8080)
- `GET /health` - Gateway health check
- `GET /` - Gateway information
- `GET /ws` - WebSocket stream for aggregated real-time data
- `GET /docs` - Swagger UI
- `GET /api-docs/openapi.json` - OpenAPI specification

### Market Data Service (Port 8081)
- `GET /health` - Service health check
- `GET /quote` - Get latest market quote
- `GET /metrics` - Prometheus metrics endpoint
- `GET /ws` - WebSocket stream for real-time market data
- **Yahoo Finance Integration**: Set `USE_YAHOO_FINANCE=true` for real market data (delayed 15-20 min on free tier)

### Pricing Service (Port 8082)
- `GET /health` - Service health check
- `POST /prices` - Calculate risk-adjusted prices
  - Request body: `{ instrument, bid_price, ask_price, bid_size, ask_size }`
  - Response: `{ instrument, bid_price, ask_price, mid_price, spread }`
- `GET /ws` - WebSocket stream for pricing updates

### Matching Engine Service (Port 8083)
- `GET /health` - Service health check
- `POST /orders` - Submit order
  - Request body: `{ instrument, side, order_type, quantity, price? }`
  - Response: `{ success, message, order_id, trades[] }`
- `POST /orders/cancel` - Cancel order
  - Request body: `{ order_id }`
  - Response: `{ success, message, order_id }`
- `GET /trades` - Get all executed trades
  - Response: `{ trades[] }`
- `GET /audit` - Get audit events
  - Response: `{ events[] }`
- **gRPC**: `0.0.0.0:50051` - gRPC service for high-performance order submission

### Risk Service (Port 8084)
- `GET /health` - Service health check
- `POST /check` - Check order risk
  - Request body: `{ instrument, side, quantity, order_id }`
  - Response: `{ success, message }`
- `GET /position/:instrument` - Get position for instrument
  - Response: `{ instrument, position }`
- `GET /exposure` - Get exposure summary
  - Response: `{ total_instruments, total_open_orders, total_exposure, instruments[], risk_limits }`
- `GET /exposure/:instrument` - Get exposure for specific instrument
  - Response: `{ instrument, position, position_abs, position_utilization, open_orders_count }`

### Router Service (Port 8085)
- `GET /health` - Service health check

### Liquidity Graph Service (Port 8091, via gateway `/liquidity`)
- `GET /liquidity/v1/graph/snapshot` - Full graph JSON
- `POST /liquidity/v1/graph/recompute` - Refresh mock graph
- `POST /liquidity/v1/plan` - Request execution plan (`instrument`, `side`, `quantity`)

### Execution Engine (Port 8092, via gateway `/execution`)
- `POST /execution/v1/execute` - Run pipeline (risk stub → AI venue scores → plan → mock fills)

### Python ML Service (Port 8086)
- `GET /health` - Service health check
- `POST /predict/volatility` - Predict volatility
  - Request body: `{ instrument, historical_data? }`
  - Response: `{ volatility, confidence }`

## WebSocket Protocol

The gateway WebSocket endpoint (`ws://gateway:8080/ws`) aggregates messages from all backend services:

### Message Types:
- **MarketData**: Real-time market quotes
- **Pricing**: Risk-adjusted pricing updates
- **Trade**: Trade execution notifications
- **Exposure**: Position and exposure updates
- **Error**: Error messages

### Client Messages:
- `{ type: \"subscribe\", messageType: \"MarketData\" }` - Subscribe to message type
- `{ type: \"unsubscribe\", messageType: \"MarketData\" }` - Unsubscribe from message type
- `{ type: \"ping\" }` - Keep-alive ping

## Authentication

Currently, the API does not require authentication. In production, implement:
- API keys for REST endpoints
- JWT tokens for WebSocket connections
- Rate limiting per client
- IP whitelisting for sensitive operations",
        version = "0.1.0",
        contact(
            name = "Roberto de Souza",
            email = "rabbittrix@hotmail.com"
        ),
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    )
)]
pub struct GatewayApi;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Gateway schemas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GatewayInfo {
    pub name: String,
    pub version: String,
}

// Market Data schemas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuoteResponse {
    pub instrument: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: u64,
    pub ask_size: u64,
    pub spread: u64,
    pub mid_price: f64,
    pub timestamp: u64,
}

// Pricing schemas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PriceQuoteRequest {
    pub instrument: String,
    pub bid_price: u64,
    pub ask_price: u64,
    pub bid_size: u64,
    pub ask_size: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PriceQuoteResponse {
    pub instrument: String,
    pub bid_price: u64,
    pub ask_price: u64,
    pub mid_price: u64,
    pub spread: u64,
}

// Matching Engine schemas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitOrderRequest {
    pub instrument: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitOrderResponse {
    pub success: bool,
    pub message: String,
    pub order_id: String,
    pub trades: Vec<TradeResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CancelOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CancelOrderResponse {
    pub success: bool,
    pub message: String,
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TradeResponse {
    pub trade_id: String,
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub instrument: String,
    pub quantity: u64,
    pub price: u64,
    pub timestamp_ns: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetTradesResponse {
    pub trades: Vec<TradeResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditEventResponse {
    pub event_type: String,
    pub order_id: String,
    pub instrument: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: Option<u64>,
    pub timestamp_ns: u64,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetAuditEventsResponse {
    pub events: Vec<AuditEventResponse>,
}

// Risk schemas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CheckOrderRequest {
    pub instrument: String,
    pub side: String,
    pub quantity: u64,
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CheckOrderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PositionResponse {
    pub instrument: String,
    pub position: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InstrumentExposure {
    pub instrument: String,
    pub position: i64,
    pub position_abs: u64,
    pub position_utilization: f64,
    pub open_orders_count: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RiskLimitsInfo {
    pub max_position_size: u64,
    pub max_order_size: u64,
    pub max_daily_loss: u64,
    pub max_open_orders: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExposureSummary {
    pub total_instruments: usize,
    pub total_open_orders: usize,
    pub total_exposure: u64,
    pub instruments: Vec<InstrumentExposure>,
    pub risk_limits: RiskLimitsInfo,
}

