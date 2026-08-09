# ADR-0012: Liquidity pool and entry-window scoring

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers
- **Supersedes (scoring formulas only):** [ADR-0003](0003-liquidity-pool-scoring.md) pool factor mix (mapping still uses slim `[liquidity]`)
- **Does not overwrite:** [ADR-0001](0001-smc-domain-coexistence-and-fixed-point.md) coexistence / fixed-point prices

## Context

Research and advisory surfaces need comparable **liquidity pool** scores and **entry-window** traffic-light scores that stay deterministic across platforms. Continuous formulas (`0.5^(age/hl)`, `exp(-dist/λ)`) must be expressed with **fixed-point `i64`/`i128` only** — no `f64` prices or scores.

CHoCH / FVG detectors and live news calendars are not yet wired; confluence and news gates are **stubs** with explicit `DataDegraded` / disclaimer facts until those detectors exist.

## Decision

### Score domain

- All scores are `i64` in `[0, score_scale]` where `score_scale = 10_000` means `1.0`.
- Config weights are expressed in the same units and **must sum to `score_scale`** (validated in `AppConfig::parse_toml`).

### Fixed-point adaptations

| Continuous | Fixed-point approximation |
|------------|---------------------------|
| `0.5^(age/hl)` | Full half-lives via integer halving; fractional part via linear blend between `1` and `1/2` over one half-life (`i64`) |
| `exp(-dist_atr/λ)` | `λ / (λ + dist_atr)` with `λ` and distance in **milli-ATR** (`i64`) |

### Liquidity pool score (`LiquidityScoreConfig` / `[liquidity_score]`)

Input: price, touches, last touch, origin, mid, now, optional equality std (ticks), ATR (ticks).

Components (each in `[0, scale]`):

1. **`s_touch`** = `min(touches, cap) * scale / cap`
2. **`s_eq`** = if `touches ≤ 1` or equality std absent → `scale/2`; else `scale - min(std * scale / tol, scale)`
3. **`s_rec`** = `half_life_decay(age_ns, half_life_ns, scale)` (default half-life 24h)
4. **`s_dist`** = `scale * λ_milli / (λ_milli + dist_atr_milli)` where `dist_atr_milli = |px-mid| * 1000 / max(atr, 1)` (default `λ = 2.0` ATR → `2000` milli)
5. **`s_ctx`** from origin (`session` / `pdh_pdl_wh_wl` / `none`); if price is on a round-number grid, `max(s_ctx, ctx_round)`

Combine:

```text
score = (w_touch·s_touch + w_eq·s_eq + w_rec·s_rec + w_dist·s_dist + w_ctx·s_ctx) / scale
```

Default weights: touches `2500`, equality `1500`, recency `2000`, distance `2500`, context `1500`.

`[liquidity]` is **mapping-only**: `min_equal_members`, `min_trendline_touches`, `max_mapped_pools`.

### Entry-window score (`WindowScoreConfig` / `[window_score]`)

`score_entry_window(side, …)` builds `EntryWindowScore { side, raw, color, facts, summary }`.

Components:

1. **`s_sweep`** = max over **opposite-side** confirmed sweeps of `pool.score * half_life_decay(age, sweep_hl) / scale` (ages beyond `sweep_max_age` ignored for this max; default hl 6h, max age 12h)
2. **`s_conf`** from `ConfSignal` (`ChoCh` / `Bos` / `HtfBias` / `None`) — **ChoCh/Bos stub** until structure detectors land
3. **`s_rr`** = clamp `rr_est_milli` into `[rr_floor, rr_cap]` then map linearly onto `[0, scale]` (defaults floor `1.0R`, cap `5.0R`; gate uses `min_rr = 3.0R`)
4. **`s_reg`** = regime alignment with entry side
5. **`s_sess`** = kill-zone / normal / low (London kill-zone stub: UTC hours `[7, 10)`)

Same weighted-sum `/ scale` form. Default weights: sweep `3500`, conf `2000`, rr `2000`, regime `1500`, session `1000`.

**Gates (applied before / as color caps):**

| Gate | Condition | Effect |
|------|-----------|--------|
| G1 | No confirmed opposite sweep within `sweep_max_age` | color ≤ Yellow |
| G2 | `rr_est < min_rr` | color ≤ Yellow |
| G3 | `vol_above_p95` OR `news_high_impact` | **Red** (news blackout stub; `news_blackout_min` reserved) |
| G4 | G1 **and** G2 | **Red** |

Then thresholds: `raw ≥ thr_green` → Green; `≥ thr_yellow` → Yellow; else Red (subject to gate caps). Defaults: green `7500`, yellow `5500`.

`best_entry_window` returns the higher-scoring side (stable tie-break: Buy then Sell comparison by `raw`).

### Facts / stubs

`Fact` variants include sweep/conf/rr/session/regime contributions, gate hits, `DataDegraded` (e.g. `ConfSignal::None`), and a mandatory **disclaimer**. FVG / live CHoCH / news calendar remain stubs.

### Sanity pattern (§3)

Injected inputs (not live detectors) that yield raw ≈ `0.865 * scale` (`~8650`) must color **Green** when gates are clear — covered by a golden unit test in `fx-smc-advisory`.

## Consequences

- ADR-0003 factor mix is replaced for new scoring; old `LiquidityConfig` weight fields are removed.
- Call sites must pass both mapping (`LiquidityConfig`) and scoring (`LiquidityScoreConfig`) configs into `map_from_ticks`.
- Advisory API exposes `window_color`, `window_raw`, `window_side`, and `facts` for UI traffic lights.
- Scores remain research ranking aids — **not** forecasts of fill probability or PnL.
