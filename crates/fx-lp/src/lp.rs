use crate::quote::LpQuote;
use chrono::Utc;
use fx_utils::{Price, Quantity, Side};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuoteMode {
    Streaming,
    Rfq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpVenue {
    pub venue_id: String,
    pub quote_mode: QuoteMode,
    pub score: i64,
    pub last_look_enabled: bool,
}

#[derive(Default)]
pub struct LpEngine {
    venues: Vec<LpVenue>,
}

impl LpEngine {
    pub fn new(venues: Vec<LpVenue>) -> Self {
        Self { venues }
    }

    pub fn best_venue(&self, mode: QuoteMode) -> Option<&LpVenue> {
        self.venues
            .iter()
            .filter(|v| v.quote_mode == mode)
            .max_by_key(|v| v.score)
    }

    pub fn generate_indicative_quote(
        &self,
        venue_id: &str,
        instrument: &str,
        mid_price: Price,
        spread_bps: u64,
        qty: Quantity,
        _side: Side,
    ) -> LpQuote {
        let spread = (mid_price.0.saturating_mul(spread_bps)).max(1) / 10_000;
        let bid = Price(mid_price.0.saturating_sub(spread));
        let ask = Price(mid_price.0.saturating_add(spread));

        LpQuote {
            venue_id: venue_id.to_owned(),
            instrument: instrument.to_owned(),
            bid,
            ask,
            bid_size: qty,
            ask_size: qty,
            timestamp: Utc::now(),
            indicative: true,
        }
    }
}
