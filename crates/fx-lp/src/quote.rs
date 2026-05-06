use chrono::{DateTime, Utc};
use fx_utils::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpQuote {
    pub venue_id: String,
    pub instrument: String,
    pub bid: Price,
    pub ask: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub timestamp: DateTime<Utc>,
    pub indicative: bool,
}
