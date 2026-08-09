//! Plan suitability with mandatory disclaimer reason.

use fx_smc_common::AdvisoryConfig;
use fx_smc_risk::KillSwitch;
use serde::{Deserialize, Serialize};

/// Suitability verdict for a candidate plan (informational only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suitability {
    /// Whether confluence / risk gates passed.
    pub suitable: bool,
    /// Human-readable reasons (always includes disclaimer / risk language).
    pub reasons: Vec<String>,
}

/// Evaluate suitability from confluence and optional kill switch.
///
/// Always appends a disclaimer reason — never promises returns.
#[must_use]
pub fn suitability(
    plan_confluence: i64,
    cfg: &AdvisoryConfig,
    kill: Option<&KillSwitch>,
    disclaimer: &str,
) -> Suitability {
    let mut reasons = Vec::new();
    let mut suitable = true;

    if plan_confluence < cfg.min_suitability_confluence {
        suitable = false;
        reasons.push(format!(
            "Confluence {plan_confluence} below min_suitability_confluence {}",
            cfg.min_suitability_confluence
        ));
    } else {
        reasons.push(format!(
            "Confluence {plan_confluence} meets min {}",
            cfg.min_suitability_confluence
        ));
    }

    if let Some(ks) = kill {
        if ks.is_tripped() {
            suitable = false;
            reasons.push(
                "Kill switch tripped — sizing blocked; treat as unsuitable until reset.".into(),
            );
        }
    }

    reasons.push(format!(
        "Disclaimer: {disclaimer} Define invalidation before any action; no return is promised."
    ));

    Suitability { suitable, reasons }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fx_smc_common::AppConfig;

    #[test]
    fn disclaimer_always_present() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let s = suitability(
            cfg.advisory.min_suitability_confluence,
            &cfg.advisory,
            None,
            &cfg.disclaimer.text,
        );
        assert!(s.suitable);
        assert!(s
            .reasons
            .iter()
            .any(|r| r.to_ascii_lowercase().contains("disclaimer")));
        assert!(s
            .reasons
            .iter()
            .any(|r| r.to_ascii_lowercase().contains("risk")
                || r.to_ascii_lowercase().contains("invalidation")));
    }

    #[test]
    fn kill_switch_blocks() {
        let cfg = AppConfig::parse_toml(include_str!("../../../config/default.toml")).unwrap();
        let mut kill = KillSwitch::new();
        kill.record_pnl(-cfg.risk.max_daily_loss_ticks, &cfg.risk);
        let s = suitability(10_000, &cfg.advisory, Some(&kill), &cfg.disclaimer.text);
        assert!(!s.suitable);
    }
}
