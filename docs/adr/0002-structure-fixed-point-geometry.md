# ADR-0002: Fixed-point geometry for structure (swings / trendlines)

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M1 structure algorithms need distances and trendline slopes. Floating-point (`f64`) is forbidden for prices and for hot-path geometry that must stay deterministic across platforms.

## Decision

1. **Prices** remain `Px(i64)` ticks. **Time** remains `TsNanos(i64)`.
2. **Equal-level tolerance** is computed in ticks:
   `tolerance = max(pips_min_ticks, (k_atr_num * atr_ticks) / k_atr_den)` with integer division.
   ATR proxy in M1: mean absolute mid-change over a configurable lookback window (all `i64`).
3. **Trendline slope** is stored as a rational `(dp_ticks: i64, dt_ns: i64)` between two anchors.
   Price at time `t` is projected with saturating `i128` arithmetic:
   `p = p0 + dp * (t - t0) / dt` (when `dt != 0`), never via floating point.
4. **Session clocks** use UTC hour/minute derived from `ts_ns` integer division (no timezone DB in M1).
   Session windows are configured as UTC hour ranges in TOML; later ADRs may add IANA TZ.

## Consequences

- Bit-identical structure outputs on all targets for the same tick input.
- Slight quantization vs classical float ATR/slope; acceptable for SMC mapping and replay hashes.
- Trendlines with `dt_ns == 0` are rejected as invalid.
