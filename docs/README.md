# FX eTrading Platform Documentation

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0  
**Repository:** <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform.git>

## 📚 Documentation Index

### Getting Started

- [Quick Start Guide](../README.md#quick-start)
- [Installation Instructions](../DEPLOYMENT.md#prerequisites)
- [Configuration Guide](../DEPLOYMENT.md#environment-configuration)

### Architecture

- [System Architecture](../flow-fx-et.md) (diagram + local/gateway notes)
- [Service Overview](ARCHITECTURE.md)
- [API Documentation](http://localhost:8080/docs) (Swagger UI)
- [Donations](../DONATION.md)

### Development

- [Development Setup](../README.md#local-development)
- [Code Standards](../README.md#code-standards)
- [Testing Guide](../README.md#testing)

### Deployment

- [Deployment Guide](../DEPLOYMENT.md)
- [Production Checklist](../DEPLOYMENT.md#production-deployment)
- [Scaling Guide](../DEPLOYMENT.md#scaling)

### Publishing

- [Publishing to Crates.io](../PUBLISHING.md)
- [Version Management](../PUBLISHING.md#version-management)

### Observability

- [Monitoring Setup](../deploy/README.md)
- [Metrics Reference](../deploy/README.md#metrics-exported)
- [Dashboard Guide](../deploy/README.md#grafana)

### API Reference

- [REST API](http://localhost:8080/docs) - Swagger UI
- [WebSocket API](../README.md#websocket-protocol)
- [gRPC API](../crates/fx-proto/proto/fx.proto)

## 🔧 Generating Documentation

### Rust Documentation

```bash
# Generate all documentation
cargo doc --all --no-deps

# Open in browser
cargo doc --open

# Generate for specific crate
cargo doc -p fx-core --open
```

### API Documentation

- **Swagger UI**: <http://localhost:8080/docs>
- **OpenAPI Spec**: <http://localhost:8080/api-docs/openapi.json>

## 📖 Additional Resources

- [Project Instructions](../project_instructions.md)
- [Deployment README](../deploy/README.md)
- [Python ML Service README](../ai/python-ml-service/README.md)
- [AI Execution Service README](../ai/ai-execution-service/README.md) — FastAPI inference for `execution-engine`
- [Build Troubleshooting](../ai/python-ml-service/BUILD_TROUBLESHOOTING.md)

## 🌐 Online Documentation

For the latest documentation, visit:

- **GitHub Repository**: <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform>
- **Crates.io** (when published): <https://crates.io/crates/fx-core>

## 📧 Support

For questions or issues:

- **GitHub Issues**: <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform/issues>
- **Email**: <rabbittrix@hotmail.com>
