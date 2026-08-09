//! Deterministic event hashing (BLAKE3) for live == replay proofs.

use crate::types::Tick;
use blake3::Hasher;

/// 32-byte event stream digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHash(pub [u8; 32]);

impl EventHash {
    /// Lowercase hex encoding.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut out = String::with_capacity(64);
        for b in self.0 {
            use std::fmt::Write as _;
            let _ = write!(out, "{b:02x}");
        }
        out
    }
}

/// Incremental hasher over SMC events.
#[derive(Debug, Default, Clone)]
pub struct EventHasher {
    inner: Hasher,
}

impl EventHasher {
    /// New empty hasher.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Hasher::new(),
        }
    }

    /// Absorb one tick in a canonical byte layout (little-endian integers, UTF-8 symbol).
    pub fn absorb_tick(&mut self, tick: &Tick) {
        self.inner.update(tick.symbol.as_str().as_bytes());
        self.inner.update(&[0xff]);
        self.inner.update(&tick.ts_ns.0.to_le_bytes());
        self.inner.update(&tick.bid.0.to_le_bytes());
        self.inner.update(&tick.ask.0.to_le_bytes());
        self.inner.update(&tick.bid_qty.0.to_le_bytes());
        self.inner.update(&tick.ask_qty.0.to_le_bytes());
        let agg = match tick.aggressor {
            None => 0_u8,
            Some(crate::types::Side::Buy) => 1,
            Some(crate::types::Side::Sell) => 2,
        };
        self.inner.update(&[agg]);
    }

    /// Finalize digest.
    #[must_use]
    pub fn finalize(self) -> EventHash {
        EventHash(*self.inner.finalize().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Px, Qty, SymbolId, Tick, TsNanos};

    fn sample(ts: i64) -> Tick {
        Tick {
            symbol: SymbolId::new("EURUSD"),
            ts_ns: TsNanos(ts),
            bid: Px(11_000),
            ask: Px(11_001),
            bid_qty: Qty(100),
            ask_qty: Qty(100),
            aggressor: None,
        }
    }

    #[test]
    fn same_ticks_same_hash() {
        let mut a = EventHasher::new();
        let mut b = EventHasher::new();
        a.absorb_tick(&sample(1));
        a.absorb_tick(&sample(2));
        b.absorb_tick(&sample(1));
        b.absorb_tick(&sample(2));
        assert_eq!(a.finalize(), b.finalize());
    }

    #[test]
    fn order_changes_hash() {
        let mut a = EventHasher::new();
        let mut b = EventHasher::new();
        a.absorb_tick(&sample(1));
        a.absorb_tick(&sample(2));
        b.absorb_tick(&sample(2));
        b.absorb_tick(&sample(1));
        assert_ne!(a.finalize(), b.finalize());
    }
}
