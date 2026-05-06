# API Reference

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0

## REST API

### Base URL

- **Development**: <http://localhost:8080>
- **Production**: Configure via environment variables

### Authentication

Currently, the API does not require authentication. For production deployments, implement JWT-based authentication.

### Endpoints

#### Gateway Service

- **Health Check**: `GET /health`
- **API Info**: `GET /`
- **Swagger UI**: `GET /docs` (includes **liquidity** and **execution** tags for proxied routes)
- **OpenAPI Spec**: `GET /api-docs/openapi.json`
- **Metrics**: `GET /metrics`
- **WebSocket**: `WS /ws`
- **Liquidity graph (proxied)**: `GET|POST /liquidity/...` → `liquidity-graph-service:8091` (path after prefix forwarded)
- **Execution engine (proxied)**: `GET|POST /execution/...` → `execution-engine:8092`

Examples (via gateway):

- `GET http://localhost:8080/liquidity/v1/graph/snapshot`
- `POST http://localhost:8080/execution/v1/execute` with JSON body (see Execution Engine below)

Direct service URLs (local dev): `http://localhost:8091`, `http://localhost:8092`.

#### Market Data Service

- **Health Check**: `GET /health`
- **Latest Quote**: `GET /quote?instrument=EURUSD`
- **Metrics**: `GET /metrics`
- **WebSocket**: `WS /ws`

#### Matching Engine Service

- **Health Check**: `GET /health`
- **Submit Order**: `POST /orders`
- **Cancel Order**: `POST /orders/cancel`
- **Get Trades**: `GET /trades`
- **Get Audit Events**: `GET /audit`
- **Metrics**: `GET /metrics`

#### Pricing Service

- **Health Check**: `GET /health`
- **Calculate Prices**: `POST /prices`
- **Metrics**: `GET /metrics`
- **WebSocket**: `WS /ws`

#### Risk Service

- **Health Check**: `GET /health`
- **Check Order Risk**: `POST /check`
- **Get Position**: `GET /position/{instrument}`
- **Get Exposure Summary**: `GET /exposure`
- **Get Instrument Exposure**: `GET /exposure/{instrument}`
- **Metrics**: `GET /metrics`

#### Liquidity Graph Service

Base URL: `http://localhost:8091` (or `/liquidity` on gateway).

- **Health**: `GET /health`
- **Metrics**: `GET /metrics`
- **Graph snapshot**: `GET /v1/graph/snapshot`
- **Recompute mock graph**: `POST /v1/graph/recompute`
- **Plan**: `POST /v1/plan` — body: `{ "instrument": "EURUSD", "side": "buy", "quantity": 1000000.0 }`

#### Execution Engine

Base URL: `http://localhost:8092` (or `/execution` on gateway).

- **Health**: `GET /health`
- **Metrics**: `GET /metrics`
- **Execute (mock pipeline)**: `POST /v1/execute` — body: `{ "instrument": "EURUSD", "side": "buy", "quantity": 1000000.0, "client_id": "c1" }`  
  Response includes echoed `client_id`, `plan`, `fills`, `ai_notes`, and latencies.

#### AI Execution Service (Python)

Base URL: `http://localhost:8093` (not proxied by gateway; called by `execution-engine`).

- **Health**: `GET /health`
- **Infer**: `POST /v1/infer` — venue feature vector and instrument context (see `ai/ai-execution-service/main.py`)

## WebSocket API

### Connection

```javascript
const ws = new WebSocket("ws://localhost:8080/ws");
```

### Message Format

```json
{
  "type": "quote|order|trade|error",
  "data": { ... }
}
```

### Message Types

#### Quote Update

```json
{
  "type": "quote",
  "data": {
    "instrument": "EURUSD",
    "bid": 1.085,
    "ask": 1.0852,
    "spread": 0.0002,
    "timestamp": 1234567890
  }
}
```

#### Order Update

```json
{
  "type": "order",
  "data": {
    "order_id": "uuid",
    "status": "filled|partial|pending|cancelled",
    "trades": [ ... ]
  }
}
```

#### Trade Execution

```json
{
  "type": "trade",
  "data": {
    "trade_id": "uuid",
    "instrument": "EURUSD",
    "quantity": 1000000,
    "price": 1.0851,
    "timestamp": 1234567890
  }
}
```

## gRPC API

### Matching Engine gRPC Service

**Service**: `fx.etrading.MatchingEngineService`

**Methods**:

- `SubmitOrder(Order) -> OrderResponse`
- `CancelOrder(CancelOrderRequest) -> CancelOrderResponse`
- `StreamQuotes(StreamQuotesRequest) -> stream Quote`

See `crates/fx-proto/proto/fx.proto` for full protocol definitions.

## Request/Response Examples

### Submit Order

**Request**:

```bash
curl -X POST http://localhost:8080/matching/orders \
  -H "Content-Type: application/json" \
  -d '{
    "instrument": "EURUSD",
    "side": "Buy",
    "order_type": "Limit",
    "quantity": 1000000,
    "price": 1.0850
  }'
```

**Response**:

```json
{
  "success": true,
  "message": "Order placed",
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "trades": []
}
```

### Get Trades

**Request**:

```bash
curl http://localhost:8080/matching/trades
```

**Response**:

```json
{
  "trades": [
    {
      "trade_id": "uuid",
      "buy_order_id": "uuid",
      "sell_order_id": "uuid",
      "instrument": "EURUSD",
      "quantity": 1000000,
      "price": 1.0851,
      "timestamp_ns": 1234567890000000000
    }
  ]
}
```

### Execute (liquidity + AI pipeline)

**Request** (via gateway):

```bash
curl -X POST http://localhost:8080/execution/v1/execute \
  -H "Content-Type: application/json" \
  -d '{
    "instrument": "EURUSD",
    "side": "buy",
    "quantity": 1000000,
    "client_id": "demo-client"
  }'
```

**Response** (shape): JSON with `client_id`, `risk_ok`, `plan`, `fills`, `total_latency_us`, `ai_notes`.

### Check Risk

**Request**:

```bash
curl -X POST http://localhost:8080/risk/check \
  -H "Content-Type: application/json" \
  -d '{
    "instrument": "EURUSD",
    "side": "Buy",
    "quantity": 1000000,
    "order_id": "uuid"
  }'
```

**Response**:

```json
{
  "success": true,
  "message": "Risk check passed"
}
```

## Error Responses

### 400 Bad Request

```json
{
  "error": "Invalid request",
  "message": "Invalid side: InvalidSide"
}
```

### 404 Not Found

```json
{
  "error": "Not found",
  "message": "Order not found"
}
```

### 500 Internal Server Error

```json
{
  "error": "Internal server error",
  "message": "Service unavailable"
}
```

## Rate Limiting

Rate limiting is implemented in the risk service. Default limits:

- **Orders per second**: 100
- **Orders per minute**: 1000
- **Orders per day**: 10000

## WebSocket Reconnection

The WebSocket client automatically reconnects on disconnection:

- **Max retries**: 10
- **Retry delay**: Exponential backoff (1s, 2s, 4s, ...)
- **Max delay**: 60 seconds
