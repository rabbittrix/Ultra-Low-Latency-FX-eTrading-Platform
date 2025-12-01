# Ultra-Low-Latency FX eTrading Platform

[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/next.js-15.0-black)](https://nextjs.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Author:** Roberto de Souza  
**Email:** <rabbittrix@hotmail.com>  
**License:** Apache-2.0

A production-grade, ultra-low-latency Foreign Exchange (FX) electronic trading platform built with Rust microservices, Next.js frontend, and a complete observability stack. Engineered to exceed typical institutional latency requirements with sub-millisecond internal processing paths.

## 🎯 Overview

This platform provides a complete FX trading solution with:

- **Ultra-low-latency matching engine** with lock-free data structures
- **Real-time market data processing** with normalized feeds
- **Risk management** with pre-trade validation
- **AI/ML integration** for volatility prediction
- **Modern trading UI** with real-time order books and charts
- **Complete observability** with Prometheus, Grafana, Jaeger, and ELK stack
- **Production-ready architecture** designed for horizontal scaling

## 🏗️ Architecture

The platform follows a microservices architecture with the following components:

### Core Services (Rust)

1. **Market Data Service** (`market-data-service`)

   - Ingests and normalizes FX market data feeds
   - Publishes L2/L3 order books via WebSocket and REST
   - Real-time quote streaming over WebSocket (`/ws`)
   - REST endpoint for latest quotes (`/quote`)
   - Mock feed generator for multiple instruments (EURUSD, GBPUSD, USDJPY, AUDUSD)
   - Prometheus metrics (quotes published, subscribers, latency)
   - Zero-copy messaging internally
   - Port: `8081` (HTTP/WebSocket), `/metrics` (Prometheus)

2. **Pricing Engine** (`pricing-service`)

   - Generates BID/ASK spreads
   - Applies risk-based price adjustments
   - Integrates with AI/ML modules
   - Port: `8082` (HTTP), `9092` (Metrics)

3. **Matching Engine** (`matching-engine-service`)

   - Ultra-low-latency order matching
   - Lock-free order book implementation
   - Supports Market, Limit, Stop, IOC, FOK orders
   - Port: `8083` (HTTP), `9093` (Metrics)

4. **Risk Engine** (`risk-service`)

   - Pre-trade risk validation
   - Real-time position tracking
   - Exposure calculation
   - Port: `8084` (HTTP), `9094` (Metrics)

5. **Order Router** (`router-service`)

   - Routes orders to external venues
   - Latency-optimized routing
   - Port: `8085` (HTTP), `9095` (Metrics)

6. **API Gateway** (`gateway-service`)
   - Aggregates all microservices
   - REST and WebSocket APIs
   - Swagger/OpenAPI documentation
   - Port: `8080` (HTTP), `9090` (Metrics)

### Frontend (Next.js)

- **Trading UI** (`nextjs-trading-ui`)
  - Real-time order book visualization
  - Live price charts
  - Order ticket panel
  - Portfolio and PnL tracking
  - Admin dashboard with observability views
  - Port: `3000`

### AI/ML Service (Python)

- **ML Service** (`python-ml-service`)
  - Volatility prediction models
  - REST/gRPC API for Rust integration
  - FastAPI-based
  - Port: `8086`

### Observability Stack

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization dashboards
- **Jaeger**: Distributed tracing
- **Elasticsearch + Kibana + Fluentd**: Log aggregation and analysis

## 📦 Project Structure

```text
.
├── crates/                    # Publishable Rust libraries
│   ├── fx-core/              # Matching engine core logic
│   ├── fx-md/                # Market data processing
│   ├── fx-pricing/           # Pricing engine
│   ├── fx-risk/              # Risk management
│   ├── fx-router/            # Order routing
│   ├── fx-gateway/           # API gateway utilities
│   ├── fx-proto/             # gRPC protocol definitions
│   └── fx-utils/             # Shared utilities
├── services/                  # Service binaries
│   ├── market-data-service/
│   ├── pricing-service/
│   ├── matching-engine-service/
│   ├── risk-service/
│   ├── router-service/
│   └── gateway-service/
├── frontend/
│   └── nextjs-trading-ui/    # Next.js trading interface
├── ai/
│   └── python-ml-service/    # Python ML service
└── deploy/
    ├── docker-compose.yml    # Complete stack orchestration
    ├── prometheus.yml        # Prometheus configuration
    ├── grafana/              # Grafana configuration
    │   ├── provisioning/     # Grafana provisioning
    │   │   ├── datasources/  # Datasource configurations
    │   │   └── dashboards/    # Dashboard configurations
    │   └── dashboards/       # Dashboard JSON files
    ├── fluentd/              # Fluentd log aggregation
    └── Dockerfile.*          # Service-specific Dockerfiles
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.82+ (stable)
- **Node.js** 20+ and npm/yarn
- **Python** 3.12+
- **Docker** and Docker Compose
- **Git**

### Local Development

#### 1. Clone the Repository

```bash
git clone <repository-url>
cd Ultra-Low-Latency_FX_eTrading
```

#### 2. Build Rust Services

```bash
# Build all services
cargo build --release

# Run individual services
cargo run --bin market-data-service
cargo run --bin pricing-service
cargo run --bin matching-engine-service
cargo run --bin risk-service
cargo run --bin router-service
cargo run --bin gateway-service
```

#### 3. Setup Frontend

```bash
cd frontend/nextjs-trading-ui
npm install
npm run dev
```

The frontend will be available at `http://localhost:3000`

#### 4. Setup Python ML Service

```bash
cd ai/python-ml-service
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
python main.py
```

#### 5. Run with Docker Compose

```bash
cd deploy
docker-compose up -d
```

This will start all services including the observability stack. Access:

- **Frontend**: <http://localhost:3000>
- **Gateway API**: <http://localhost:8080>
- **Swagger UI**: <http://localhost:8080/docs>
- **Grafana**: <http://localhost:3001> (admin/admin)
- **Prometheus**: <http://localhost:9099>
- **Jaeger**: <http://localhost:16686>
- **Kibana**: <http://localhost:5601>

## 🔧 Configuration

### Environment Variables

Services can be configured via environment variables:

- `RUST_LOG`: Logging level (e.g., `info`, `debug`, `trace`)
- `NEXT_PUBLIC_API_URL`: Frontend API endpoint
- `PYTHONUNBUFFERED`: Python output buffering

### Service Ports

| Service         | HTTP Port | Metrics Port |
| --------------- | --------- | ------------ |
| Gateway         | 8080      | 9090         |
| Market Data     | 8081      | 9091         |
| Pricing         | 8082      | 9092         |
| Matching Engine | 8083      | 9093         |
| Risk            | 8084      | 9094         |
| Router          | 8085      | 9095         |
| ML Service      | 8086      | -            |
| Frontend        | 3000      | -            |

## 📊 Performance Characteristics

The platform is designed for ultra-low-latency:

- **Matching Engine**: Sub-millisecond order processing
- **Lock-free Data Structures**: Zero-contention order book
- **Zero Allocations**: Hot path avoids heap allocations
- **Network Optimizations**: TCP_NODELAY, disabled Nagle algorithm
- **NUMA-Aware**: Optimized for multi-socket systems

## 🧪 Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p fx-core

# Run with benchmarks
cargo bench
```

### Frontend Tests

```bash
cd frontend/nextjs-trading-ui
npm test
```

### Integration Tests

```bash
# Run integration tests with docker-compose
cd deploy
docker-compose -f docker-compose.test.yml up --abort-on-container-exit
```

## 📚 Documentation

### API Documentation

- **Swagger UI**: Available at `/docs` endpoint on the gateway service
- **OpenAPI Spec**: Generated automatically from code annotations

### Code Documentation

Generate Rust documentation:

```bash
cargo doc --open
```

### Architecture Documentation

See `flow-fx-et.md` for detailed architecture diagrams and service interactions.

## 🔒 Security

- **Memory Safety**: Rust's ownership system prevents common vulnerabilities
- **Input Validation**: All inputs are validated before processing
- **Rate Limiting**: Built into risk engine
- **Audit Logging**: All trades are logged for compliance

## 🚢 Deployment

### Production Deployment

1. **Build Docker Images**:

```bash
cd deploy
docker-compose build
```

1. **Configure Environment**:

Set appropriate environment variables in `docker-compose.yml` or use `.env` files.

1. **Deploy**:

```bash
docker-compose up -d
```

### Publishing Crates

To publish Rust crates to crates.io:

```bash
cd crates/fx-core
cargo publish
# Repeat for other crates
```

## 📈 Monitoring

### Metrics

All services expose Prometheus metrics at `/metrics` endpoints. Key metrics include:

- Order processing latency
- Trade execution rate
- Order book depth
- Risk check duration
- Service health status

### Dashboards

Pre-configured Grafana dashboards are available for:

- Trading metrics
- Latency analysis
- System health
- CPU/NUMA awareness

### Tracing

Distributed tracing via Jaeger shows request flows across all microservices.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards

- **Rust**: Follow `rustfmt` and `clippy` guidelines
- **TypeScript**: Use ESLint and Prettier
- **Python**: Follow PEP 8
- **Documentation**: All public APIs must be documented

## 📝 License

This project is licensed under the Apache License, Version 2.0.

- [Apache License 2.0](http://www.apache.org/licenses/LICENSE-2.0)

**Author:** Roberto de Souza  
**Email:** <rabbittrix@hotmail.com>

## 🙏 Acknowledgments

Built with:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Next.js](https://nextjs.org/) - React framework
- [FastAPI](https://fastapi.tiangolo.com/) - Python web framework
- [Prometheus](https://prometheus.io/) - Metrics
- [Grafana](https://grafana.com/) - Visualization
- [Jaeger](https://www.jaegertracing.io/) - Tracing

## 📧 Contact

**Author:** Roberto de Souza  
**Email:** <rabbittrix@hotmail.com>

For questions, issues, or contributions, please open an issue on [GitHub](https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform).

---

**Note**: This is a production-grade platform designed for real financial environments. Ensure proper testing and compliance with regulatory requirements before deployment.
