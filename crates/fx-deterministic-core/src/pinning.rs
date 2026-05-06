//! CPU and NUMA pinning — operational contract for deterministic latency.
//!
//! Production deployments should:
//! 1. Pin the matching / ingress thread(s) to dedicated physical cores (no hyper-thread sibling load).
//! 2. Allocate huge pages / pre-touch working set where supported.
//! 3. Keep NIC IRQ and trading threads on the same NUMA node.
//!
//! ### Linux (example)
//! Use `taskset -c 2,3` or `sched_setaffinity` from a tiny launcher crate; isolate cores via `isolcpus`.
//!
//! ### Windows
//! Use `SetThreadAffinityMask` / process affinity in a dedicated bootstrap binary.
//!
//! This repository does not call OS APIs here to remain cross-platform at compile time; wire affinity in your deploy layer.

/// Documented placeholder for future `core_affinity`-style hook.
pub fn recommended_isolated_cores() -> &'static str {
    "Pin 1 core for matching, 1 for market-data ingest, 1 for egress; avoid sharing with GC runtimes."
}
