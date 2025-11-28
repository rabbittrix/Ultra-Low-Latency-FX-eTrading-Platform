//! Order router implementation

use crate::venue::{MockVenue, Venue};
use fx_core::Order;
use fx_utils::Result;
use std::sync::Arc;

/// Order router that routes orders to external venues
pub struct OrderRouter {
    venues: Vec<Arc<dyn Venue>>,
}

impl OrderRouter {
    pub fn new() -> Self {
        let venues: Vec<Arc<dyn Venue>> = vec![
            Arc::new(MockVenue::new("MockVenue1".to_string())),
            Arc::new(MockVenue::new("MockVenue2".to_string())),
        ];

        Self { venues }
    }

    pub fn route_order(&self, order: Order) -> Result<()> {
        // Simple round-robin routing for now
        if let Some(venue) = self.venues.first() {
            venue.submit_order(order)?;
        }
        Ok(())
    }
}

impl Default for OrderRouter {
    fn default() -> Self {
        Self::new()
    }
}
