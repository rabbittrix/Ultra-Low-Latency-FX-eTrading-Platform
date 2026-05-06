# Architecture FX eTrading — Ultra-Low-Latency Platform

```mermaid
flowchart LR
    subgraph Frontend
        UI[Next.js Trading UI]
    end

    subgraph Gateway
        GW["API Gateway (Rust - axum)"]
        SW["Swagger / OpenAPI"]
    end

    subgraph CoreServices
        MD["Market Data Service (Rust)"]
        PR["Pricing Engine (Rust)"]
        ME["Matching Engine (Rust)"]
        RK["Risk Engine (Rust)"]
        OR["Order Router (Rust)"]
        LG["Liquidity Graph (Rust) :8091"]
        EX["Execution Engine (Rust) :8092"]
    end

    subgraph AI
        PY["Python ML Service (FastAPI/ONNX)"]
        AIX["AI Execution (FastAPI/ONNX) :8093"]
    end

    subgraph Messaging
        MQ["Shared Memory / Chronicle-like Queue"]
        KAF["Kafka (optional async streams)"]
    end

    subgraph Observability
        PROM[Prometheus]
        GRAF[Grafana]
        JAEG[Jaeger]
        EFK[Elasticsearch / Fluentd / Kibana]
    end

    subgraph Storage
        AUDIT[(Audit Log / Append-only Storage)]
        TSDB[(Timeseries / ClickHouse or kdb+)]
    end

    EXTERNAL((Exchanges / Mock Venues))

    UI -->|REST / WS| GW
    GW -->|gRPC / REST| MD
    GW --> PR
    GW --> ME
    GW --> RK
    GW -->|/liquidity| LG
    GW -->|/execution| EX
    EX -->|HTTP infer| AIX
    MD --> MQ
    PR --> MQ
    MQ --> ME
    ME --> MQ
    ME --> OR
    OR -->|external venues| EXTERNAL
    PR --> PY
    PY --> PR
    ME --> AUDIT
    PROM -->|scrapes| MD
    PROM --> PR
    PROM --> ME
    PROM --> RK
    PROM --> GW
    PROM --> LG
    PROM --> EX
    GRAF --> PROM
    JAEG --> GW
    JAEG --> MD
    EFK -->|logs| GW
    EFK --> ME
    MQ --- KAF
    KAF --> TSDB
    AUDIT --> TSDB
```
