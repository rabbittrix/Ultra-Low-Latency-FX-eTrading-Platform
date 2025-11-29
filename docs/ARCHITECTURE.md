# Architecture Documentation

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0

## System Architecture

### High-Level Overview

The Ultra-Low-Latency FX eTrading Platform follows a microservices architecture with the following layers:

1. **Frontend Layer**: Next.js trading interface
2. **API Gateway Layer**: Single entry point for all services
3. **Business Logic Layer**: Core trading services
4. **Data Layer**: Market data and order book
5. **Observability Layer**: Monitoring and logging

### Service Communication

```text
Frontend (Next.js)
    ↓ HTTP/WebSocket
Gateway Service
    ↓ HTTP/gRPC
┌─────────────────────────────────────┐
│  Market Data  │  Pricing  │  Risk  │
│  Matching     │  Router   │  AI    │
└─────────────────────────────────────┘
    ↓
Observability Stack (Prometheus, Grafana, Jaeger, ELK)
```

## Core Services

### Market Data Service

- **Purpose**: Ingests and normalizes FX market data
- **Protocols**: REST, WebSocket
- **Port**: 8081
- **Dependencies**: None

### Pricing Service

- **Purpose**: Generates risk-adjusted prices
- **Protocols**: REST, WebSocket
- **Port**: 8082
- **Dependencies**: Market Data Service, Risk Service, AI Service (optional)

### Matching Engine Service

- **Purpose**: Order matching and trade execution
- **Protocols**: REST, gRPC
- **Port**: 8083 (REST), 50051 (gRPC)
- **Dependencies**: Risk Service

### Risk Service

- **Purpose**: Pre-trade risk validation
- **Protocols**: REST
- **Port**: 8084
- **Dependencies**: None

### Router Service

- **Purpose**: Order routing to external venues
- **Protocols**: REST
- **Port**: 8085
- **Dependencies**: Matching Engine Service

### Gateway Service

- **Purpose**: API aggregation and request routing
- **Protocols**: REST, WebSocket
- **Port**: 8080
- **Dependencies**: All backend services

## Data Flow

### Order Submission Flow

```text
1. Frontend → Gateway Service (POST /matching/orders)
2. Gateway → Risk Service (POST /check)
3. Gateway → Matching Engine (POST /orders)
4. Matching Engine → Order Book (lock-free matching)
5. Matching Engine → Trade Log (store execution)
6. Matching Engine → Response (trades + order status)
7. Gateway → Frontend (WebSocket notification)
```

### Market Data Flow

```text
1. Market Data Service → Generate Quotes
2. Market Data Service → WebSocket Broadcast
3. Gateway Service → Aggregate Streams
4. Gateway Service → Frontend (WebSocket)
5. Frontend → Display Real-time Data
```

## Performance Characteristics

### Latency Targets

- **Order Matching**: < 1ms (p99)
- **Risk Check**: < 500μs (p99)
- **Price Calculation**: < 1ms (p99)
- **WebSocket Latency**: < 10ms (p99)

### Throughput Targets

- **Orders/Second**: 10,000+
- **Quotes/Second**: 100,000+
- **Concurrent Connections**: 1,000+

## Technology Stack

### Backend (Rust)

- **Runtime**: Tokio async runtime
- **Web Framework**: Axum
- **gRPC**: Tonic
- **Serialization**: Serde
- **Metrics**: Prometheus

### Frontend

- **Framework**: Next.js 15
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **State Management**: React Hooks

### Observability

- **Metrics**: Prometheus
- **Visualization**: Grafana
- **Tracing**: Jaeger
- **Logging**: Fluentd + Elasticsearch + Kibana

## Scalability

### Horizontal Scaling

All services are stateless and can be scaled horizontally:

```bash
docker-compose up -d --scale matching-engine-service=3
```

### Load Balancing

Use a load balancer to distribute traffic across service instances.

### Database Considerations

Currently uses in-memory data structures. For persistence:

- **Order Book**: Redis or custom storage
- **Trade Log**: PostgreSQL or TimescaleDB
- **Audit Log**: Elasticsearch or ClickHouse

## Security Architecture

### Network Security

- Services communicate over internal Docker network
- External access only through Gateway Service
- TLS/HTTPS for external endpoints (production)

### Authentication (Future)

- JWT tokens for API access
- WebSocket authentication
- Rate limiting per client

## Deployment Architecture

### Container Orchestration

- **Development**: Docker Compose
- **Production**: Kubernetes (recommended) or Docker Swarm

### Service Discovery

- **Development**: Docker DNS
- **Production**: Kubernetes Service Discovery or Consul

## Monitoring Architecture

### Metrics Collection

```text
Services → Prometheus → Grafana
```

### Log Aggregation

```text
Services → Fluentd → Elasticsearch → Kibana
```

### Distributed Tracing

```text
Services → OpenTelemetry → Jaeger
```

## Disaster Recovery

### Backup Strategy

1. **Configuration**: Version controlled in Git
2. **Data**: Regular backups of persistent storage
3. **State**: Stateless services enable quick recovery

### Recovery Procedures

1. Restore from backups
2. Redeploy services
3. Verify health checks
4. Monitor metrics
