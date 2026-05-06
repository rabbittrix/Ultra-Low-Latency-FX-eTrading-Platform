# Deployment Guide

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0  
**Repository:** <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform.git>

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Production Deployment](#production-deployment)
4. [Environment Configuration](#environment-configuration)
5. [Service Configuration](#service-configuration)
6. [Scaling](#scaling)
7. [Monitoring & Observability](#monitoring--observability)
8. [Security](#security)
9. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 20.04+, Debian 11+, or RHEL 8+)
- **CPU**: 4+ cores (8+ recommended for production)
- **RAM**: 8GB minimum (16GB+ recommended)
- **Disk**: 50GB+ free space
- **Network**: Low-latency network (<1ms internal latency)

### Software Requirements

- **Docker**: 20.10+ with Docker Compose 2.0+
- **Git**: 2.30+
- **curl**: For health checks

### Optional (for local development)

- **Rust**: 1.83+ (for building from source)
- **Node.js**: 20+ (for frontend development)
- **Python**: 3.12+ (for AI service development)

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform.git
cd Ultra-Low-Latency-FX-eTrading-Platform
```

### 2. Start All Services

```bash
cd deploy
docker-compose up -d
```

### 3. Verify Services

```bash
# Check service status
docker-compose ps

# Check logs
docker-compose logs -f gateway-service

# Test health endpoints
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health
curl http://localhost:8083/health
curl http://localhost:8084/health
curl http://localhost:8085/health
```

**Liquidity graph, execution engine, and AI execution** (ports `8091`–`8093`) are part of the workspace but are **not** started by the default `docker-compose.yml`. For a full local chain:

1. Start `ai/ai-execution-service` on port `8093` (or set `PORT`).
2. `cargo run --bin liquidity-graph-service` (8091) and `cargo run --bin execution-engine` (8092), with `AI_EXECUTION_URL` pointing at the Python service if not on localhost.
3. Use the gateway: `http://localhost:8080/liquidity/...` and `http://localhost:8080/execution/...`.

### 4. Access Services

- **Frontend** (Docker Compose): <http://localhost:3002> (default; override with `FRONTEND_HOST_PORT` in `deploy/.env`)
- **Gateway API**: <http://localhost:8080>
- **Swagger UI**: <http://localhost:8080/docs>
- **Grafana**: <http://localhost:3001> (admin/admin)
- **Prometheus**: <http://localhost:9099>
- **Jaeger**: <http://localhost:16686>
- **Kibana**: <http://localhost:5601>

## Production Deployment

### Step 1: Prepare Environment

```bash
# Create production directory
mkdir -p /opt/fx-trading
cd /opt/fx-trading

# Clone repository
git clone https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform.git .
```

### Step 2: Configure Environment Variables

Create `.env` file in `deploy/` directory:

```bash
cd deploy
cat > .env <<EOF
# Service URLs (internal Docker network)
MARKET_DATA_URL=http://market-data-service:8081
PRICING_URL=http://pricing-service:8082
MATCHING_ENGINE_URL=http://matching-engine-service:8083
RISK_URL=http://risk-service:8084

# Logging
RUST_LOG=info

# Frontend
NEXT_PUBLIC_API_URL=http://gateway-service:8080
NODE_ENV=production

# Grafana
GF_SECURITY_ADMIN_PASSWORD=your-secure-password

# Elasticsearch
ES_JAVA_OPTS=-Xms1g -Xmx1g
EOF
```

### Step 3: Build Docker Images

```bash
# Build all services
docker-compose build

# Or build specific service
docker-compose build gateway-service
```

### Step 4: Deploy Services

```bash
# Start all services
docker-compose up -d

# Verify deployment
docker-compose ps
docker-compose logs --tail=50 gateway-service
```

### Step 5: Configure Firewall

```bash
# Allow required ports
sudo ufw allow 3000/tcp  # Frontend
sudo ufw allow 8080/tcp  # Gateway API
sudo ufw allow 3001/tcp  # Grafana
sudo ufw allow 9099/tcp  # Prometheus
sudo ufw allow 16686/tcp # Jaeger
sudo ufw allow 5601/tcp  # Kibana
```

### Step 6: Setup Reverse Proxy (Optional)

#### Nginx Configuration

```nginx
# /etc/nginx/sites-available/fx-trading
server {
    listen 80;
    server_name trading.example.com;

    # Frontend
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API Gateway
    location /api {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket
    location /ws {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

## Environment Configuration

### Service-Specific Configuration

#### Market Data Service

```yaml
# docker-compose.yml
market-data-service:
  environment:
    - RUST_LOG=info
    - INSTRUMENTS=EURUSD,GBPUSD,USDJPY,AUDUSD
    - QUOTE_INTERVAL_MS=100
```

#### Matching Engine Service

```yaml
matching-engine-service:
  environment:
    - RUST_LOG=info
    - DEFAULT_INSTRUMENT=EURUSD
    - MAX_ORDER_SIZE=10000000
```

#### Risk Service

```yaml
risk-service:
  environment:
    - RUST_LOG=info
    - MAX_POSITION_SIZE=10000000
    - MAX_DAILY_LOSS=100000
    - MAX_OPEN_ORDERS=100
```

#### Pricing Service

```yaml
pricing-service:
  environment:
    - RUST_LOG=info
    - AI_SERVICE_URL=http://python-ml-service:8086
    - SPREAD_BASIS_POINTS=2
```

### Logging Configuration

Set `RUST_LOG` environment variable:

- `error`: Only errors
- `warn`: Warnings and errors
- `info`: Info, warnings, and errors (recommended for production)
- `debug`: Debug information
- `trace`: Very verbose logging

## Service Configuration

### Health Checks

All services include health check endpoints:

```bash
# Check individual service health
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health
curl http://localhost:8083/health
curl http://localhost:8084/health
curl http://localhost:8085/health
```

### Metrics Endpoints

All services expose Prometheus metrics:

```bash
# Get metrics
curl http://localhost:8080/metrics
curl http://localhost:8081/metrics
curl http://localhost:8082/metrics
curl http://localhost:8083/metrics
curl http://localhost:8084/metrics
curl http://localhost:8085/metrics
```

## Scaling

### Horizontal Scaling

#### Scale Matching Engine

```bash
# Scale to 3 instances
docker-compose up -d --scale matching-engine-service=3
```

#### Scale Market Data Service

```bash
# Scale to 2 instances
docker-compose up -d --scale market-data-service=2
```

### Load Balancing

Use a load balancer (Nginx, HAProxy, or cloud load balancer) to distribute traffic:

```nginx
# Nginx load balancing example
upstream matching_engine {
    least_conn;
    server matching-engine-service-1:8083;
    server matching-engine-service-2:8083;
    server matching-engine-service-3:8083;
}
```

### Resource Limits

Set resource limits in `docker-compose.yml`:

```yaml
matching-engine-service:
  deploy:
    resources:
      limits:
        cpus: "2"
        memory: 2G
      reservations:
        cpus: "1"
        memory: 1G
```

## Monitoring & Observability

### Prometheus

- **URL**: <http://localhost:9099>
- **Configuration**: `deploy/prometheus.yml`
- **Retention**: 30 days (configurable)

### Grafana

- **URL**: <http://localhost:3001>
- **Default Credentials**: admin/admin (change in production!)
- **Dashboards**: File-provisioned from `deploy/grafana/dashboard-definitions/` (see `deploy/grafana/provisioning/dashboards/dashboards.yaml`)

### Jaeger

- **URL**: <http://localhost:16686>
- **OTLP Endpoint**: <http://localhost:14268>

### Kibana

- **URL**: <http://localhost:5601>
- **Elasticsearch**: <http://localhost:9200>

## Security

### Production Security Checklist

- [ ] Change default Grafana password
- [ ] Use HTTPS/TLS for all external endpoints
- [ ] Implement API authentication (JWT tokens)
- [ ] Enable rate limiting
- [ ] Configure firewall rules
- [ ] Use secrets management (Docker secrets, Vault)
- [ ] Enable audit logging
- [ ] Regular security updates
- [ ] Network isolation (separate Docker networks)
- [ ] Encrypt sensitive data at rest

### API Authentication (Future)

```rust
// Example: Add JWT authentication middleware
use axum::extract::Request;
use axum::middleware::Next;

async fn auth_middleware(request: Request, next: Next) -> Response {
    // Verify JWT token
    // Add user context to request
    next.run(request).await
}
```

### Network Security

```yaml
# docker-compose.yml
networks:
  fx-network:
    driver: bridge
    internal: false # Set to true for isolated network
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
docker-compose logs service-name

# Check container status
docker-compose ps

# Restart service
docker-compose restart service-name
```

### High Memory Usage

```bash
# Check memory usage
docker stats

# Increase memory limits in docker-compose.yml
# Or add swap space
```

### Network Issues

```bash
# Check network connectivity
docker-compose exec gateway-service curl http://matching-engine-service:8083/health

# Check DNS resolution
docker-compose exec gateway-service nslookup matching-engine-service
```

### Frontend shows 404 on `/matching/audit` (or other proxied paths)

The matching engine exposes `GET /audit`; the gateway serves it as `GET http://localhost:8080/matching/audit`.

1. **Confirm the process on port 8080 is this gateway** (not another app): `curl http://localhost:8080/health` should return gateway health JSON. If another program owns 8080, stop it or change the gateway host port in Compose.
2. **Hit the matching engine directly** (from the host, if port 8083 is published): `curl http://localhost:8083/audit` — expect `{"events":...}`. If that 404s, rebuild and recreate the matching-engine container so it includes the current REST routes:
   `docker compose build matching-engine-service && docker compose up -d matching-engine-service`
3. **Through the gateway** (inside Docker network):  
   `docker compose exec gateway-service curl -s http://localhost:8080/matching/audit`

Environment variables: the gateway accepts both `MATCHING_ENGINE_SERVICE_URL` and the legacy Compose name `MATCHING_ENGINE_URL`.

### Performance Issues

1. **Check metrics**: Review Prometheus metrics
2. **Check logs**: Look for errors in service logs
3. **Check resources**: Monitor CPU/memory usage
4. **Check network**: Verify low latency between services

### Common Issues

#### Port Already in Use

```bash
# Find process using port
sudo lsof -i :8080

# Kill process or change port in docker-compose.yml
```

#### Out of Disk Space

```bash
# Clean Docker
docker system prune -a

# Remove old images
docker image prune -a
```

#### Service Health Check Failing

```bash
# Check service logs
docker-compose logs service-name

# Test health endpoint manually
curl http://localhost:PORT/health

# Verify service is listening
docker-compose exec service-name netstat -tlnp
```

## Backup & Recovery

### Database Backup (if using persistent storage)

```bash
# Backup Elasticsearch
curl -X POST "http://localhost:9200/_snapshot/backup_repo/snapshot_1"

# Backup Prometheus data
docker-compose exec prometheus tar -czf /backup/prometheus-$(date +%Y%m%d).tar.gz /prometheus
```

### Configuration Backup

```bash
# Backup configuration files
tar -czf config-backup-$(date +%Y%m%d).tar.gz \
  deploy/docker-compose.yml \
  deploy/prometheus.yml \
  deploy/grafana/
```

## Maintenance

### Updates

```bash
# Pull latest changes
git pull

# Rebuild and restart services
docker-compose build
docker-compose up -d
```

### Log Rotation

Configure log rotation in `docker-compose.yml`:

```yaml
services:
  matching-engine-service:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

## Support

For issues or questions:

- **GitHub Issues**: <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform/issues>
- **Email**: <rabbittrix@hotmail.com>
