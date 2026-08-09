# fx-smc-liquidity

Maps SMC structure features into scored `LiquidityPool` records (M2).

Score factors (fixed-point, ADR-0012): touches, equality tightness, recency half-life,
ATR-scaled distance, origin/round-number context. Mapping thresholds live in slim
`[liquidity]`; scoring weights in `[liquidity_score]`.

## Disclaimer

Pool scores rank relative liquidity *interest* for research — they are **not** fill probabilities
or profit forecasts. Trading involves substantial risk of loss; define invalidation before acting.
