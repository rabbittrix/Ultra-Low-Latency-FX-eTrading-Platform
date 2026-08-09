//! Fixed-size `Copy` payloads for SPSC rings (no heap on hot path).

use fx_smc_common::{Px, Qty, Side, SymbolId, Tick, TsNanos};

/// Buy / sell as a `u8`-sized enum for ring slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ExecSide {
    /// Bid / buy.
    Buy = 0,
    /// Ask / sell.
    Sell = 1,
}

impl From<Side> for ExecSide {
    fn from(s: Side) -> Self {
        match s {
            Side::Buy => Self::Buy,
            Side::Sell => Self::Sell,
        }
    }
}

/// Compact TOB tick for ring transport (all `Copy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickSlot {
    /// Symbol numeric id (caller-defined mapping).
    pub symbol: u32,
    /// Bid ticks.
    pub bid: i64,
    /// Ask ticks.
    pub ask: i64,
    /// Bid size.
    pub bid_qty: i64,
    /// Ask size.
    pub ask_qty: i64,
    /// Event time (ns UTC).
    pub ts_ns: i64,
    /// Sequence / logical clock.
    pub seq: u64,
}

impl TickSlot {
    /// Build from a domain [`Tick`] with an explicit symbol mapping and sequence.
    #[must_use]
    pub fn from_tick(tick: &Tick, symbol: u32, seq: u64) -> Self {
        Self {
            symbol,
            bid: tick.bid.0,
            ask: tick.ask.0,
            bid_qty: tick.bid_qty.0,
            ask_qty: tick.ask_qty.0,
            ts_ns: tick.ts_ns.0,
            seq,
        }
    }

    /// Mid price in ticks (`(bid+ask)/2`).
    #[must_use]
    pub fn mid(&self) -> i64 {
        self.bid.saturating_add(self.ask) / 2
    }

    /// Spread in ticks.
    #[must_use]
    pub fn spread(&self) -> i64 {
        self.ask.saturating_sub(self.bid)
    }

    /// Convert back to a domain tick (cold path / tests).
    #[must_use]
    pub fn as_tick(self, symbol: SymbolId) -> Tick {
        Tick {
            symbol,
            bid: Px(self.bid),
            ask: Px(self.ask),
            bid_qty: Qty(self.bid_qty),
            ask_qty: Qty(self.ask_qty),
            ts_ns: TsNanos(self.ts_ns),
            aggressor: None,
        }
    }
}

/// Pre-sized execution intent (research / paper path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecIntent {
    /// Monotonic intent id.
    pub id: u64,
    /// Symbol numeric id.
    pub symbol: u32,
    /// Side.
    pub side: ExecSide,
    /// Limit / market reference price (ticks).
    pub px: i64,
    /// Quantity (lots as i64).
    pub qty: i64,
    /// Intent time (ns UTC).
    pub ts_ns: i64,
    /// Source tick sequence.
    pub src_seq: u64,
    /// Opaque flags (kill / paper / etc.).
    pub flags: u32,
}

impl ExecIntent {
    /// Stable 48-byte fingerprint for hashing (no heap).
    #[must_use]
    pub fn fingerprint_bytes(self) -> [u8; 48] {
        let mut buf = [0_u8; 48];
        buf[0..8].copy_from_slice(&self.id.to_le_bytes());
        buf[8..12].copy_from_slice(&self.symbol.to_le_bytes());
        buf[12] = self.side as u8;
        buf[16..24].copy_from_slice(&self.px.to_le_bytes());
        buf[24..32].copy_from_slice(&self.qty.to_le_bytes());
        buf[32..40].copy_from_slice(&self.ts_ns.to_le_bytes());
        buf[40..48].copy_from_slice(&self.src_seq.to_le_bytes());
        buf
    }
}
