# ADR-0010: SMC advisory HTTP API surface

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M9 exposes research analysis over HTTP/WebSocket with OpenAPI. Secrets (Telegram) must never
live in TOML or source.

## Decision

1. Binary `fx-smc-advisory-api` (Axum) on `[api].http_port` (default 8094).
2. Typed JSON DTOs; OpenAPI via `utoipa` + Swagger UI at `/docs`.
3. Endpoints: `GET /health`, `GET /disclaimer`, `POST /v1/analyze` (synth→plans+regime),
   `GET /ws` (heartbeat + disclaimer push).
4. Telegram: optional; enabled only when `api.telegram_enabled` **and** env
   `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` are set. Missing secrets → skip with log, no panic.
5. Every analyze response includes disclaimer text; never promise returns.

## Consequences

- Cold-path Tokio is fine (not matching hot path).
- Operators must inject Telegram secrets via environment only.
