//! Bounded event ring wrapping `crossbeam_queue::ArrayQueue` (preallocated, lock-free).

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

/// Fixed-capacity order event slot (stack-sized payload — no heap per push).
#[derive(Debug, Clone, Copy, Default)]
pub struct OrderEventSlot {
    pub order_id: u64,
    pub qty: u64,
    pub price_ticks: u64,
    pub flags: u32,
}

/// Shareable ring buffer for hot-path handoff between threads.
#[derive(Clone)]
pub struct OrderEventRing {
    inner: Arc<ArrayQueue<OrderEventSlot>>,
}

impl OrderEventRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    #[inline]
    pub fn try_push(&self, ev: OrderEventSlot) -> Result<(), OrderEventSlot> {
        self.inner.push(ev)
    }

    #[inline]
    pub fn try_pop(&self) -> Option<OrderEventSlot> {
        self.inner.pop()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
