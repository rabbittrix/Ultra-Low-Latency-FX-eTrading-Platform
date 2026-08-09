//! Session levels: Asia, previous-day high/low, week high/low (UTC).

use fx_smc_common::{Px, SessionConfig, Tick, TsNanos};
use serde::{Deserialize, Serialize};

const NS_PER_SEC: i64 = 1_000_000_000;
const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_HOUR: i64 = 3_600;

/// Snapshot of session reference levels at a point in the series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Timestamp of the last consumed tick.
    pub as_of_ns: TsNanos,
    /// Asia session high (ticks), if any tick fell in the Asia window so far today/prior days tracked.
    pub asia_high: Option<Px>,
    /// Asia session low.
    pub asia_low: Option<Px>,
    /// Previous calendar day (UTC) high.
    pub pdh: Option<Px>,
    /// Previous calendar day (UTC) low.
    pub pdl: Option<Px>,
    /// Current UTC week high (Monday 00:00 UTC start).
    pub wh: Option<Px>,
    /// Current UTC week low.
    pub wl: Option<Px>,
}

/// Running session level tracker.
#[derive(Debug, Clone, Default)]
pub struct SessionLevels {
    asia_high: Option<Px>,
    asia_low: Option<Px>,
    day_high: Option<Px>,
    day_low: Option<Px>,
    prev_day_high: Option<Px>,
    prev_day_low: Option<Px>,
    week_high: Option<Px>,
    week_low: Option<Px>,
    current_day: Option<i64>,
    current_week: Option<i64>,
    last_ts: TsNanos,
}

impl SessionLevels {
    /// Create an empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Consume one tick and update session extremes.
    pub fn on_tick(&mut self, tick: &Tick, cfg: &SessionConfig) {
        let mid = tick.mid_ticks();
        let ts = tick.ts_ns;
        self.last_ts = ts;

        let day = utc_day_index(ts.0);
        let week = utc_week_index(ts.0);

        if self.current_day == Some(day) {
            self.day_high = Some(max_px(self.day_high, mid));
            self.day_low = Some(min_px(self.day_low, mid));
        } else {
            if cfg.track_pdh_pdl {
                self.prev_day_high = self.day_high;
                self.prev_day_low = self.day_low;
            }
            self.day_high = Some(mid);
            self.day_low = Some(mid);
            self.current_day = Some(day);
            // Reset Asia for the new day.
            self.asia_high = None;
            self.asia_low = None;
        }

        if cfg.track_wh_wl {
            if self.current_week == Some(week) {
                self.week_high = Some(max_px(self.week_high, mid));
                self.week_low = Some(min_px(self.week_low, mid));
            } else {
                self.week_high = Some(mid);
                self.week_low = Some(mid);
                self.current_week = Some(week);
            }
        }

        if in_asia_window(ts.0, cfg.asia_start_hour_utc, cfg.asia_end_hour_utc) {
            self.asia_high = Some(max_px(self.asia_high, mid));
            self.asia_low = Some(min_px(self.asia_low, mid));
        }
    }

    /// Current snapshot.
    #[must_use]
    pub fn snapshot(&self, cfg: &SessionConfig) -> SessionSnapshot {
        SessionSnapshot {
            as_of_ns: self.last_ts,
            asia_high: self.asia_high,
            asia_low: self.asia_low,
            pdh: if cfg.track_pdh_pdl {
                self.prev_day_high
            } else {
                None
            },
            pdl: if cfg.track_pdh_pdl {
                self.prev_day_low
            } else {
                None
            },
            wh: if cfg.track_wh_wl {
                self.week_high
            } else {
                None
            },
            wl: if cfg.track_wh_wl { self.week_low } else { None },
        }
    }
}

/// Scan an entire tick series and return the final session snapshot.
#[must_use]
pub fn scan_session_levels(ticks: &[Tick], cfg: &SessionConfig) -> SessionSnapshot {
    let mut s = SessionLevels::new();
    for t in ticks {
        s.on_tick(t, cfg);
    }
    s.snapshot(cfg)
}

