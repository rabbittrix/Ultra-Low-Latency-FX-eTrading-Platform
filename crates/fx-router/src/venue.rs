//! External venue interface

use fx_core::Order;
use fx_utils::Result;

/// External trading venue interface
pub trait Venue: Send + Sync {
    fn submit_order(&self, order: Order) -> Result<()>;
    fn cancel_order(&self, order_id: fx_utils::OrderId) -> Result<()>;
}

/// Mock venue for testing
pub struct MockVenue {
    name: String,
}

impl MockVenue {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Venue for MockVenue {
    fn submit_order(&self, _order: Order) -> Result<()> {
        tracing::info!(venue = %self.name, "Mock order submitted");
        Ok(())
    }

    fn cancel_order(&self, _order_id: fx_utils::OrderId) -> Result<()> {
        tracing::info!(venue = %self.name, "Mock order cancelled");
        Ok(())
    }
}
