FROM python:3.12-slim

WORKDIR /app

# Install system dependencies and configure pip
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Configure pip with retries and timeout
RUN pip config set global.timeout 60 || true && \
    pip config set global.retries 10 || true

COPY requirements.txt .
# Install with multiple retry strategies
RUN pip install --no-cache-dir --timeout=60 --retries=10 \
    --trusted-host pypi.org \
    --trusted-host files.pythonhosted.org \
    -r requirements.txt

COPY main.py .

EXPOSE 8086

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8086"]

