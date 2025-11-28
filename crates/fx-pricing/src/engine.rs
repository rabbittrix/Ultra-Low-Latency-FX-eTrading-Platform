//! Pricing engine implementation

use crate::ai_client::AiClient;
use crate::risk_adjuster::RiskAdjuster;
use crate::spread::{FixedSpreadModel, SpreadModel};
use fx_md::Quote;
use fx_risk::RiskEngine;
use fx_utils::{Price, Result};
use std::sync::Arc;

/// Pricing engine that generates risk-adjusted prices
pub struct PricingEngine {
    spread_model: Box<dyn SpreadModel + Send + Sync>,
    risk_adjuster: RiskAdjuster,
    ai_client: Option<Arc<AiClient>>,
}

impl PricingEngine {
    pub fn new() -> Self {
        Self {
            spread_model: Box::new(FixedSpreadModel::new(2)), // 2 bps default
            risk_adjuster: RiskAdjuster::new(None),
            ai_client: None,
        }
    }

    pub fn with_risk_engine(mut self, risk_engine: Arc<RiskEngine>) -> Self {
        self.risk_adjuster = RiskAdjuster::new(Some(risk_engine));
        self
    }

    pub fn with_ai_client(mut self, ai_client: Arc<AiClient>) -> Self {
        self.ai_client = Some(ai_client);
        self
    }

    pub async fn calculate_prices(&self, base_quote: &Quote) -> Result<(Price, Price)> {
        let mid = base_quote.mid_price();

        // Get volatility from AI module if available
        let volatility = if let Some(ai_client) = &self.ai_client {
            ai_client
                .predict_volatility(&base_quote.instrument)
                .await
                .unwrap_or(0.0) // Fallback to 0.0 if AI service fails
        } else {
            0.0
        };

        let spread = self.spread_model.calculate_spread(mid, volatility);

        let bid = Price(mid.0.saturating_sub(spread.0 / 2));
        let ask = Price(mid.0 + spread.0 / 2);

        // Apply risk-based adjustments
        let (adjusted_bid, adjusted_ask) =
            self.risk_adjuster
                .adjust_prices(&base_quote.instrument, bid, ask);

        Ok((adjusted_bid, adjusted_ask))
    }
}

impl Default for PricingEngine {
    fn default() -> Self {
        Self::new()
    }
}
