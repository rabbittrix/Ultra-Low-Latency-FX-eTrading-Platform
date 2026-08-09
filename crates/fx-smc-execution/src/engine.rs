//! Minimal hot-path engine: ingest ticks, emit research intents when spread is sane.

use crate::ring::SpscPair;
use crate::slot::{ExecIntent, ExecSide, TickSlot};
use blake3::Hasher;
use thiserror::Error;

/// Hot-path failures (ring full / kill).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HotPathError {
    /// Intent ring is full — caller must shed or enlarge capacity offline.
    #[error("intent ring full")]
    IntentRingFull,
    /// Engine killed (feed quality / external trip).
    #[error("engine killed")]
    Killed,
}

/// Stateless-ish engine state kept on the trading thread (no heap growth).
#[derive(Debug)]
pub struct HotPathEngine {
    next_intent_id: u64,
    max_spread_ticks: i64,
    killed: bool,
    intent_hash: Hasher,
    intent_count: u64,
}

impl HotPathEngine {
    /// Create with a maximum acceptable TOB spread (ticks).
    #[must_use]
    pub fn new(max_spread_ticks: i64) -> Self {
        Self {
            next_intent_id: 1,
            max_spread_ticks: max_spread_ticks.max(0),
            killed: false,
            intent_hash: Hasher::new(),
            intent_count: 0,
        }
    }

    /// Trip the engine (feed kill). Further ticks are ignored.
    pub fn kill(&mut self) {
        self.killed = true;
    }

    /// Whether the engine is tripped.
    #[must_use]
    pub fn is_killed(&self) -> bool {
        self.killed
    }

    /// Number of intents emitted since construction / reset.
    #[must_use]
    pub fn intent_count(&self) -> u64 {
        self.intent_count
    }

    /// BLAKE3 of all emitted intent fingerprints (determinism proof).
    #[must_use]
    pub fn intents_fingerprint(&self) -> [u8; 32] {
        *self.intent_hash.finalize().as_bytes()
    }

    /// Drain `ticks` consumer and push research intents into `intents` producer.
    ///
    /// Emits at most one intent per tick when spread ≤ `max_spread_ticks` and mid moves.
    /// Zero heap allocation on the steady path.
    ///
    /// # Errors
    /// [`HotPathError::Killed`] or [`HotPathError::IntentRingFull`].
    pub fn drain_ticks(
        &mut self,
        ticks: &mut SpscPair<TickSlot>,
        intents: &mut SpscPair<ExecIntent>,
    ) -> Result<u32, HotPathError> {
        if self.killed {
            return Err(HotPathError::Killed);
        }
        let mut produced = 0_u32;
        let mut last_mid: Option<i64> = None;
        while let Ok(slot) = ticks.consumer.pop() {
            if slot.spread() > self.max_spread_ticks {
                continue;
            }
            let mid = slot.mid();
            let moved = last_mid != Some(mid);
            last_mid = Some(mid);
            if !moved {
                continue;
            }
            let side = if slot.bid_qty >= slot.ask_qty {
                ExecSide::Buy
            } else {
                ExecSide::Sell
            };
            let intent = ExecIntent {
                id: self.next_intent_id,
                symbol: slot.symbol,
                side,
                px: mid,
                qty: 1,
                ts_ns: slot.ts_ns,
                src_seq: slot.seq,
                flags: 0,
            };
            self.next_intent_id = self.next_intent_id.saturating_add(1);
            if intents.producer.push(intent).is_err() {
                return Err(HotPathError::IntentRingFull);
            }
            self.intent_hash.update(&intent.fingerprint_bytes());
            self.intent_count = self.intent_count.saturating_add(1);
            produced = produced.saturating_add(1);
        }
        Ok(produced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring::spsc_pair;

    #[test]
    fn dual_run_same_fingerprint() {
        let slots: Vec<TickSlot> = (0_u64..64)
            .map(|i| TickSlot {
                symbol: 1,
                bid: 100 + i64::try_from(i).unwrap_or(0),
                ask: 102 + i64::try_from(i).unwrap_or(0),
                bid_qty: 10,
                ask_qty: 8,
                ts_ns: i64::try_from(i).unwrap_or(0).saturating_mul(1_000),
                seq: i,
            })
            .collect();

        let run = |slots: &[TickSlot]| {
            let mut ticks = spsc_pair::<TickSlot>(128);
            let mut intents = spsc_pair::<ExecIntent>(128);
            let mut eng = HotPathEngine::new(10);
            for s in slots {
                ticks.producer.push(*s).expect("push tick");
            }
            eng.drain_ticks(&mut ticks, &mut intents).expect("drain");
            eng.intents_fingerprint()
        };

        assert_eq!(run(&slots), run(&slots));
    }
}
