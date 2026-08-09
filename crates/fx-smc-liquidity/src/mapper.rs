//! Map structure features into scored liquidity pools.

use crate::pool::{LiquidityPool, PoolId, PoolOrigin, PoolSide};
use crate::score::{score_pool, PoolScoreInput};
use fx_smc_common::{LiquidityConfig, LiquidityScoreConfig, Px, StructureConfig, Tick, TsNanos};
use fx_smc_structure::{
    atr_proxy_ticks, cluster_equal_levels, detect_swings, detect_trendlines, equal_tolerance_ticks,
    project_price, scan_session_levels, EqualCluster, EqualKind, SessionSnapshot, SwingPoint,
    Trendline, TrendlineSide,
};

/// Precomputed structure features used by the liquidity mapper.
#[derive(Debug, Clone)]
pub struct StructureFeatures {
    /// Detected swings.
    pub swings: Vec<SwingPoint>,
    /// Equal high/low clusters.
    pub equals: Vec<EqualCluster>,
    /// Trendlines.
    pub trendlines: Vec<Trendline>,
    /// Session snapshot.
    pub sessions: SessionSnapshot,
    /// Reference mid (last tick mid).
    pub mid: Px,
    /// Logical "now" (last tick time).
    pub as_of_ns: TsNanos,
    /// ATR proxy in ticks used for distance scoring.
    pub atr_ticks: i64,
}

impl StructureFeatures {
    /// Build features from an ordered tick series.
    #[must_use]
    pub fn from_ticks(ticks: &[Tick], structure: &StructureConfig) -> Self {
        let swings = detect_swings(ticks, &structure.swings);
        let atr = atr_proxy_ticks(ticks, structure.equal.atr_lookback);
        let tol = equal_tolerance_ticks(&structure.equal, atr);
        let equals = cluster_equal_levels(&swings, tol);
        let trendlines = detect_trendlines(&swings, &structure.trendline);
        let sessions = scan_session_levels(ticks, &structure.sessions);
        let (mid, as_of_ns) = ticks
            .last()
            .map_or((Px(0), TsNanos(0)), |t| (t.mid_ticks(), t.ts_ns));
        Self {
            swings,
            equals,
            trendlines,
            sessions,
            mid,
            as_of_ns,
            atr_ticks: atr.max(1),
        }
    }
}

/// Map structure features into scored pools (sorted by score descending).
#[must_use]
pub fn map_liquidity(
    features: &StructureFeatures,
    map_cfg: &LiquidityConfig,
    score_cfg: &LiquidityScoreConfig,
) -> Vec<LiquidityPool> {
    let mid = features.mid;
    let now = features.as_of_ns;
    let atr = features.atr_ticks.max(1);
    let mut pools = map_equal_pools(
        &features.equals,
        &features.swings,
        mid,
        now,
        atr,
        map_cfg,
        score_cfg,
    );
    pools.extend(map_trendline_pools(
        &features.trendlines,
        mid,
        now,
        atr,
        map_cfg,
        score_cfg,
    ));
    pools.extend(map_session_pools(
        &features.sessions,
        mid,
        now,
        atr,
        score_cfg,
    ));
    pools.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.0.cmp(&b.id.0)));
    if map_cfg.max_mapped_pools > 0 && pools.len() > map_cfg.max_mapped_pools {
        pools.truncate(map_cfg.max_mapped_pools);
    }
    pools
}

/// Convenience: ticks → structure → scored pools.
#[must_use]
pub fn map_from_ticks(
    ticks: &[Tick],
    structure: &StructureConfig,
    liquidity: &LiquidityConfig,
    score: &LiquidityScoreConfig,
) -> Vec<LiquidityPool> {
    let features = StructureFeatures::from_ticks(ticks, structure);
    map_liquidity(&features, liquidity, score)
}

fn map_equal_pools(
    equals: &[EqualCluster],
    swings: &[SwingPoint],
    mid: Px,
    now: TsNanos,
    atr: i64,
    map_cfg: &LiquidityConfig,
    score_cfg: &LiquidityScoreConfig,
) -> Vec<LiquidityPool> {
    let mut pools = Vec::new();
    for (i, eq) in equals.iter().enumerate() {
        if eq.members.len() < map_cfg.min_equal_members.max(1) {
            continue;
        }
        let side = match eq.kind {
            EqualKind::Highs => PoolSide::SellSide,
            EqualKind::Lows => PoolSide::BuySide,
        };
        let touches = saturating_u32(eq.members.len());
        let origin = PoolOrigin::Equal;
        let std_ticks = Some(equality_std_ticks(swings, &eq.members));
        let score = score_pool(
            &PoolScoreInput {
                price: eq.price,
                touches,
                last_touch_ns: eq.last_touch_ns,
                origin,
                mid,
                now_ns: now,
                equality_std_ticks: std_ticks,
                atr_ticks: atr,
            },
            score_cfg,
        );
        pools.push(LiquidityPool {
            id: PoolId::new(format!("eq-{i}")),
            side,
            price: eq.price,
            touches,
            last_touch_ns: eq.last_touch_ns,
            score,
            origin,
        });
    }
    pools
}

