# ADR-0011: SMC research-path hardening (benches + spike tests)

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** Project maintainers

## Context

M10 requires measurable latency on research-path kernels and resilience under spread spikes,
without claiming production hot-path zero-alloc yet for advisory crates.

## Decision

1. `fx-smc-benches` hosts Criterion benches for event hash, pool scoring, liquidity map, sweeps.
2. Integration tests cover wide-spread spikes and synth determinism under sweep injection.
3. Percentiles come from Criterion reports (`cargo bench -p fx-smc-benches`); document in README.
4. Advisory API remains cold-path Tokio; matching engine hot-path rules unchanged.

## Consequences

- Operators can track regressions on research kernels.
- Spike tests do not assert profitability — only determinism and non-panic behavior.
