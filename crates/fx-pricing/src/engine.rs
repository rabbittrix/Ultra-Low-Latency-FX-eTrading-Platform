//! Pricing engine implementation

use crate::spread::{FixedSpreadModel, SpreadModel};
use fx_md::Quote;
use fx_utils::{Price, Result};

/// Pricing engine that generates risk-adjusted prices
pub struct PricingEngine {
    spread_model: Box<dyn SpreadModel + Send + Sync>,
}

impl PricingEngine {
    pub fn new() -> Self {
        Self {
            spread_model: Box::new(FixedSpreadModel::new(2)), // 2 bps default
        }
    }

    pub fn calculate_prices(&self, base_quote: &Quote) -> Result<(Price, Price)> {
        let mid = base_quote.mid_price();
        let volatility = 0.0; // TODO: Get from AI module
        let spread = self.spread_model.calculate_spread(mid, volatility);

        let bid = Price(mid.0.saturating_sub(spread.0 / 2));
        let ask = Price(mid.0 + spread.0 / 2);

        Ok((bid, ask))
    }
}

impl Default for PricingEngine {
    fn default() -> Self {
        Self::new()
    }
}
