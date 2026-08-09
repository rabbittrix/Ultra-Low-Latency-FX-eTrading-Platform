//! Fixed-point pool scoring (ADR-0012).

use crate::pool::{LiquidityPool, PoolOrigin};
use fx_smc_common::{LiquidityScoreConfig, Px, TsNanos};

/// Inputs for [`score_pool`] (ADR-0012).
#[derive(Debug, Clone, Copy)]
pub struct PoolScoreInput {
    /// Pool anchor price (ticks).
    pub price: Px,
    /// Touch / member count.
    pub touches: u32,
    /// Most recent touch time.
    pub last_touch_ns: TsNanos,
    /// Feature origin.
    pub origin: PoolOrigin,
    /// Reference mid (ticks).
    pub mid: Px,
    /// Scoring "now".
    pub now_ns: TsNanos,
    /// Equality cluster std in ticks; `None` → non-equal (neutral `s_eq`).
    pub equality_std_ticks: Option<i64>,
    /// ATR proxy in ticks (`>= 1` after clamping inside scorer).
    pub atr_ticks: i64,
}

/// Score a pool into `[0, score_scale]` (ADR-0012).
#[must_use]
pub fn score_pool(input: &PoolScoreInput, cfg: &LiquidityScoreConfig) -> i64 {
    let scale = cfg.score_scale.max(1);
    let s_touch = touch_factor(input.touches, cfg.touches_cap.max(1), scale);
    let s_eq = equality_factor(
        input.touches,
        input.equality_std_ticks,
        cfg.equality_tol_ticks.max(1),
        scale,
    );
    let age = input.now_ns.0.saturating_sub(input.last_touch_ns.0).max(0);
    let s_rec = half_life_decay(age, cfg.half_life_ns.max(1), scale);
    let dist_ticks = (input.price.0 - input.mid.0).abs();
    let atr = input.atr_ticks.max(1);
    let dist_atr_milli = dist_ticks.saturating_mul(1_000) / atr;
    let s_dist = distance_atr_factor(dist_atr_milli, cfg.lambda_atr_milli.max(1), scale);
    let s_ctx = context_factor(input.price, input.origin, cfg, scale);

    let weighted = cfg
        .w_touches
        .saturating_mul(s_touch)
        .saturating_add(cfg.w_equality.saturating_mul(s_eq))
        .saturating_add(cfg.w_recency.saturating_mul(s_rec))
        .saturating_add(cfg.w_distance.saturating_mul(s_dist))
        .saturating_add(cfg.w_context.saturating_mul(s_ctx));
    (weighted / scale).clamp(0, scale)
}

/// Re-score an existing pool in place (updates `score` only).
pub fn rescore(
    pool: &mut LiquidityPool,
    mid: Px,
    now_ns: TsNanos,
    atr_ticks: i64,
    equality_std_ticks: Option<i64>,
    cfg: &LiquidityScoreConfig,
) {
    pool.score = score_pool(
        &PoolScoreInput {
            price: pool.price,
            touches: pool.touches,
            last_touch_ns: pool.last_touch_ns,
            origin: pool.origin,
            mid,
            now_ns,
            equality_std_ticks,
            atr_ticks,
        },
        cfg,
    );
}

/// Approximate `scale * 0.5^(age/half_life)` with integer half-lives + linear fractional blend.
#[must_use]
pub fn half_life_decay(age_ns: i64, half_life_ns: i64, scale: i64) -> i64 {
    let hl = half_life_ns.max(1);
    let mut age = age_ns.max(0);
    let mut out = scale.max(0);
    while age >= hl {
        out /= 2;
        age -= hl;
        if out == 0 {
            return 0;
        }
    }
    // age in [0, hl): blend from `out` down to `out/2` linearly.
    let two_hl = hl.saturating_mul(2);
    out.saturating_mul(two_hl.saturating_sub(age)) / two_hl.max(1)
}

fn touch_factor(touches: u32, cap: u32, scale: i64) -> i64 {
    let t = touches.min(cap);
    i64::from(t).saturating_mul(scale) / i64::from(cap).max(1)
}

fn equality_factor(touches: u32, std_ticks: Option<i64>, tol: i64, scale: i64) -> i64 {
    let Some(std_raw) = std_ticks else {
        return scale / 2;
    };
    if touches <= 1 {
        return scale / 2;
    }
    let std = std_raw.max(0);
    let penalty = std.saturating_mul(scale) / tol.max(1);
    scale.saturating_sub(penalty.min(scale))
}

fn distance_atr_factor(dist_atr_milli: i64, lambda_milli: i64, scale: i64) -> i64 {
    // exp(-d/λ) ≈ λ/(λ+d)
    let lam = lambda_milli.max(1);
    let den = lam.saturating_add(dist_atr_milli.max(0)).max(1);
    scale.saturating_mul(lam) / den
}

fn context_factor(price: Px, origin: PoolOrigin, cfg: &LiquidityScoreConfig, scale: i64) -> i64 {
    let base = match origin {
        PoolOrigin::Asia => cfg.ctx_session,
        PoolOrigin::PrevDay | PoolOrigin::Week => cfg.ctx_pdh_pdl_wh_wl,
        PoolOrigin::Equal | PoolOrigin::Trendline => cfg.ctx_none,
    }
    .clamp(0, scale);
    let grid = cfg.round_number_ticks.max(1);
    if price.0.rem_euclid(grid) == 0 {
        base.max(cfg.ctx_round.clamp(0, scale))
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;

    fn cfg() -> LiquidityScoreConfig {
        AppConfig::parse_toml(include_str!("../../../config/default.toml"))
            .unwrap()
            .liquidity_score
    }

    #[test]
    fn fresher_and_closer_scores_higher() {
        let c = cfg();
        let now = TsNanos(10_000);
        let far_old = score_pool(
            &PoolScoreInput {
                price: Px(500),
                touches: 1,
                last_touch_ns: TsNanos(0),
                origin: PoolOrigin::Equal,
                mid: Px(100),
                now_ns: now,
                equality_std_ticks: None,
                atr_ticks: 10,
            },
            &c,
        );
        let near_fresh = score_pool(
            &PoolScoreInput {
                price: Px(101),
                touches: 4,
                last_touch_ns: TsNanos(9_500),
                origin: PoolOrigin::Asia,
                mid: Px(100),
                now_ns: now,
                equality_std_ticks: Some(0),
                atr_ticks: 10,
            },
            &c,
        );
        assert!(near_fresh > far_old);
        assert!(near_fresh <= c.score_scale);
        assert!(far_old >= 0);
    }

    #[test]
    fn half_life_halves_at_one_half_life() {
        let scale = 10_000;
        let hl = 1_000;
        assert_eq!(half_life_decay(0, hl, scale), scale);
        assert_eq!(half_life_decay(hl, hl, scale), scale / 2);
        assert_eq!(half_life_decay(hl * 2, hl, scale), scale / 4);
    }
}
