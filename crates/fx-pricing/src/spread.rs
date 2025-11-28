//! Spread calculation models

use fx_utils::Price;

/// Spread model for calculating bid/ask spreads
pub trait SpreadModel {
    fn calculate_spread(&self, mid_price: Price, volatility: f64) -> Price;
}

/// Simple fixed spread model
pub struct FixedSpreadModel {
    spread_bps: u64, // Spread in basis points
}

impl FixedSpreadModel {
    pub fn new(spread_bps: u64) -> Self {
        Self { spread_bps }
    }
}

impl SpreadModel for FixedSpreadModel {
    fn calculate_spread(&self, mid_price: Price, _volatility: f64) -> Price {
        // Calculate spread as basis points of mid price
        let spread = (mid_price.0 * self.spread_bps) / 10000;
        Price(spread)
    }
}
