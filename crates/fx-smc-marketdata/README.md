# fx-smc-marketdata

Tick ingest helpers and a **deterministic** synthetic series generator (regimes / sweeps).

## Example

```rust
use fx_smc_common::{AppConfig, InstrumentMeta};
use fx_smc_marketdata::synth::{SynthParams, generate_ticks};

let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
let meta = cfg.instrument.default.to_meta();
let params = SynthParams::from_config(&cfg.synth, &meta);
let ticks = generate_ticks(&params);
assert_eq!(ticks.len(), cfg.synth.tick_count);
```

## Disclaimer

Synthetic series are for testing and research only. They are not market predictions.
Trading involves substantial risk of loss.
