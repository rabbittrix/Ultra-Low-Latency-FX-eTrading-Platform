//! Risk-based price adjustment

use fx_risk::RiskEngine;
use fx_utils::Price;

/// Risk-based price adjuster that widens spreads based on position risk
pub struct RiskAdjuster {
    risk_engine: Option<std::sync::Arc<RiskEngine>>,
}

impl RiskAdjuster {
    pub fn new(risk_engine: Option<std::sync::Arc<RiskEngine>>) -> Self {
        Self { risk_engine }
    }

    /// Adjust prices based on current position risk
    pub fn adjust_prices(&self, instrument: &str, bid: Price, ask: Price) -> (Price, Price) {
        if let Some(risk_engine) = &self.risk_engine {
            // Get current position
            let position = risk_engine.get_position(instrument);

            // Calculate risk adjustment factor
            // Wider spreads for larger positions (riskier)
            let position_risk_factor = self.calculate_position_risk_factor(position);

            // Adjust spread based on position
            let mid = (bid.0 + ask.0) / 2;
            let base_spread = ask.0.saturating_sub(bid.0);
            let adjusted_spread = (base_spread as f64 * position_risk_factor) as u64;

            let adjusted_bid = Price(mid.saturating_sub(adjusted_spread / 2));
            let adjusted_ask = Price(mid + adjusted_spread / 2);

            (adjusted_bid, adjusted_ask)
        } else {
            // No risk engine, return original prices
            (bid, ask)
        }
    }

    fn calculate_position_risk_factor(&self, position: i64) -> f64 {
        // Base factor: 1.0 (no adjustment)
        // Increase spread by up to 50% for large positions
        let abs_position = position.abs() as f64;
        let max_position = 10_000_000.0; // 10M units
        let risk_ratio = (abs_position / max_position).min(1.0);
        1.0 + (risk_ratio * 0.5) // 1.0 to 1.5 multiplier
    }
}
