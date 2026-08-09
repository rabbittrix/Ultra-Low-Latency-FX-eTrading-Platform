# ADR-0008: Advisory regimes and suitability

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M7 ranks windows/symbols and gates plan suitability for advisory surfaces without promising returns.

## Decision

1. **`Regime`:** `TrendUp` / `TrendDown` / `Range` / `Volatile` from window mid drift + `atr_proxy_ticks`.
2. **`WindowScore`:** integer score + regime + window length; `rank_symbols` sorts by score descending.
3. **`suitability`:** confluence vs `min_suitability_confluence`, optional kill-switch check; **disclaimer reason always present**.
4. User copy must include risk / invalidation language; never promise returns.

## Consequences

- Scores are relative ranking aids, not forecasts.
- Suitability `false` under kill switch even if confluence passes.
