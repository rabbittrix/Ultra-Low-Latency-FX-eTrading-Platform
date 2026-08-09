# ADR-0004: Tick-level liquidity sweep detection

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M3 needs a deterministic detector that marks when price **pierces** a mapped liquidity pool and then **confirms** (reclaims) without look-ahead. Candle “wick then close” maps to a tick window, not wall-clock.

## Decision

1. **Inputs:** ordered ticks + scored `LiquidityPool`s (from M2). No `f64`; prices/distances in ticks (`i64`).
2. **Pierce (SellSide):** ask (or mid if extremes disabled) reaches `pool.price + min_pierce_ticks`.
3. **Pierce (BuySide):** bid (or mid) reaches `pool.price - min_pierce_ticks`.
4. **Confirm (pre-close):** within the next `confirm_max_ticks` ticks **after** the pierce tick, price reclaims:
   - SellSide: bid/mid ≤ `pool.price - min_reclaim_ticks`
   - BuySide: ask/mid ≥ `pool.price + min_reclaim_ticks`
5. **No look-ahead:** confirmation uses only ticks with index `> pierce_idx` and `≤ pierce_idx + confirm_max_ticks`.
6. **Output:** `SweepEvent` with pool id, side, pierce/confirm timestamps, displacement ticks, and logical indices. Fake pierces that expire the window emit nothing.
7. **Ordering:** events sorted by `(confirm_ts_ns, pool_id)` for stable hashes.

## Consequences

- Same ticks + same pools → same sweep list (replay-safe).
- Window length is a research knob in TOML; too short → missed confirms; too long → delayed signals.
- Sweep events are structural observations — **not** trade recommendations or return promises.
