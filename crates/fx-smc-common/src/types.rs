//! Fixed-point market primitives (no floating-point prices).

use serde::{Deserialize, Serialize};

/// Nanoseconds since Unix epoch (UTC), or logical nanos in replay.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct TsNanos(pub i64);

impl TsNanos {
    /// Saturating add of a non-negative delta.
    #[must_use]
    pub const fn saturating_add_ns(self, delta_ns: i64) -> Self {
        let d = if delta_ns < 0 { 0 } else { delta_ns };
        Self(self.0.saturating_add(d))
    }
}

/// Instrument symbol (ASCII, uppercase preferred at boundaries).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub String);

impl SymbolId {
    /// Construct from any string-like value.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the inner symbol text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Price in instrument ticks (`i64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Px(pub i64);

/// Quantity in instrument quantity ticks (`i64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Qty(pub i64);

/// Trade aggressor side when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// Bid lifted / buy aggressor.
    Buy,
    /// Offer hit / sell aggressor.
    Sell,
}

/// Static instrument metadata (tick size from config, not hard-coded in algorithms).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentMeta {
    /// Symbol identifier.
    pub symbol: SymbolId,
    /// Scale factor for converting human decimal quotes to ticks (documentation only in M0).
    pub price_scale: i64,
    /// Minimum price increment in ticks.
    pub tick_size: i64,
    /// Scale for quantities.
    pub qty_scale: i64,
}

/// One top-of-book / trade tick for the SMC pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tick {
    /// Instrument.
    pub symbol: SymbolId,
    /// Event time (UTC nanos or logical nanos).
    pub ts_ns: TsNanos,
    /// Best bid in ticks.
    pub bid: Px,
    /// Best ask in ticks.
    pub ask: Px,
    /// Bid size in qty ticks.
    pub bid_qty: Qty,
    /// Ask size in qty ticks.
    pub ask_qty: Qty,
    /// Optional trade aggressor.
    pub aggressor: Option<Side>,
}

impl Tick {
    /// Mid price in ticks using integer averaging (floor toward bid on odd spreads).
    #[must_use]
    pub fn mid_ticks(&self) -> Px {
        Px(self.bid.0.saturating_add(self.ask.0) / 2)
    }

    /// Spread in ticks (`ask - bid`), saturating at zero if crossed.
    #[must_use]
    pub fn spread_ticks(&self) -> i64 {
        self.ask.0.saturating_sub(self.bid.0).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mid_and_spread_are_integer() {
        let t = Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(1),
            bid: Px(100),
            ask: Px(103),
            bid_qty: Qty(1),
            ask_qty: Qty(1),
            aggressor: None,
        };
        assert_eq!(t.mid_ticks(), Px(101));
        assert_eq!(t.spread_ticks(), 3);
    }
}
