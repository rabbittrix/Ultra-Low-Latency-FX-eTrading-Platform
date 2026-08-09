//! Postgres persistence for profiles, journal rows, and paper stats (sqlx).
//!
//! Enable with `--features postgres`. Connection string comes from
//! `SMC_DATABASE_URL` (preferred) or `[store].postgres_url`. Secrets stay in env.

use crate::traits::{PaperStatsSnapshot, ResearchProfile};
use fx_smc_common::{SmcError, StoreConfig, TsNanos};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::env;
use std::time::Duration;

/// Async Postgres backend for research metadata (not tick hot path).
#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    /// Connect using env / config URL and apply DDL.
    ///
    /// # Errors
    /// Missing URL, connect failure, or migration SQL errors.
    pub async fn connect(store: &StoreConfig) -> Result<Self, SmcError> {
        let url = env::var("SMC_DATABASE_URL")
            .ok()
            .or_else(|| store.postgres_url.clone())
            .ok_or_else(|| {
                SmcError::Config(
                    "SMC_DATABASE_URL or store.postgres_url required for Postgres".into(),
                )
            })?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&url)
            .await
            .map_err(|e| SmcError::Store(format!("postgres connect: {e}")))?;
        let s = Self { pool };
        s.migrate().await?;
        Ok(s)
    }

    /// Apply idempotent DDL.
    ///
    /// # Errors
    /// SQL execution failures.
    pub async fn migrate(&self) -> Result<(), SmcError> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS smc_profiles (
                id TEXT PRIMARY KEY,
                risk_bps BIGINT NOT NULL,
                notes TEXT NOT NULL DEFAULT '',
                updated_ns BIGINT NOT NULL
            )
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("migrate profiles: {e}")))?;

        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS smc_journal (
                id BIGSERIAL PRIMARY KEY,
                ts_ns BIGINT NOT NULL,
                kind TEXT NOT NULL,
                plan_id TEXT,
                detail TEXT NOT NULL
            )
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("migrate journal: {e}")))?;

        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS smc_paper_stats (
                scope TEXT PRIMARY KEY,
                trades BIGINT NOT NULL,
                wins BIGINT NOT NULL,
                losses BIGINT NOT NULL,
                net_pnl_ticks BIGINT NOT NULL,
                win_rate_bps BIGINT NOT NULL,
                updated_ns BIGINT NOT NULL
            )
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("migrate paper_stats: {e}")))?;

        Ok(())
    }

    /// Borrow the pool (cold-path callers).
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert or update a research profile.
    ///
    /// # Errors
    /// SQL failures.
    pub async fn upsert_profile(&self, profile: &ResearchProfile) -> Result<(), SmcError> {
        sqlx::query(
            r"
            INSERT INTO smc_profiles (id, risk_bps, notes, updated_ns)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
                risk_bps = EXCLUDED.risk_bps,
                notes = EXCLUDED.notes,
                updated_ns = EXCLUDED.updated_ns
            ",
        )
        .bind(&profile.id)
        .bind(profile.risk_bps)
        .bind(&profile.notes)
        .bind(profile.updated_ns.0)
        .execute(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("upsert profile: {e}")))?;
        Ok(())
    }

    /// Fetch a profile by id.
    ///
    /// # Errors
    /// SQL failures.
    pub async fn get_profile(&self, id: &str) -> Result<Option<ResearchProfile>, SmcError> {
        let row =
            sqlx::query(r"SELECT id, risk_bps, notes, updated_ns FROM smc_profiles WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| SmcError::Store(format!("get profile: {e}")))?;
        Ok(row.map(|r| ResearchProfile {
            id: r.get("id"),
            risk_bps: r.get("risk_bps"),
            notes: r.get("notes"),
            updated_ns: TsNanos(r.get("updated_ns")),
        }))
    }

    /// Append a journal row; returns assigned id.
    ///
    /// # Errors
    /// SQL failures.
    pub async fn append_journal(
        &self,
        ts_ns: TsNanos,
        kind: &str,
        plan_id: Option<&str>,
        detail: &str,
    ) -> Result<i64, SmcError> {
        let row = sqlx::query(
            r"
            INSERT INTO smc_journal (ts_ns, kind, plan_id, detail)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            ",
        )
        .bind(ts_ns.0)
        .bind(kind)
        .bind(plan_id)
        .bind(detail)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("append journal: {e}")))?;
        Ok(row.get("id"))
    }

    /// List newest journal rows (desc by id).
    ///
    /// # Errors
    /// SQL failures.
    pub async fn list_journal(
        &self,
        limit: i64,
    ) -> Result<Vec<(i64, TsNanos, String, Option<String>, String)>, SmcError> {
        let lim = limit.clamp(1, 10_000);
        let rows = sqlx::query(
            r"
            SELECT id, ts_ns, kind, plan_id, detail
            FROM smc_journal
            ORDER BY id DESC
            LIMIT $1
            ",
        )
        .bind(lim)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("list journal: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.get("id"),
                    TsNanos(r.get("ts_ns")),
                    r.get("kind"),
                    r.get("plan_id"),
                    r.get("detail"),
                )
            })
            .collect())
    }

    /// Upsert paper stats for a scope key.
    ///
    /// # Errors
    /// SQL failures.
    pub async fn upsert_stats(
        &self,
        scope: &str,
        stats: &PaperStatsSnapshot,
    ) -> Result<(), SmcError> {
        sqlx::query(
            r"
            INSERT INTO smc_paper_stats
                (scope, trades, wins, losses, net_pnl_ticks, win_rate_bps, updated_ns)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (scope) DO UPDATE SET
                trades = EXCLUDED.trades,
                wins = EXCLUDED.wins,
                losses = EXCLUDED.losses,
                net_pnl_ticks = EXCLUDED.net_pnl_ticks,
                win_rate_bps = EXCLUDED.win_rate_bps,
                updated_ns = EXCLUDED.updated_ns
            ",
        )
        .bind(scope)
        .bind(i64::try_from(stats.trades).unwrap_or(i64::MAX))
        .bind(i64::try_from(stats.wins).unwrap_or(i64::MAX))
        .bind(i64::try_from(stats.losses).unwrap_or(i64::MAX))
        .bind(stats.net_pnl_ticks)
        .bind(stats.win_rate_bps)
        .bind(stats.updated_ns.0)
        .execute(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("upsert stats: {e}")))?;
        Ok(())
    }

    /// Fetch paper stats for a scope.
    ///
    /// # Errors
    /// SQL failures.
    pub async fn get_stats(&self, scope: &str) -> Result<Option<PaperStatsSnapshot>, SmcError> {
        let row = sqlx::query(
            r"
            SELECT trades, wins, losses, net_pnl_ticks, win_rate_bps, updated_ns
            FROM smc_paper_stats WHERE scope = $1
            ",
        )
        .bind(scope)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SmcError::Store(format!("get stats: {e}")))?;
        Ok(row.map(|r| {
            let trades: i64 = r.get("trades");
            let wins: i64 = r.get("wins");
            let losses: i64 = r.get("losses");
            PaperStatsSnapshot {
                trades: u64::try_from(trades.max(0)).unwrap_or(0),
                wins: u64::try_from(wins.max(0)).unwrap_or(0),
                losses: u64::try_from(losses.max(0)).unwrap_or(0),
                net_pnl_ticks: r.get("net_pnl_ticks"),
                win_rate_bps: r.get("win_rate_bps"),
                updated_ns: TsNanos(r.get("updated_ns")),
            }
        }))
    }
}
