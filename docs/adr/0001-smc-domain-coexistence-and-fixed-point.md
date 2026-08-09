# ADR-0001: SMC domain coexistence and fixed-point prices

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

The repository already ships an FX matching / gateway / Tauri stack (`fx-*` crates, Tokio services).
A second product line (quantified SMC liquidity / advisory) requires stricter rules:
deterministic live==replay, fixed-point prices (no `f64`), hot-path isolation, and milestone crates.

Rewriting the existing matching engine in M0 would be high risk and out of scope.

## Decision

1. **Coexistence:** Implement SMC/advisory under `crates/fx-smc-*` and `services/fx-smc-*` as a parallel domain.
   Legacy `fx-*` services remain until a later ADR defines integration or migration.
2. **Crate naming:** Use the `fx-smc-` prefix (`fx-smc-common`, `fx-smc-marketdata`, …) to match the monorepo `fx-*` convention and avoid collisions with generic names (`common`, `api`) and with existing `fx-md` / `fx-risk`.
3. **Prices:** Domain prices and quantities are **fixed-point `i64` ticks**. Tick size and scale live in
   TOML config (no hard-coded magic in algorithms). Conversion from external feeds happens at boundaries only.
4. **Time:** Timestamps are **`i64` nanoseconds UTC**. Logical clock advances only from tick/event input
   (no wall-clock on the hot path).
5. **Persistence (M0):** Parquet for ticks/events. Postgres/`sqlx` is deferred (journal M8); `fx-smc-store`
   exposes traits so a SQL backend can be added without rewriting callers.
6. **Errors:** `thiserror` in libraries; `anyhow` in binaries. No `unwrap`/`expect` outside tests.

## Consequences

- M0 delivers skeleton + tick types + Parquet + replay harness + synth without touching matching UX.
- CI must lint/test new crates with `clippy -D warnings` (pedantic on SMC crates where practical).
- User-facing advisory copy (later milestones) must include risk, invalidation, and disclaimers — never promised returns.
