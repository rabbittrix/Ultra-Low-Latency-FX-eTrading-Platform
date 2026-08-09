# ADR-0006: Prefix-only backtest (no look-ahead)

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M5 must simulate sweep→plan→fill paths without leaking future structure into past decisions.

## Decision

1. **`analyze_prefix(ticks[..=i], …)`** maps pools, detects sweeps, and builds plans on that prefix only.
2. **Emission rule:** walk `i` and keep a plan only when `as_of_idx == i` (confirm at the frontier).
3. **Fills:** entry/exit use adverse spread+slippage; exits scan **subsequent** ticks only for stop/target/time-stop.
4. **Costs:** `CostModel` from `BacktestConfig` (ticks); `CostReport` tracks spread, commission, slippage, `net_pnl_ticks`.
5. **Walk-forward:** sliding `train_len` + `test_len` windows; simulate plans confirmed in the test region only.
6. **Determinism:** same ticks + config → same report fingerprint.

## Consequences

- Correctness may be slower (re-map per prefix); research series should size windows accordingly.
- Reports are research metrics — not return promises.
