# ADR-0003: Fixed-point liquidity pool scoring

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M2 maps structure features into `LiquidityPool` records with a comparable `score`.
Scores must be deterministic and free of `f64`.

## Decision

1. **Score domain:** unsigned-friendly `i64` in \[0, `score_scale`\] (default `10_000`).
2. **Factors** (each mapped to \[0, `score_scale`\] then combined):
   - **touches:** `min(touches, max_touches) * scale / max_touches`
   - **recency:** `scale * half_life_ns / (half_life_ns + age_ns)` (integer)
   - **distance:** closer to reference mid → higher:
     `scale * (max_dist - min(|px - mid|, max_dist)) / max_dist`
   - **session:** `scale` if pool origin is a session level, else `session_off_score`
3. **Combine:** weighted sum with config weights (`w_touches`, …), divided by weight sum
   (all `i64`, saturating). Weights are required to be `>= 0`; if sum is 0, score is 0.
4. **Pool side:**
   - `SellSide` = liquidity above price (equal highs, resistance, PDH/WH/Asia high)
   - `BuySide` = liquidity below price (equal lows, support, PDL/WL/Asia low)

## Consequences

- Stable ordering across platforms for the same inputs.
- Quantization vs continuous decay; tune via TOML half-life and weights.
- Scores are relative ranks for research/strategy — **not** forecasts of fill probability or PnL.
