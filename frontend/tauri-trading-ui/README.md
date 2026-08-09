# FX eTrading — Tauri Desktop UI

Native trading client for the Ultra-Low-Latency FX eTrading Platform.

**Stack:** Tauri 2 · Vite · React 18 · TypeScript · Tailwind CSS

## Why Tauri

- Smaller binary and lower memory than Electron
- OS-native webview with a **strict CSP** and **minimal capabilities**
- Observability is **in-app**: Tauri scrapes each service `/health` + `/metrics` and keeps local charts (no Prometheus/Grafana required for day-to-day)

## Quick start

```bash
npm install
npm run dev:stack          # matching + risk + liquidity + execution (Rust AI) + gateway + Tauri
# or
DEV_STACK_WEB_ONLY=1 npm run dev:stack   # Vite in browser only
npm run tauri:dev          # UI only (gateway must already be up)
npm run tauri:build        # production desktop installer
```

## Configuration

| Variable | Default | Purpose |
|----------|---------|---------|
| `VITE_API_URL` | `http://127.0.0.1:8080` | Gateway REST base |
| `VITE_WS_URL` | derived from API URL | Gateway WebSocket (`/ws`) |
| `VITE_PROMETHEUS_URL` | `http://127.0.0.1:9099` | Prometheus for in-app charts |
| `VITE_SMC_API_URL` | `http://127.0.0.1:8094` | fx-smc advisory API |

## Optional web / Docker

Compose can serve the Vite `dist/` via nginx (`deploy/docker-compose.yml` → host port **3002**). Prefer the Tauri desktop build for day-to-day trading.

## Author

Roberto de Souza \<rabbittrix@hotmail.com\> — Apache-2.0