fn max_px(cur: Option<Px>, v: Px) -> Px {
    match cur {
        None => v,
        Some(c) => Px(c.0.max(v.0)),
    }
}

fn min_px(cur: Option<Px>, v: Px) -> Px {
    match cur {
        None => v,
        Some(c) => Px(c.0.min(v.0)),
    }
}

fn utc_day_index(ts_ns: i64) -> i64 {
    let secs = ts_ns.div_euclid(NS_PER_SEC);
    secs.div_euclid(SECS_PER_DAY)
}

fn utc_week_index(ts_ns: i64) -> i64 {
    // Unix epoch Thursday; Monday-based week: (day + 3) / 7 for Thursday epoch → Monday weeks
    // day_index 0 = 1970-01-01 Thursday. Monday-start week index:
    let day = utc_day_index(ts_ns);
    (day + 3).div_euclid(7)
}

fn utc_hour(ts_ns: i64) -> u8 {
    let secs = ts_ns.div_euclid(NS_PER_SEC);
    let sod = secs.rem_euclid(SECS_PER_DAY);
    let h = sod.div_euclid(SECS_PER_HOUR);
    u8::try_from(h).unwrap_or(0)
}

fn in_asia_window(ts_ns: i64, start: u8, end: u8) -> bool {
    let h = utc_hour(ts_ns);
    if start == end {
        return true;
    }
    if start < end {
        h >= start && h < end
    } else {
        // wraps midnight
        h >= start || h < end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{Qty, SymbolId};

    fn tick_at(mid: i64, ts_ns: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(ts_ns),
            bid: Px(mid),
            ask: Px(mid + 1),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }
    }

    #[test]
    fn asia_and_day_roll() {
        let cfg = SessionConfig {
            asia_start_hour_utc: 0,
            asia_end_hour_utc: 7,
            track_pdh_pdl: true,
            track_wh_wl: true,
        };
        // Day 0 03:00 UTC
        let t0 = 3 * SECS_PER_HOUR * NS_PER_SEC;
        // Day 0 12:00 UTC (outside Asia)
        let t1 = 12 * SECS_PER_HOUR * NS_PER_SEC;
        // Day 1 04:00 UTC
        let t2 = (SECS_PER_DAY + 4 * SECS_PER_HOUR) * NS_PER_SEC;

        let mut s = SessionLevels::new();
        s.on_tick(&tick_at(100, t0), &cfg);
        s.on_tick(&tick_at(110, t1), &cfg);
        let mid = s.snapshot(&cfg);
        assert_eq!(mid.asia_high, Some(Px(100)));
        assert_eq!(mid.asia_low, Some(Px(100)));

        s.on_tick(&tick_at(90, t2), &cfg);
        let end = s.snapshot(&cfg);
        assert_eq!(end.pdh, Some(Px(110)));
        assert_eq!(end.pdl, Some(Px(100)));
        assert_eq!(end.asia_high, Some(Px(90)));
    }

    #[test]
    fn session_high_bounds_mid() {
        // start == end ⇒ treat as full-day Asia window for tests.
        let cfg = SessionConfig {
            asia_start_hour_utc: 0,
            asia_end_hour_utc: 0,
            track_pdh_pdl: false,
            track_wh_wl: true,
        };
        let ticks = vec![
            tick_at(50, 0),
            tick_at(80, NS_PER_SEC),
            tick_at(60, 2 * NS_PER_SEC),
        ];
        let snap = scan_session_levels(&ticks, &cfg);
        assert_eq!(snap.asia_high, Some(Px(80)));
        assert_eq!(snap.asia_low, Some(Px(50)));
        assert_eq!(snap.wh, Some(Px(80)));
        assert_eq!(snap.wl, Some(Px(50)));
    }
}
