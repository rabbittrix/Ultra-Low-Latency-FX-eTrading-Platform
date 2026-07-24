use fx_core::{Price, Quantity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct L2Level {
    pub price: Price,
    pub quantity: Quantity,
}
