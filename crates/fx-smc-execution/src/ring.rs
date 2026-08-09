//! SPSC ring helpers over `rtrb` (lock-free, wait-free producer/consumer).

use rtrb::{Consumer, Producer, RingBuffer};

/// Split producer/consumer pair.
pub struct SpscPair<T: Copy> {
    /// Single producer.
    pub producer: Producer<T>,
    /// Single consumer.
    pub consumer: Consumer<T>,
}

/// Create a bounded SPSC ring of `capacity` slots (power-of-two recommended by `rtrb`).
#[must_use]
pub fn spsc_pair<T: Copy>(capacity: usize) -> SpscPair<T> {
    let (producer, consumer) = RingBuffer::new(capacity.max(2));
    SpscPair { producer, consumer }
}
