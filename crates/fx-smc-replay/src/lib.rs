//! Deterministic tick replay harness.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use fx_smc_common::{EventHash, EventHasher, LogicalClock, Tick, TsNanos};
use tracing::debug;

/// Result of replaying an ordered tick stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    /// Number of ticks consumed.
    pub ticks: usize,
    /// Final logical clock.
    pub final_ts: TsNanos,
    /// BLAKE3 digest over the canonical tick encoding.
    pub event_hash: EventHash,
}

/// Replay ticks through the logical clock and event hasher (cold-path / research).
#[must_use]
pub fn replay_ticks(ticks: &[Tick], epoch: TsNanos) -> ReplayReport {
    let mut clock = LogicalClock::new(epoch);
    let mut hasher = EventHasher::new();
    for tick in ticks {
        let _now = clock.observe(tick.ts_ns);
        hasher.absorb_tick(tick);
    }
    debug!(ticks = ticks.len(), "replay complete");
    ReplayReport {
        ticks: ticks.len(),
        final_ts: clock.now(),
        event_hash: hasher.finalize(),
    }
}

/// Assert two tick sequences produce the same event hash (determinism check).
#[must_use]
pub fn same_event_hash(a: &[Tick], b: &[Tick], epoch: TsNanos) -> bool {
    replay_ticks(a, epoch).event_hash == replay_ticks(b, epoch).event_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;
    use fx_smc_marketdata::{generate_ticks, SynthParams};
    use fx_smc_store::{ParquetTickStore, TickStore};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn live_equals_replay_via_parquet() {
        let cfg =
            AppConfig::parse_toml(include_str!("../../../config/default.toml")).expect("config");
        let meta = cfg.instrument.default.to_meta();
        let mut params = SynthParams::from_config(&cfg.synth, &meta);
        params.tick_count = 2_000;

        let live = generate_ticks(&params);
        let epoch = TsNanos(cfg.clock.epoch_ns);
        let live_report = replay_ticks(&live, epoch);

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fx_smc_replay_{nanos}"));
        let store = ParquetTickStore::new(&dir).expect("store");
        store.write_ticks("synth", &live).expect("write");
        let loaded = store.read_ticks("synth").expect("read");
        let replay_report = replay_ticks(&loaded, epoch);

        assert_eq!(live_report.event_hash, replay_report.event_hash);
        assert_eq!(live_report.ticks, replay_report.ticks);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
