//! Daily-loss and feed-quality kill switch.

use fx_smc_common::RiskConfig;
use serde::{Deserialize, Serialize};

/// Blocks new sizing once daily loss or feed quality trips.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KillSwitch {
    /// Running daily `PnL` in ticks (research / paper).
    pub daily_pnl_ticks: i64,
    /// Whether new size is blocked.
    tripped: bool,
    /// Last feed trip reason (empty if none / cleared).
    feed_reason: String,
}

impl KillSwitch {
    /// Fresh switch (not tripped).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the switch has tripped.
    #[must_use]
    pub fn is_tripped(&self) -> bool {
        self.tripped
    }

    /// Alias: allow new plans only when not tripped.
    #[must_use]
    pub fn allows_new(&self) -> bool {
        !self.tripped
    }

    /// Last feed-quality trip reason, if any.
    #[must_use]
    pub fn feed_reason(&self) -> &str {
        &self.feed_reason
    }

    /// Accumulate realized `PnL` and trip when daily loss breaches the configured limit.
    pub fn record_pnl(&mut self, pnl_ticks: i64, cfg: &RiskConfig) {
        self.daily_pnl_ticks = self.daily_pnl_ticks.saturating_add(pnl_ticks);
        let limit = cfg.max_daily_loss_ticks.max(0);
        if self.daily_pnl_ticks <= -limit {
            self.tripped = true;
        }
    }

    /// Explicitly trip on a feed-quality failure.
    pub fn trip_feed(&mut self, reason: impl Into<String>) {
        self.feed_reason = reason.into();
        self.tripped = true;
    }

    /// Check spread (ticks) and tick latency (ns); trip when configured maxima are breached.
    ///
    /// `max_spread_ticks == 0` or `max_tick_latency_ns == 0` disables that check.
    pub fn check_feed(&mut self, spread_ticks: i64, tick_latency_ns: i64, cfg: &RiskConfig) {
        if cfg.max_spread_ticks > 0 && spread_ticks > cfg.max_spread_ticks {
            self.trip_feed(format!(
                "spread {spread_ticks} > max_spread_ticks {}",
                cfg.max_spread_ticks
            ));
            return;
        }
        if cfg.max_tick_latency_ns > 0 && tick_latency_ns > cfg.max_tick_latency_ns {
            self.trip_feed(format!(
                "latency_ns {tick_latency_ns} > max_tick_latency_ns {}",
                cfg.max_tick_latency_ns
            ));
        }
    }

    /// Reset daily `PnL` and clear trip (e.g. new session).
    pub fn reset_day(&mut self) {
        self.daily_pnl_ticks = 0;
        self.tripped = false;
        self.feed_reason.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;

    #[test]
    fn trips_on_daily_loss() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut ks = KillSwitch::new();
        ks.record_pnl(-(cfg.risk.max_daily_loss_ticks - 1), &cfg.risk);
        assert!(!ks.is_tripped());
        ks.record_pnl(-1, &cfg.risk);
        assert!(ks.is_tripped());
        assert!(!ks.allows_new());
        ks.reset_day();
        assert!(ks.allows_new());
    }

    #[test]
    fn trips_on_feed_breach() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut ks = KillSwitch::new();
        ks.check_feed(cfg.risk.max_spread_ticks, 0, &cfg.risk);
        assert!(!ks.is_tripped());
        ks.check_feed(cfg.risk.max_spread_ticks + 1, 0, &cfg.risk);
        assert!(ks.is_tripped());
        assert!(ks.feed_reason().contains("spread"));

        let mut ks2 = KillSwitch::new();
        ks2.check_feed(0, cfg.risk.max_tick_latency_ns + 1, &cfg.risk);
        assert!(ks2.is_tripped());
        assert!(ks2.feed_reason().contains("latency"));
    }
}
