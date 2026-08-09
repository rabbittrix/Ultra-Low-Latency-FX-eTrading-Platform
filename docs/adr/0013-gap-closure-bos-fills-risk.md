# ADR-0013: Gap closure — scenarios, anti-sweep stops, fills, Kelly/feed risk

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers
- **Related:** [ADR-0005](0005-sweep-trade-plans.md), [ADR-0006](0006-backtest-no-lookahead.md), [ADR-0007](0007-risk-sizing-kill-switch.md)

## Context

Several research-path gaps remained after the core SMC pipeline (structure → liquidity → sweep → plan → backtest → risk): named synth fixtures, ATR-aware stops, deterministic fill jitter, and Kelly / feed-quality sizing guards. Structure BOS / CHoCH detectors may land in parallel; this ADR covers the harness and risk/fill/stop closures only.

## Decision

### A — Named synth scenarios (`fx-smc-marketdata`)

`SynthScenario` + `generate_scenario(scenario, meta, seed)` emit deterministic tick paths:

| Scenario | Intent |
|----------|--------|
| `AsianRangeLondonSweep` | Flat ~80-tick range, downside pierce + reclaim |
| `FakeoutChop` | Repeated pierces without sustained reclaim |
| `CleanTrendBos` | Rising HH/HL with small gaps (FVG-friendly) |

No `f64` prices; time is `i64` ns. Seeds drive an LCG / fixed steps only.

### B — Anti-sweep stops (`StrategyConfig` / `fx-smc-strategy`)

Stop buffer beyond pierce:

`extra = max(stop_buffer_ticks, stop_buffer_pips_ticks + atr_proxy_ticks(prefix) * num / den)`

ATR is measured on `ticks[..=confirm_idx]` via `fx_smc_structure::atr_proxy_ticks`. ReasoningTrace `STOP` records the ATR buffer terms. Legacy `stop_buffer_ticks` remains as a floor.

### C — Xoshiro fills + PnL curve hash (`fx-smc-backtest`)

In-crate `Xoshiro256PlusPlus` (wrapping ops) seeded from confirm timestamp / plan id. Market entries: base slippage + local-range vol factor + jitter; sweep-proximate fills widen spread by config multipliers. Limit exits fill when mid crosses target/stop with adverse `spread/2`. `pnl_curve_fingerprint(report)` BLAKE3-hashes `(plan_id, exit_idx, pnl_ticks)`.

### D — Kelly + feed kill (`fx-smc-risk`)

`sizing_mode`: `fixed_bps` | `kelly`. Kelly uses milli win-prob / payoff / fraction math, still capped by `risk_per_trade_bps`. `KillSwitch::check_feed` / `trip_feed` trip on `max_spread_ticks` or `max_tick_latency_ns`.

## Consequences

- Config callers must supply the new TOML fields (defaults in `config/default.toml`).
- Simulations and Kelly inputs are research tools — they do not promise or predict returns.
- Named scenarios improve detector/property tests without OS entropy.
