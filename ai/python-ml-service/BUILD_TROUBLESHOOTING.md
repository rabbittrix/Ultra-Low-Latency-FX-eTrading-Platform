# Python ML Service Build Troubleshooting

## SSL Connection Errors

If you encounter SSL errors during Docker build:

```
SSLError(SSLEOFError(8, '[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol (_ssl.c:1010)')
```

### Solutions

#### 1. Build with Host Network (Docker 20.10+)

```bash
docker build --network=host -t python-ml-service .
```

#### 2. Build Locally First

```bash
cd ai/python-ml-service
docker build -t python-ml-service .
```

Then update `docker-compose.yml` to use the image:

```yaml
python-ml-service:
  image: python-ml-service
  # ... rest of config
```

#### 3. Use Docker BuildKit with Network Mode

```bash
DOCKER_BUILDKIT=1 docker build --network=host -t python-ml-service .
```

#### 4. Retry Build

Sometimes the issue is transient. Simply retry:

```bash
docker-compose build python-ml-service
```

#### 5. Use Alternative PyPI Mirror

Edit `Dockerfile` to use a different index:

```dockerfile
RUN pip install --index-url https://mirror.example.com/simple \
    --trusted-host mirror.example.com \
    -r requirements.txt
```

#### 6. Pre-download Packages

Build a base image with packages pre-installed:

```dockerfile
FROM python:3.12-slim as base
RUN pip install --no-cache-dir fastapi uvicorn pydantic numpy scikit-learn onnxruntime python-multipart

FROM base
# ... rest of Dockerfile
```

### Platform Functionality

**Important**: The FX Trading Platform is fully functional without the AI service. The pricing service will use default risk adjustments when the AI service is unavailable.

The AI service provides:

- Volatility predictions for risk-based price adjustments
- Enhanced pricing accuracy

Without it, the platform operates with:

- Standard risk-based price adjustments
- All other features fully operational
