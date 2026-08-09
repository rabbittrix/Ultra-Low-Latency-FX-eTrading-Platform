//! Paper trade simulator with fixed-point stats.

use crate::entry::JournalKind;
use crate::ring::Journal;
use fx_smc_common::{JournalConfig, Px, TsNanos};
use fx_smc_strategy::{TradePlan, TradeSide};
use serde::{Deserialize, Serialize};

/// Aggregate paper stats (all `i64` / counts — no `f64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PaperStats {
    /// Closed trades.
    pub trades: u64,
    /// Winning closes.
    pub wins: u64,
    /// Losing closes.
    pub losses: u64,
    /// Net `PnL` in ticks after paper slippage.
    pub net_pnl_ticks: i64,
    /// Win rate in basis points (`wins * 10000 / trades`, `0` if no trades).
    pub win_rate_bps: i64,
}

impl PaperStats {
    fn recompute_win_rate(&mut self) {
        self.win_rate_bps = if self.trades == 0 {
            0
        } else {
            i64::try_from(self.wins.saturating_mul(10_000) / self.trades).unwrap_or(0)
        };
    }
}

#[derive(Debug, Clone)]
struct OpenPaper {
    plan: TradePlan,
    entry_fill: Px,
}

/// Applies plans with paper slippage; tracks open/closed and journal.
#[derive(Debug)]
pub struct PaperSimulator {
    journal: Journal,
    open: Vec<OpenPaper>,
    stats: PaperStats,
    slippage_ticks: i64,
}

impl PaperSimulator {
    /// Create from `[journal]` config.
    #[must_use]
    pub fn from_config(cfg: &JournalConfig) -> Self {
        Self {
            journal: Journal::from_config(cfg),
            open: Vec::new(),
            stats: PaperStats::default(),
            slippage_ticks: cfg.paper_slippage_ticks.max(0),
        }
    }

    /// Borrow journal.
    #[must_use]
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    /// Open plan count.
    #[must_use]
    pub fn open_count(&self) -> usize {
        self.open.len()
    }

    /// Current stats.
    #[must_use]
    pub fn stats(&self) -> PaperStats {
        self.stats
    }

    /// Open a plan at `plan.entry` ± paper slippage (adverse).
    pub fn open_plan(&mut self, plan: TradePlan, ts_ns: TsNanos) {
        let slip = self.slippage_ticks;
        let entry_fill = match plan.side {
            TradeSide::Long => Px(plan.entry.0.saturating_add(slip)),
            TradeSide::Short => Px(plan.entry.0.saturating_sub(slip)),
        };
        let detail = format!(
            "Paper open {} {:?} entry_fill={} stop={} target={}. Invalidation: {}. Informational only — no return promised.",
            plan.id, plan.side, entry_fill.0, plan.stop.0, plan.target.0, plan.invalidation
        );
        self.journal
            .push(ts_ns, JournalKind::PlanOpen, Some(plan.id.clone()), detail);
        self.open.push(OpenPaper { plan, entry_fill });
    }

    /// Close by plan id at `exit` ± adverse slippage; updates stats.
    ///
    /// Returns `false` if plan id was not open.
    pub fn close_plan(&mut self, plan_id: &str, exit: Px, ts_ns: TsNanos) -> bool {
        let Some(pos) = self.open.iter().position(|o| o.plan.id == plan_id) else {
            return false;
        };
        let OpenPaper { plan, entry_fill } = self.open.remove(pos);
        let slip = self.slippage_ticks;
        let exit_fill = match plan.side {
            TradeSide::Long => Px(exit.0.saturating_sub(slip)),
            TradeSide::Short => Px(exit.0.saturating_add(slip)),
        };
        let pnl = match plan.side {
            TradeSide::Long => exit_fill.0.saturating_sub(entry_fill.0),
            TradeSide::Short => entry_fill.0.saturating_sub(exit_fill.0),
        };
        let won = pnl > 0;
        self.stats.trades = self.stats.trades.saturating_add(1);
        if won {
            self.stats.wins = self.stats.wins.saturating_add(1);
        } else {
            self.stats.losses = self.stats.losses.saturating_add(1);
        }
        self.stats.net_pnl_ticks = self.stats.net_pnl_ticks.saturating_add(pnl);
        self.stats.recompute_win_rate();

        let detail = format!(
            "Paper close {} exit_fill={} pnl_ticks={pnl}. Risk remains; past paper results do not predict future outcomes.",
            plan.id, exit_fill.0
        );
        self.journal
            .push(ts_ns, JournalKind::PlanClose, Some(plan.id), detail);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;
    use fx_smc_liquidity::PoolId;
    use fx_smc_strategy::{ReasoningTrace, TradeSide};

    fn hand_plan(id: &str, side: TradeSide, entry: i64, stop: i64, target: i64) -> TradePlan {
        TradePlan {
            id: id.into(),
            side,
            sweep_pool_id: PoolId::new("p"),
            entry: Px(entry),
            stop: Px(stop),
            target: Px(target),
            risk_ticks: (entry - stop).abs(),
            reward_ticks: (target - entry).abs(),
            rr_num: 2,
            rr_den: 1,
            confluence: 5000,
            as_of_ns: TsNanos(0),
            as_of_idx: 0,
            invalidation: "Invalidation: breach of stop.".into(),
            disclaimer: "Trading involves risk.".into(),
            reasoning: ReasoningTrace::default(),
        }
    }

    #[test]
    fn paper_round_trips_update_stats() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut sim = PaperSimulator::from_config(&cfg.journal);

        sim.open_plan(hand_plan("tp-a", TradeSide::Long, 100, 90, 120), TsNanos(1));
        assert!(sim.close_plan("tp-a", Px(120), TsNanos(2)));

        sim.open_plan(
            hand_plan("tp-b", TradeSide::Short, 100, 110, 80),
            TsNanos(3),
        );
        assert!(sim.close_plan("tp-b", Px(110), TsNanos(4))); // loss for short

        let s = sim.stats();
        assert_eq!(s.trades, 2);
        assert_eq!(s.wins, 1);
        assert_eq!(s.losses, 1);
        assert_eq!(s.win_rate_bps, 5000);
        assert_eq!(sim.journal().len(), 4);
        assert!(sim.journal().entries().iter().any(|e| e
            .detail
            .to_ascii_lowercase()
            .contains("invalidation")
            || e.detail.to_ascii_lowercase().contains("risk")));
    }
}
