# fx-smc-execution

Hot-path SMC execution helpers: SPSC `rtrb` rings, `Copy` tick/intent slots, no Tokio / locks / heap growth on the trading thread.

## Disclaimer

This crate wires deterministic research intents — it does not promise fills, edge, or returns. Trading involves substantial risk of loss.
