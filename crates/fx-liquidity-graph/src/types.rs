//! Core types for the liquidity graph and execution plan output.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Venue category for graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(rename_all = "snake_case")]
pub enum VenueClass {
    InternalBook,
    LiquidityProvider,
    Ecn,
    Exchange,
}

/// A venue or logical liquidity source (node).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiquidityNode {
    pub id: String,
    pub class: VenueClass,
    /// Optional display name.
    pub label: String,
}

/// Directed edge: executable path from `from` → `to` with economics and microstructure.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiquidityEdge {
    pub from: String,
    pub to: String,
    /// Executable price (e.g. all-in rate for the instrument).
    pub price: f64,
    pub available_size: f64,
    /// Expected one-way latency in microseconds (network + matching).
    pub latency_us: f64,
    /// Model-estimated fill probability in (0, 1].
    pub fill_probability: f64,
    /// Toxicity / adverse selection score in [0, 1].
    pub toxicity: f64,
}

/// One leg of a routed order.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VenueAllocation {
    pub venue_id: String,
    pub quantity: f64,
    pub expected_price: f64,
    pub hop: u32,
}

/// How child orders are sliced in time.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(rename_all = "snake_case")]
pub enum SliceStrategy {
    /// Single wave.
    Immediate,
    /// Equal slices over `interval_ms` (hint for EMS).
    TimeWeighted { slices: u32, interval_ms: u64 },
}

/// Best-effort execution plan: venue split + slippage estimate.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecutionPlan {
    pub instrument: String,
    pub side: String,
    pub total_quantity: f64,
    pub allocations: Vec<VenueAllocation>,
    pub slice_strategy: SliceStrategy,
    /// Expected slippage vs. mid, in basis points (signed).
    pub expected_slippage_bps: f64,
    /// Graph path node ids for visualization (primary path).
    pub primary_path: Vec<String>,
    /// Effective edge costs used on the primary path.
    pub path_cost: f64,
}
