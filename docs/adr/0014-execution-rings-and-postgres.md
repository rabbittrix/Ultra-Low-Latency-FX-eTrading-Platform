# ADR-0014: Hot-path execution rings and Postgres research store

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers
- **Related:** [ADR-0001](0001-smc-domain-coexistence-and-fixed-point.md), [ADR-0009](0009-journal-paper-simulator.md), [ADR-0013](0013-gap-closure-bos-fills-risk.md)

## Context

The SMC research stack needed (1) a true hot-path crate with SPSC rings and `Copy` slots, and (2) Postgres persistence for profiles / journal / paper stats while ticks remain on Parquet.

## Decision

### A — `fx-smc-execution`

- `rtrb` SPSC pairs (`spsc_pair`) carry `TickSlot` and `ExecIntent` (`Copy`, fixed layout).
- `HotPathEngine::drain_ticks` runs without Tokio, locks, or heap growth; emits research intents when spread ≤ cap and mid moves.
- Dual-run BLAKE3 of intent fingerprints proves determinism for identical tick sequences.
- Not a live OMS — intents are research / paper wiring only; no returns promised.

### B — Postgres via `sqlx` (`fx-smc-store` feature `postgres`)

- Parquet remains the tick/event store.
- Optional `PostgresStore` tables: `smc_profiles`, `smc_journal`, `smc_paper_stats`.
- Connection: `SMC_DATABASE_URL` (preferred) or `[store].postgres_url`. Secrets stay in env.
- Cold path only (async); never on the execution thread.

### C — Structure confluence wiring

- BOS/CHoCH/FVG live in `fx-smc-structure`; advisory `conf_from_structure_breaks` feeds WindowScore gates (ADR-0012).

## Consequences

- Default builds omit `sqlx` unless `--features postgres`.
- Hot-path callers must pre-size rings; `IntentRingFull` is back-pressure, not a retry loop with alloc.
- User-facing surfaces keep risk / invalidation / disclaimer language.
