# Ultra-Low-Latency FX eTrading Platform

[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/next.js-16.x-black)](https://nextjs.org/)
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
- **AI/ML integration** for volatility prediction (pricing) and venue scoring (execution)
- **Global liquidity graph** with path planning across mock venues and internal liquidity
- **AI-assisted execution pipeline** (Rust orchestrator + optional Python ONNX inference)
- **Modern trading UI** with real-time order books, charts, and a liquidity-engine dashboard
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
   - **Yahoo Finance integration** - Optional real market data feed (set `USE_YAHOO_FINANCE=true`)
   - Prometheus metrics (quotes published, subscribers, latency)
   - Zero-copy messaging internally
   - Port: `8081` (HTTP/WebSocket), `/metrics` (Prometheus)

2. **Pricing Engine** (`pricing-service`)

   - Generates BID/ASK spreads
   - Applies risk-based price adjustments
   - Integrates with AI/ML modules
   - Port: `8082` (HTTP), `9092` (Metrics)

3. **Matching Engine** (`matching-engine-service`)

   - Ultra-low-latency order matching (`fx-core`)
   - Lock-free order book; bids kept **descending** by price, asks ascending
   - `cancel_order` removes resting orders from the book (returns `false` if unknown)
   - Supports Market, Limit, Stop, IOC, FOK orders
   - Port: `8083` (HTTP), `50051` (gRPC), `9093` (Metrics)

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
   - Aggregates core microservices
   - REST and WebSocket APIs (browser connects directly to the gateway; Next.js does not proxy WS)
   - Swagger/OpenAPI documentation
   - Nested reverse proxies with path remainder forwarding, e.g. `/matching/*`, `/risk/*`, `/market-data/*`, `/pricing/*`, `/liquidity/*` → `8091`, `/execution/*` → `8092`
   - CORS `OPTIONS` on proxied routes; backend URLs default to `http://127.0.0.1:<port>` (override with `*_URL` / Compose)
   - Port: `8080` (HTTP; override with `GATEWAY_HTTP_PORT`), `9090` (Metrics)

7. **Liquidity Graph Service** (`liquidity-graph-service`)

   - In-memory global liquidity graph (mock data) and Dijkstra-style execution planning
   - REST: snapshot, recompute, plan; JSON `/health`
   - Prometheus metrics on default registry at `/metrics` on the **HTTP** port
   - Port: `8091` (HTTP + `/metrics`)

8. **Execution Engine** (`execution-engine`)

   - Deterministic-style pipeline: risk stub → AI venue scores → graph plan → parallel mock fills
   - Uses `fx-deterministic-core` ring buffer for hot-path handoff demo; `fx-ai-execution` HTTP client to Python
   - JSON `/health`; Port: `8092` (HTTP + `/metrics`)

### Domain crates (Rust libraries)

Additional workspace crates support front-office style modeling and the liquidity stack:

- `fx-oms`, `fx-ems` — order / execution management types
- `fx-exchange`, `fx-lp` — venue and liquidity-provider modeling
- `fx-liquidity-graph` — graph types and planning
- `fx-ai-execution` — low-latency HTTP client to the AI execution service
- `fx-deterministic-core` — deterministic ring buffer and TCP helpers

### Frontend (Next.js)

- **Trading UI** (`nextjs-trading-ui`)
  - Real-time order book visualization
  - Live price charts (supports real Yahoo Finance data)
  - Order ticket panel
  - Portfolio and PnL tracking
  - Admin dashboard with observability views
  - Liquidity Engine page (graph snapshot, recompute, execute via gateway)
  - Global backend status strip (gateway `/health` + WebSocket)
  - Default API/WS base: `http://127.0.0.1:8080` (`NEXT_PUBLIC_API_URL`; `localhost` normalized to `127.0.0.1`)
  - Port: `3000`

### AI/ML Services (Python)

- **ML Service** (`python-ml-service`)
  - Volatility prediction models
  - REST/gRPC API for Rust integration
  - FastAPI-based
  - Port: `8086`

- **AI Execution Service** (`ai/ai-execution-service`)
  - Venue-level inference: ONNX when `model.onnx` is present, else NumPy fallback
  - Train/export helper: `train_export.py` (scikit-learn → ONNX)
  - Port: `8093` (override with `PORT`)

### Observability Stack

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization dashboards
- **Jaeger**: Distributed tracing
- **Elasticsearch + Kibana + Fluentd**: Log aggregation and analysis

## 🚀 Quick Start

### Using Docker Compose (Recommended)

1. **Start all services with mock data:**

   ```bash
   cd deploy
   docker-compose up -d
   ```

2. **Start with real Yahoo Finance data:**

   ```bash
   cd deploy
   # Edit docker-compose.yml and set USE_YAHOO_FINANCE=true for market-data-service
   docker-compose up -d
   ```

3. **Access the services:**
   - Frontend (Docker): <http://localhost:3002> — host port set in `deploy/docker-compose.yml` (`FRONTEND_HOST_PORT`, default `3002`) so it does not conflict with a local dev server on 3000
   - Gateway API: <http://localhost:8080>
   - Swagger UI: <http://localhost:8080/docs>
   - Grafana: <http://localhost:3001> (admin/admin)
   - Prometheus: <http://localhost:9099>

**Liquidity / execution stack:** `liquidity-graph-service`, `execution-engine`, and `ai-execution-service` are not yet defined in `deploy/docker-compose.yml`. Run them locally for development (see (#getting-started) below) or add Dockerfiles and services to Compose as needed.

### Using Real Market Data

The market data service supports Yahoo Finance integration for real FX quotes:

```bash
# Set environment variable
export USE_YAHOO_FINANCE=true

# Or in docker-compose.yml
environment:
  - USE_YAHOO_FINANCE=true
```

**Note:** Yahoo Finance free tier provides delayed data (15-20 minutes). For production, consider paid APIs like Alpha Vantage, FXCM, or OANDA.

## 📦 Project Structure

```text
.
├── crates/                    # Publishable Rust libraries
│   ├── fx-core/              # Matching engine core logic
│   ├── fx-oms/               # Order management types
│   ├── fx-ems/               # Execution management types
│   ├── fx-md/                # Market data processing
│   ├── fx-exchange/          # Exchange / venue modeling
│   ├── fx-lp/                # Liquidity provider modeling
│   ├── fx-pricing/           # Pricing engine
│   ├── fx-risk/              # Risk management
│   ├── fx-router/            # Order routing
│   ├── fx-gateway/           # API gateway utilities
│   ├── fx-proto/             # gRPC protocol definitions
│   ├── fx-utils/             # Shared utilities
│   ├── fx-liquidity-graph/   # Global liquidity graph + planning
│   ├── fx-ai-execution/      # HTTP client for AI execution inference
│   └── fx-deterministic-core/# Deterministic ring buffer + socket helpers
├── services/                  # Service binaries
│   ├── market-data-service/  # Market data with Yahoo Finance support
│   ├── pricing-service/
│   ├── matching-engine-service/
│   ├── risk-service/
│   ├── router-service/
│   ├── gateway-service/
│   ├── liquidity-graph-service/
│   └── execution-engine/
├── frontend/
│   └── nextjs-trading-ui/    # Next.js trading interface (+ liquidity-engine page)
├── ai/
│   ├── python-ml-service/    # Volatility / pricing ML
│   └── ai-execution-service/ # Venue scoring for execution engine
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

## 🚀 Getting Started

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
cargo run --bin liquidity-graph-service
cargo run --bin execution-engine
```

#### 3. Frontend + local Rust stack (recommended on Windows)

For a single command that builds matching, liquidity graph, execution engine, and gateway into `target/dev-stack` (avoids locking `target/debug/*.exe`), waits on HTTP `/health`, then starts Next.js:

```bash
cd frontend/nextjs-trading-ui
npm install
npm run dev:stack
```

Behavior highlights:

- Stops prior `matching-engine-service` / `liquidity-graph-service` / `execution-engine` / `gateway-service` processes (set `DEV_STACK_NO_KILL=1` to skip)
- Auto-picks a free gateway port in `8080–8099` if `8080` is busy (skips Compose defaults such as `8081–8086`, `8091`, `8092`); sets `GATEWAY_HTTP_PORT` and `NEXT_PUBLIC_API_URL` for the UI
- Pin a port with `GATEWAY_HTTP_PORT=8088` (and matching `NEXT_PUBLIC_API_URL` if you run Next separately)

Or run the UI alone (gateway must already be up):

```bash
cd frontend/nextjs-trading-ui
npm install
npm run dev
```

The frontend will be available at `http://localhost:3000`. Lint uses the ESLint CLI (`npm run lint`) — Next.js 16 removed `next lint`.

#### 4. Setup Python ML Service

```bash
cd ai/python-ml-service
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
python main.py
```

#### 4b. AI Execution Service (for `execution-engine`)

```bash
cd ai/ai-execution-service
python -m venv .venv
# Windows: .venv\Scripts\activate
source .venv/bin/activate
pip install -r requirements.txt
python main.py
```

Default URL is `http://127.0.0.1:8093`. The Rust execution engine reads `AI_EXECUTION_URL` if set.

#### 5. Run with Docker Compose

```bash
cd deploy
docker-compose up -d
```

This will start all services including the observability stack. Access:

- **Frontend** (Compose): <http://localhost:3002>
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
- `USE_YAHOO_FINANCE`: Enable real market data from Yahoo Finance (`true`/`false`, default: `false`)
- `GATEWAY_HTTP_PORT`: Gateway listen port (default `8080`)
- `NEXT_PUBLIC_API_URL`: Frontend API/WS base URL (default `http://127.0.0.1:8080`)
- `DEV_STACK_TARGET_DIR` / `DEV_STACK_NO_KILL`: Overrides for `npm run dev:stack`
- `PYTHONUNBUFFERED`: Python output buffering
- `AI_EXECUTION_URL`: Base URL for AI execution service (default `http://127.0.0.1:8093`; used by `execution-engine`)
- `LIQUIDITY_INSTRUMENT`: Instrument string for mock liquidity graph (default `EURUSD`; used by liquidity + execution services)
- `PORT`: AI execution service listen port (default `8093`)

### Service Ports

| Service              | HTTP Port | Metrics Port |
| -------------------- | --------- | ------------ |
| Gateway              | 8080      | 9090         |
| Market Data          | 8081      | 9091         |
| Pricing              | 8082      | 9092         |
| Matching Engine      | 8083      | 9093         |
| Risk                 | 8084      | 9094         |
| Router               | 8085      | 9095         |
| ML Service (pricing) | 8086      | -            |
| Liquidity Graph      | 8091      | (same host)  |
| Execution Engine     | 8092      | (same host)  |
| AI Execution         | 8093      | -            |
| Frontend             | 3000      | -            |

Liquidity graph and execution engine expose Prometheus metrics at `GET /metrics` on their HTTP ports (no separate metrics listener).

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

### Frontend Tests / Lint

```bash
cd frontend/nextjs-trading-ui
npm run lint      # ESLint CLI (Next.js 16+)
npm run format -- --check
npm run build
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

See `flow-fx-et.md` and `docs/ARCHITECTURE.md` for diagrams and service interactions. REST details for the liquidity and execution APIs are in `docs/API.md`.

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

Publishable libraries (current release **0.1.2** on crates.io) include `fx-utils`, `fx-md`, `fx-risk`, `fx-core`, `fx-liquidity-graph`, `fx-pricing`, `fx-router`, `fx-gateway`, and `fx-proto`. Each package ships a `DONATION.md`. Prefer the automated script (dependency order, wait for index, restore path deps afterward):

```powershell
# From repo root (requires cargo login)
.\scripts\publish-all.ps1
```

See [PUBLISHING.md](PUBLISHING.md) for checklist and version bumps. For local workspace development, crates must use **path** dependencies (not crates.io versions) to avoid duplicate `fx_utils` type mismatches.

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

## 📧 Contact / Donations

**Author:** Roberto de Souza  
**Email:** <rabbittrix@hotmail.com>

Donation details: [DONATION.md](DONATION.md) (also included in published crates).

For questions, issues, or contributions, please open an issue on [GitHub](https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform).

---

**Note**: This is a production-grade platform designed for real financial environments. Ensure proper testing and compliance with regulatory requirements before deployment.
