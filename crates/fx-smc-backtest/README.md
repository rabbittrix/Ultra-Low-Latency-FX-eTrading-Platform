# fx-smc-backtest

Prefix-only SMC backtest and walk-forward with fixed-point cost model (ADR-0006).

Plans are analyzed on `ticks[..=i]` only; fills use subsequent ticks. Results are **research metrics** — not forecasts and not a promise of returns. Always retain invalidation / risk context from source plans.
