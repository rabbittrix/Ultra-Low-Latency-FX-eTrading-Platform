# fx-smc-common

Shared types for the SMC / advisory domain.

## Rules

- Prices and quantities: fixed-point `i64` ticks (never `f64` in this crate).
- Time: `i64` nanoseconds UTC (`TsNanos`).
- Hot-path consumers must not allocate from these types beyond copies of `Copy` fields.

## Example

```rust
use fx_smc_common::{InstrumentMeta, Px, Qty, Side, SymbolId, Tick, TsNanos};

let meta = InstrumentMeta {
    symbol: SymbolId::new("EURUSD"),
    price_scale: 10_000,
    tick_size: 1,
    qty_scale: 1,
};
let tick = Tick {
    symbol: meta.symbol,
    ts_ns: TsNanos(1_700_000_000_000_000_000),
    bid: Px(11_000),
    ask: Px(11_001),
    bid_qty: Qty(1_000_000),
    ask_qty: Qty(1_000_000),
    aggressor: None,
};
assert!(tick.ask.0 > tick.bid.0);
let _ = Side::Buy;
```

## Disclaimer

Types here support analysis and simulation tooling. They do not constitute investment advice.
Trading involves substantial risk of loss; always define invalidation before acting.
