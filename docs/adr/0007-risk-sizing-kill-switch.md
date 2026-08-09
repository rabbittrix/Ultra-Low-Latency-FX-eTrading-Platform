# ADR-0007: Fixed-point risk sizing and kill switch

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M6 needs deterministic position sizing without `f64`, plus hard guardrails for research / paper flows.

## Decision

1. **`size_qty`:** `risk_ticks = equity_ticks * risk_per_trade_bps / 10000`; `qty = risk_ticks / stop_distance` (`i64`).
2. **Guardrails:** reject when `stop_distance < min_stop_ticks` or `open_plans >= max_open_plans`.
3. **`KillSwitch`:** trip when cumulative daily PnL ≤ `-max_daily_loss_ticks`; block new size while tripped.
4. Sizing never places orders and never implies expected returns.

## Consequences

- Qty may be zero for tiny equity / wide stops — callers must handle reject / zero.
- Kill switch is a session brake, not a guarantee against loss.
