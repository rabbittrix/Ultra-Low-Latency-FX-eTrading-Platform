# ADR-0005: Sweep-based trade plans with fixed-point R:R and ReasoningTrace

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M4 turns confirmed sweeps + mapped pools into candidate trade plans. Plans must be deterministic,
auditable, and free of `f64`. User-facing text must include invalidation and never promise returns.

## Decision

1. **Bias:** BuySide sweep → Long candidate; SellSide sweep → Short candidate.
2. **Entry:** mid at confirm tick (or ask for long / bid for short when extremes preferred later).
3. **Stop:** beyond pierce extreme by `stop_buffer_ticks`.
4. **Target:** nearest opposing-side pool beyond entry that yields R:R ≥ `min_rr_num/min_rr_den`,
   else fallback distance `fallback_rr_num/fallback_rr_den * risk`.
5. **R:R:** integer ratio `reward_ticks * den` vs `risk_ticks * num` (no float division for gates).
6. **Confluence:** sum of config point awards (sweep/equal/session/trendline/strong displace),
   clamped to `score_scale`.
7. **ReasoningTrace:** ordered steps with stable codes (`SWEEP`, `STOP`, `TARGET`, `RR`, `CONF`,
   `INVALIDATION`, `DISCLAIMER`).
8. **Filter:** emit plan only if confluence ≥ `min_confluence` and R:R gate passes.

## Consequences

- Plans are research candidates, not orders.
- Golden tests lock numeric fields for hand-crafted fixtures.
