//! Build trade plans from sweeps + pools.

use crate::plan::{TradePlan, TradeSide};
use crate::trace::ReasoningTrace;
use fx_smc_common::{Px, StrategyConfig, Tick};
use fx_smc_liquidity::{LiquidityPool, PoolOrigin, PoolSide};
use fx_smc_structure::atr_proxy_ticks;
use fx_smc_sweep::SweepEvent;

/// Build filtered trade plans (deterministic order by confirm idx, then id).
#[must_use]
pub fn build_plans(
    ticks: &[Tick],
    sweeps: &[SweepEvent],
    pools: &[LiquidityPool],
    cfg: &StrategyConfig,
    disclaimer: &str,
) -> Vec<TradePlan> {
    let mut out = Vec::new();
    for (i, sw) in sweeps.iter().enumerate() {
        if let Some(plan) = plan_from_sweep(i, ticks, sw, pools, cfg, disclaimer) {
            out.push(plan);
        }
    }
    out.sort_by(|a, b| a.as_of_idx.cmp(&b.as_of_idx).then_with(|| a.id.cmp(&b.id)));
    out
}

/// Anti-sweep stop buffer: `max(legacy stop_buffer_ticks, pips + atr*num/den)`.
fn stop_extra_ticks(ticks: &[Tick], confirm_idx: usize, cfg: &StrategyConfig) -> (i64, i64) {
    let end = confirm_idx.min(ticks.len().saturating_sub(1));
    let prefix = &ticks[..=end];
    let lookback = cfg.stop_atr_lookback.max(1);
    let atr = atr_proxy_ticks(prefix, lookback);
    let den = cfg.stop_atr_den.max(1);
    let atr_buf = atr.saturating_mul(cfg.stop_atr_num.max(0)) / den;
    let computed = cfg.stop_buffer_pips_ticks.max(0).saturating_add(atr_buf);
    let legacy = cfg.stop_buffer_ticks.max(0);
    (computed.max(legacy), atr)
}

fn plan_from_sweep(
    idx: usize,
    ticks: &[Tick],
    sw: &SweepEvent,
    pools: &[LiquidityPool],
    cfg: &StrategyConfig,
    disclaimer: &str,
) -> Option<TradePlan> {
    let tick = ticks.get(sw.confirm_idx)?;
    let side = match sw.side {
        PoolSide::BuySide => TradeSide::Long,
        PoolSide::SellSide => TradeSide::Short,
    };
    let entry = tick.mid_ticks();
    let (buffer, atr_at_confirm) = stop_extra_ticks(ticks, sw.confirm_idx, cfg);
    let stop = match side {
        TradeSide::Long => Px(sw.pierce_price_ticks.saturating_sub(buffer)),
        TradeSide::Short => Px(sw.pierce_price_ticks.saturating_add(buffer)),
    };
    let risk = (entry.0 - stop.0).abs();
    if risk <= 0 {
        return None;
    }

    let (target, target_note) = select_target(side, entry, risk, pools, cfg);
    let reward = (target.0 - entry.0).abs();
    if reward <= 0 {
        return None;
    }

    let rr_den = cfg.min_rr_den.max(1);
    let rr_num = cfg.min_rr_num.max(0);
    // Pass if reward/risk >= min_rr_num/min_rr_den ⇔ reward * den >= risk * num
    if reward.saturating_mul(rr_den) < risk.saturating_mul(rr_num) {
        return None;
    }

    let origin = pools.iter().find(|p| p.id == sw.pool_id).map(|p| p.origin);
    let confluence = confluence_score(sw, origin, cfg);
    if confluence < cfg.min_confluence {
        return None;
    }

    let invalidation = match side {
        TradeSide::Long => format!(
            "Invalidation: sustained trade below stop {} ticks (pierce extreme buffered).",
            stop.0
        ),
        TradeSide::Short => format!(
            "Invalidation: sustained trade above stop {} ticks (pierce extreme buffered).",
            stop.0
        ),
    };

    let mut reasoning = ReasoningTrace::default();
    reasoning.push(
        "SWEEP",
        format!(
            "{:?} sweep on pool {} displacement={}",
            sw.side, sw.pool_id.0, sw.displacement_ticks
        ),
    );
    reasoning.push(
        "ENTRY",
        format!("mid at confirm idx {} → {}", sw.confirm_idx, entry.0),
    );
    reasoning.push(
        "STOP",
        format!(
            "stop {} risk_ticks={risk} atr_proxy={atr_at_confirm} atr_buffer=pips+atr*{}/{} extra={buffer}",
            stop.0,
            cfg.stop_atr_num,
            cfg.stop_atr_den.max(1)
        ),
    );
    reasoning.push(
        "TARGET",
        format!("{target_note} target={} reward_ticks={reward}", target.0),
    );
    reasoning.push(
        "RR",
        format!(
            "reward*den={} risk*num={} gate={rr_num}/{rr_den}",
            reward.saturating_mul(rr_den),
            risk.saturating_mul(rr_num)
        ),
    );
    reasoning.push(
        "CONF",
        format!("confluence={confluence} min={}", cfg.min_confluence),
    );
    reasoning.push("INVALIDATION", invalidation.clone());
    reasoning.push(
        "DISCLAIMER",
        "Informational plan only — not investment advice; no return is promised.",
    );

    Some(TradePlan {
        id: format!("tp-{idx}"),
        side,
        sweep_pool_id: sw.pool_id.clone(),
        entry,
        stop,
        target,
        risk_ticks: risk,
        reward_ticks: reward,
        rr_num: reward,
        rr_den: risk,
        confluence,
        as_of_ns: sw.confirm_ts_ns,
        as_of_idx: sw.confirm_idx,
        invalidation,
        disclaimer: disclaimer.to_string(),
        reasoning,
    })
}

