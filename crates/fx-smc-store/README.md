# fx-smc-store

Parquet persistence for SMC ticks/events. Optional Postgres (`--features postgres`) stores research profiles, journal rows, and paper stats via `sqlx`. Connection: `SMC_DATABASE_URL` (preferred) or `[store].postgres_url`.

## Disclaimer

Stored research data is not a performance guarantee. Trading involves substantial risk of loss.
