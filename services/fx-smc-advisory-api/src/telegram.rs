//! Optional Telegram alerts (secrets from environment only).

use crate::dto::AnalyzeResponse;
use anyhow::{bail, Result};
use fx_smc_common::AppConfig;
use tracing::info;

/// Send a short Telegram summary when enabled and env secrets are present.
pub async fn maybe_send_telegram(cfg: &AppConfig, resp: &AnalyzeResponse) -> Result<()> {
    if !cfg.api.telegram_enabled {
        return Ok(());
    }
    let token = match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => {
            info!("telegram enabled in config but TELEGRAM_BOT_TOKEN unset — skip");
            return Ok(());
        }
    };
    let chat = match std::env::var("TELEGRAM_CHAT_ID") {
        Ok(c) if !c.is_empty() => c,
        _ => {
            info!("TELEGRAM_CHAT_ID unset — skip");
            return Ok(());
        }
    };

    let text = format!(
        "SMC analyze (research only — not advice)\n\
         ticks={} pools={} sweeps={} plans={}\n\
         regime={} window_score={}\n\
         {}",
        resp.tick_count,
        resp.pool_count,
        resp.sweeps.len(),
        resp.plans.len(),
        resp.regime.label,
        resp.window.score,
        resp.disclaimer
    );

    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&serde_json::json!({
            "chat_id": chat,
            "text": text,
            "disable_web_page_preview": true
        }))
        .send()
        .await?;
    if !res.status().is_success() {
        bail!("telegram HTTP {}", res.status());
    }
    Ok(())
}
