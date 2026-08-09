//! SMC M0 replay CLI: generate synthetic ticks, persist Parquet, print event hash.
//!
//! Disclaimer: output is for pipeline validation only — not investment advice.
//! Trading involves substantial risk of loss; always define invalidation before acting.

use anyhow::{Context, Result};
use fx_smc_common::{AppConfig, TsNanos};
use fx_smc_marketdata::{generate_ticks, SynthParams};
use fx_smc_replay::replay_ticks;
use fx_smc_store::{ParquetTickStore, TickStore};
use std::env;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let config_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/default.toml"));

    let cfg = AppConfig::load_path(&config_path)
        .with_context(|| format!("load config {}", config_path.display()))?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cfg.tracing.default_filter.clone()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!(disclaimer = %cfg.disclaimer.text, "user-facing risk notice");

    let meta = cfg.instrument.default.to_meta();
    let params = SynthParams::from_config(&cfg.synth, &meta);
    let ticks = generate_ticks(&params);
    info!(count = ticks.len(), symbol = %meta.symbol.as_str(), "generated synthetic ticks");

    let store = ParquetTickStore::new(&cfg.store.parquet_dir).context("create parquet store")?;
    store
        .write_ticks("synth_m0", &ticks)
        .context("write parquet")?;
    let loaded = store.read_ticks("synth_m0").context("read parquet")?;

    let report = replay_ticks(&loaded, TsNanos(cfg.clock.epoch_ns));
    println!("ticks={}", report.ticks);
    println!("final_ts_ns={}", report.final_ts.0);
    println!("event_hash={}", report.event_hash.to_hex());
    println!("disclaimer={}", cfg.disclaimer.text);

    Ok(())
}
