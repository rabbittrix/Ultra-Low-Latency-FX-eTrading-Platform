# FX eTrading Platform - Deployment & Observability

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0

## Observability Stack

### Prometheus

Prometheus is configured to scrape metrics from all services:

- **Gateway Service**: `gateway-service:9090`
- **Market Data Service**: `market-data-service:9091`
- **Pricing Service**: `pricing-service:9092`
- **Matching Engine Service**: `matching-engine-service:9093`
- **Risk Service**: `risk-service:9094`
- **Router Service**: `router-service:9095`

**Optional (local dev, not in default Compose):** `liquidity-graph-service` and `execution-engine` expose `GET /metrics` on HTTP ports `8091` and `8092`. Add scrape jobs to `prometheus.yml` when those services run in Docker.

Access Prometheus UI at: `http://localhost:9099`

### Grafana

Grafana is pre-configured with:

- **Prometheus Datasource**: Automatically provisioned from `grafana/provisioning/datasources/datasource.yaml`
- **Dashboards**:
  - FX Trading Platform - Overview
  - Matching Engine - Performance
  - Risk Service - Monitoring
  - Market Data Service - Monitoring

**Configuration Structure:**

```
grafana/
├── provisioning/
│   ├── datasources/
│   │   └── datasource.yaml       # Prometheus (uid: prometheus)
│   └── dashboards/
│       └── dashboards.yaml       # File provider → dashboard-definitions
└── dashboard-definitions/        # Dashboard JSON (overwrite + datasource uid)
    ├── fx-trading-overview.json
    ├── matching-engine.json
    └── ...
```

After changing dashboards, recreate Grafana or clear its volume once so provisioning reloads:  
`docker compose up -d --force-recreate grafana` (or `docker compose down` and remove the `grafana-data` volume if dashboards still missing).

Access Grafana at: `http://localhost:3001` (admin/admin)

### Jaeger

Jaeger is configured for distributed tracing:

- **UI**: `http://localhost:16686`
- **Collector**: `http://localhost:14268` (OTLP endpoint)

**Note**: Full OpenTelemetry integration requires adding dependencies to services. The infrastructure is ready.

### Fluentd + Elasticsearch + Kibana

Log aggregation stack:

- **Fluentd**: Collects logs from all services
- **Elasticsearch**: Stores logs at `http://localhost:9200`
- **Kibana**: Visualizes logs at `http://localhost:5601`

Fluentd configuration: `deploy/fluentd/fluent.conf`

## Metrics Exported

### Gateway Service

- `gateway_requests_total` - Total requests
- `gateway_request_duration_seconds` - Request latency
- `gateway_websocket_connections_total` - WebSocket connections
- `gateway_active_websocket_clients` - Active clients
- `gateway_backend_errors_total` - Backend errors

### Market Data Service

- `market_data_quotes_published_total` - Quotes published
- `market_data_websocket_connections` - WebSocket connections
- `market_data_active_subscribers` - Active subscribers
- `market_data_quote_latency_ns` - Quote processing latency

### Pricing Service

- `pricing_calculations_total` - Price calculations
- `pricing_calculation_duration_seconds` - Calculation latency
- `pricing_ai_client_requests_total` - AI client requests
- `pricing_ai_client_errors_total` - AI client errors
- `pricing_risk_adjustments_total` - Risk adjustments

### Matching Engine Service

- `matching_engine_orders_submitted_total` - Orders submitted
- `matching_engine_orders_cancelled_total` - Orders cancelled
- `matching_engine_orders_rejected_total` - Orders rejected
- `matching_engine_trades_executed_total` - Trades executed
- `matching_engine_order_matching_duration_seconds` - Matching latency
- `matching_engine_orderbook_depth` - Order book depth
- `matching_engine_active_orders` - Active orders

### Risk Service

- `risk_checks_total` - Risk checks performed
- `risk_checks_passed_total` - Checks passed
- `risk_checks_failed_total` - Checks failed
- `risk_check_duration_seconds` - Check latency
- `risk_total_positions` - Total positions
- `risk_total_exposure` - Total exposure
- `risk_position_limit_utilization` - Limit utilization

### Router Service

- `router_orders_routed_total` - Orders routed
- `router_routing_duration_seconds` - Routing latency
- `router_venue_errors_total` - Venue errors
- `router_active_venues` - Active venues

## Deployment

```bash
cd deploy
docker-compose up -d
```

This will start:

- All Rust services with metrics endpoints
- Prometheus for metrics collection
- Grafana for visualization
- Jaeger for tracing
- Elasticsearch for log storage
- Kibana for log visualization
- Fluentd for log aggregation

## Access Points

- **Frontend** (Compose maps host **3002** → container 3000): <http://localhost:3002>
- **Gateway API**: <http://localhost:8080>
- **Swagger UI**: <http://localhost:8080/docs>
- **Grafana**: <http://localhost:3001 (admin/admin)>
- **Prometheus**: <http://localhost:9099>
- **Jaeger**: <http://localhost:16686>
- **Kibana**: <http://localhost:5601>
- **Elasticsearch**: <http://localhost:9200>
