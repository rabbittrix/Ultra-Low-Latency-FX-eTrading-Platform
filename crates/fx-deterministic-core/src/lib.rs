//! Deterministic ultra-low-latency building blocks.
//!
//! ## Hot path guidelines
//! - Prefer pre-allocated `ArrayQueue` / ring buffers; avoid `Vec` growth on the matching thread.
//! - No mutexes on the trading thread; use SPSC/MPSC queues with bounded capacity.
//! - Pin critical threads to cores and isolate NUMA nodes in production (see [`pinning`]).
//! - Set `TCP_NODELAY` on latency-sensitive sockets ([`tcp_tune`]).
//!
//! Full “zero heap” matching is a larger refactor of `fx-core`; this crate supplies shared primitives
//! for new services and future matching-engine hardening.

pub mod pinning;
pub mod ring;
pub mod tcp_tune;

pub use ring::{OrderEventRing, OrderEventSlot};
pub use tcp_tune::set_tcp_nodelay;
