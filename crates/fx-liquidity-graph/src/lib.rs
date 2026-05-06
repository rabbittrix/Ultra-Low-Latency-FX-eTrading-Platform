//! Global liquidity graph engine: aggregate venues, build a directed graph, plan execution.
//!
//! Service-layer graph updates may use heap; path search uses pre-sized buffers for typical venue counts.

pub mod graph;
pub mod planner;
pub mod types;

pub use graph::LiquidityGraph;
pub use planner::{plan_execution, GraphPlanner};
pub use types::{
    ExecutionPlan, LiquidityEdge, LiquidityNode, SliceStrategy, VenueAllocation, VenueClass,
};