fn map_trendline_pools(
    trendlines: &[Trendline],
    mid: Px,
    now: TsNanos,
    atr: i64,
    map_cfg: &LiquidityConfig,
    score_cfg: &LiquidityScoreConfig,
) -> Vec<LiquidityPool> {
    let mut pools = Vec::new();
    for (i, tl) in trendlines.iter().enumerate() {
        if tl.touch_count() < map_cfg.min_trendline_touches.max(2) {
            continue;
        }
        let side = match tl.side {
            TrendlineSide::Resistance => PoolSide::SellSide,
            TrendlineSide::Support => PoolSide::BuySide,
        };
        let price = project_price(tl.p0, tl.t0_ns.0, tl.dp_ticks, tl.dt_ns, tl.last_touch_ns.0)
            .unwrap_or(tl.p0);
        let touches = saturating_u32(tl.touch_count());
        let origin = PoolOrigin::Trendline;
        let score = score_pool(
            &PoolScoreInput {
                price,
                touches,
                last_touch_ns: tl.last_touch_ns,
                origin,
                mid,
                now_ns: now,
                equality_std_ticks: None,
                atr_ticks: atr,
            },
            score_cfg,
        );
        pools.push(LiquidityPool {
            id: PoolId::new(format!("tl-{i}")),
            side,
            price,
            touches,
            last_touch_ns: tl.last_touch_ns,
            score,
            origin,
        });
    }
    pools
}

fn map_session_pools(
    sessions: &SessionSnapshot,
    mid: Px,
    now: TsNanos,
    atr: i64,
    score_cfg: &LiquidityScoreConfig,
) -> Vec<LiquidityPool> {
    let specs = [
        (
            "asia-h",
            sessions.asia_high,
            PoolSide::SellSide,
            PoolOrigin::Asia,
        ),
        (
            "asia-l",
            sessions.asia_low,
            PoolSide::BuySide,
            PoolOrigin::Asia,
        ),
        ("pdh", sessions.pdh, PoolSide::SellSide, PoolOrigin::PrevDay),
        ("pdl", sessions.pdl, PoolSide::BuySide, PoolOrigin::PrevDay),
        ("wh", sessions.wh, PoolSide::SellSide, PoolOrigin::Week),
        ("wl", sessions.wl, PoolSide::BuySide, PoolOrigin::Week),
    ];
    let mut pools = Vec::new();
    for (id, price, side, origin) in specs {
        if let Some(pool) = session_pool(id, price, side, origin, mid, now, atr, score_cfg) {
            pools.push(pool);
        }
    }
    pools
}

#[allow(clippy::too_many_arguments)]
fn session_pool(
    id: &str,
    price: Option<Px>,
    side: PoolSide,
    origin: PoolOrigin,
    mid: Px,
    now: TsNanos,
    atr: i64,
    score_cfg: &LiquidityScoreConfig,
) -> Option<LiquidityPool> {
    let price = price?;
    let touches = 1;
    let score = score_pool(
        &PoolScoreInput {
            price,
            touches,
            last_touch_ns: now,
            origin,
            mid,
            now_ns: now,
            equality_std_ticks: None,
            atr_ticks: atr,
        },
        score_cfg,
    );
    Some(LiquidityPool {
        id: PoolId::new(id),
        side,
        price,
        touches,
        last_touch_ns: now,
        score,
        origin,
    })
}

/// Population std-dev of member swing prices in ticks (integer sqrt).
fn equality_std_ticks(swings: &[SwingPoint], members: &[usize]) -> i64 {
    if members.len() < 2 {
        return 0;
    }
    let mut sum: i128 = 0;
    let mut n: i128 = 0;
    for &idx in members {
        if let Some(s) = swings.get(idx) {
            sum += i128::from(s.price.0);
            n += 1;
        }
    }
    if n < 2 {
        return 0;
    }
    let mean = sum / n;
    let mut var_sum: i128 = 0;
    for &idx in members {
        if let Some(s) = swings.get(idx) {
            let d = i128::from(s.price.0) - mean;
            var_sum = var_sum.saturating_add(d.saturating_mul(d));
        }
    }
    let var = var_sum / n;
    isqrt_i128(var.max(0))
}

fn isqrt_i128(n: i128) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    i64::try_from(x).unwrap_or(i64::MAX)
}

fn saturating_u32(n: usize) -> u32 {
    match u32::try_from(n) {
        Ok(v) => v,
        Err(_) => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;
    use fx_smc_marketdata::{generate_ticks, SynthParams};

    #[test]
    fn maps_synth_deterministically() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let meta = cfg.instrument.default.to_meta();
        let mut params = SynthParams::from_config(&cfg.synth, &meta);
        params.tick_count = 2_500;
        let ticks = generate_ticks(&params);
        let a = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
        let b = map_from_ticks(&ticks, &cfg.structure, &cfg.liquidity, &cfg.liquidity_score);
        assert_eq!(a, b);
        assert!(!a.is_empty());
        for w in a.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
        for p in &a {
            assert!(p.score >= 0 && p.score <= cfg.liquidity_score.score_scale);
        }
    }
}
