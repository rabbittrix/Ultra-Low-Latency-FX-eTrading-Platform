use crate::book::L2Level;
use fx_core::{MatchingEngine, Order, Trade};
use fx_utils::OrderId;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct VenueExecution {
    pub order_id: OrderId,
    pub trades: Vec<Trade>,
}

pub struct ExchangeVenue {
    engine: MatchingEngine,
}

impl ExchangeVenue {
    pub fn new(instrument: String) -> Self {
        Self {
            engine: MatchingEngine::new(instrument),
        }
    }

    pub fn execute(&mut self, order: Arc<Order>) -> VenueExecution {
        let order_id = order.id;
        let result = self.engine.match_order(order);
        VenueExecution {
            order_id,
            trades: result.trades,
        }
    }

    pub fn cancel(&mut self, order_id: OrderId) -> bool {
        self.engine.cancel_order(order_id)
    }

    pub fn top_of_book(&self) -> (Option<L2Level>, Option<L2Level>) {
        let book = self.engine.orderbook();
        let bid = book.best_bid().map(|l| L2Level {
            price: l.price,
            quantity: l.total_quantity,
        });
        let ask = book.best_ask().map(|l| L2Level {
            price: l.price,
            quantity: l.total_quantity,
        });
        (bid, ask)
    }
}
