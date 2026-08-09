# ADR-0009: Research journal and paper simulator

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M8 needs an auditable, capped event log and a paper book with fixed-point statistics.

## Decision

1. **`JournalEntry`:** `id`, `ts_ns`, `kind`, `plan_id`, `detail`; ring capacity `max_entries`.
2. **`PaperSimulator`:** open/close plans with `paper_slippage_ticks` adverse fills.
3. **`PaperStats`:** `trades`, `wins`, `losses`, `net_pnl_ticks`, `win_rate_bps` (`wins * 10000 / trades`) — no `f64`.
4. Paper results are process metrics; copy must not promise live returns.

## Consequences

- Oldest journal rows drop when over capacity.
- Win rate in bps avoids floating point at the stats boundary.
