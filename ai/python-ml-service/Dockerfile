FROM python:3.12-slim

WORKDIR /app

# Install system dependencies and update certificates for SSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Skip pip upgrade to avoid SSL issues during build
# Use system pip which should be sufficient
# Configure pip with extended timeouts and retries
RUN pip config set global.timeout 300 || true && \
    pip config set global.retries 5 || true && \
    pip config set global.trusted-host "pypi.org files.pythonhosted.org pypi.python.org" || true

COPY requirements.txt .

# Install packages directly without upgrading pip first
# Use multiple fallback strategies
RUN pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    -r requirements.txt || \
# Strategy 2: Try with explicit index URL
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --index-url https://pypi.org/simple \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    -r requirements.txt || \
# Strategy 3: Install packages individually for better error isolation
    (pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    fastapi==0.122.0 && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    "uvicorn[standard]==0.38.0" && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    pydantic==2.12.5 && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    numpy==2.3.5 && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    scikit-learn==1.7.2 && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    onnxruntime==1.23.2 && \
    pip install --no-cache-dir --timeout=300 --retries=5 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    --trusted-host pypi.python.org \
    python-multipart==0.0.20)

COPY main.py .

EXPOSE 8086

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8086"]