fn select_target(
    side: TradeSide,
    entry: Px,
    risk: i64,
    pools: &[LiquidityPool],
    cfg: &StrategyConfig,
) -> (Px, String) {
    let den = cfg.fallback_rr_den.max(1);
    let num = cfg.fallback_rr_num.max(0);
    let fallback_dist = risk.saturating_mul(num) / den;
    let fallback = match side {
        TradeSide::Long => Px(entry.0.saturating_add(fallback_dist.max(1))),
        TradeSide::Short => Px(entry.0.saturating_sub(fallback_dist.max(1))),
    };

    let mut best: Option<(i64, Px)> = None;
    for p in pools {
        match side {
            TradeSide::Long => {
                if p.side == PoolSide::SellSide && p.price.0 > entry.0 {
                    let dist = p.price.0 - entry.0;
                    if best.is_none_or(|(d, _)| dist < d) {
                        best = Some((dist, p.price));
                    }
                }
            }
            TradeSide::Short => {
                if p.side == PoolSide::BuySide && p.price.0 < entry.0 {
                    let dist = entry.0 - p.price.0;
                    if best.is_none_or(|(d, _)| dist < d) {
                        best = Some((dist, p.price));
                    }
                }
            }
        }
    }

    if let Some((_, px)) = best {
        let reward = (px.0 - entry.0).abs();
        let rr_den = cfg.min_rr_den.max(1);
        let rr_num = cfg.min_rr_num.max(0);
        if reward.saturating_mul(rr_den) >= risk.saturating_mul(rr_num) {
            return (px, "opposing pool".into());
        }
    }
    (fallback, "fallback R multiple".into())
}

fn confluence_score(sw: &SweepEvent, origin: Option<PoolOrigin>, cfg: &StrategyConfig) -> i64 {
    let scale = cfg.score_scale.max(1);
    let mut pts = cfg.pts_sweep.max(0);
    match origin {
        Some(PoolOrigin::Equal) => pts = pts.saturating_add(cfg.pts_equal.max(0)),
        Some(PoolOrigin::Asia | PoolOrigin::PrevDay | PoolOrigin::Week) => {
            pts = pts.saturating_add(cfg.pts_session.max(0));
        }
        Some(PoolOrigin::Trendline) => pts = pts.saturating_add(cfg.pts_trendline.max(0)),
        None => {}
    }
    if sw.displacement_ticks >= cfg.strong_displacement_ticks {
        pts = pts.saturating_add(cfg.pts_strong_displace.max(0));
    }
    pts.clamp(0, scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::{AppConfig, Qty, SymbolId, Tick, TsNanos};
    use fx_smc_liquidity::{PoolId, PoolOrigin, PoolSide};
    use fx_smc_sweep::SweepEvent;

    fn tick(ts: i64, bid: i64, ask: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(ts),
            bid: Px(bid),
            ask: Px(ask),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        }
    }

    #[test]
    fn golden_long_plan_from_buy_side_sweep() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let pools = vec![
            LiquidityPool {
                id: PoolId::new("eq-0"),
                side: PoolSide::BuySide,
                price: Px(100),
                touches: 3,
                last_touch_ns: TsNanos(0),
                score: 8000,
                origin: PoolOrigin::Equal,
            },
            LiquidityPool {
                id: PoolId::new("eq-1"),
                side: PoolSide::SellSide,
                price: Px(140),
                touches: 2,
                last_touch_ns: TsNanos(0),
                score: 7000,
                origin: PoolOrigin::Asia,
            },
        ];

        let sweeps = [SweepEvent {
            pool_id: PoolId::new("eq-0"),
            side: PoolSide::BuySide,
            pool_price_ticks: 100,
            pierce_price_ticks: 90,
            displacement_ticks: 10,
            pierce_ts_ns: TsNanos(1),
            confirm_ts_ns: TsNanos(2),
            pierce_idx: 1,
            confirm_idx: 2,
        }];
        let ticks = [
            tick(0, 105, 107),
            tick(1, 90, 92),
            tick(2, 104, 106), // mid 105
        ];
        let plans = build_plans(&ticks, &sweeps, &pools, &cfg.strategy, &cfg.disclaimer.text);
        assert_eq!(plans.len(), 1);
        let p = &plans[0];
        assert_eq!(p.side, TradeSide::Long);
        assert_eq!(p.entry, Px(105));
        // atr on [106,91,105] ≈ (15+14)/2 = 14; extra = max(2, 2+14*1/4) = max(2,5) = 5
        assert_eq!(p.stop, Px(85)); // 90 - 5
        assert_eq!(p.risk_ticks, 20);
        assert_eq!(p.target, Px(140));
        assert!(p.confluence >= cfg.strategy.min_confluence);
        assert!(p.reasoning.steps.iter().any(|s| s.code == "DISCLAIMER"));
        assert!(p
            .reasoning
            .steps
            .iter()
            .any(|s| s.code == "STOP" && s.detail.contains("atr_buffer")));
        assert!(p.invalidation.to_ascii_lowercase().contains("invalidation"));
    }
}
