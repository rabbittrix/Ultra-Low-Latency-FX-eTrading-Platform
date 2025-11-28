//! Risk limits configuration

use fx_utils::Quantity;

/// Risk limits for a trading account
#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_position_size: Quantity,
    pub max_order_size: Quantity,
    pub max_daily_loss: u64, // In quote currency
    pub max_open_orders: usize,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size: Quantity(10_000_000), // 10M units
            max_order_size: Quantity(1_000_000),     // 1M units
            max_daily_loss: 100_000,                 // 100k
            max_open_orders: 100,
        }
    }
}
