# fx-smc-structure

Market structure primitives for the SMC domain (M1):

- Swing highs / lows
- Equal highs / equal lows (configurable tick tolerance)
- Trendline liquidity (≥ `min_touches`)
- Session levels (Asia, PDH/PDL, WH/WL) in UTC
- BOS / CHoCH from confirmed swings
- Fair value gaps (FVG) on TOB bid/ask

All geometry uses fixed-point `i64` ticks and rational time slopes (see ADR-0002).

## Disclaimer

Structure maps are analytical tools only — **not** trade recommendations.
Trading involves substantial risk of loss. Always define invalidation before acting.
