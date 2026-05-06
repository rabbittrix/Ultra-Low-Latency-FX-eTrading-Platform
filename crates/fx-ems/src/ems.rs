use crate::strategy::{build_slice_plan, ExecutionAlgo, SlicePlan};
use fx_oms::OmsOrder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionDestination {
    Internal,
    LiquidityProvider,
    Exchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDecision {
    pub order_id: fx_utils::OrderId,
    pub destination: ExecutionDestination,
    pub slice_plan: SlicePlan,
}

#[derive(Default)]
pub struct EmsEngine;

impl EmsEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn decide(&self, order: &OmsOrder, algo: ExecutionAlgo) -> ExecutionDecision {
        let destination = if order.quantity.0 <= 5_000_000 {
            ExecutionDestination::Internal
        } else {
            ExecutionDestination::LiquidityProvider
        };

        let parts = if matches!(algo, ExecutionAlgo::Direct) { 1 } else { 4 };
        let slice_plan = build_slice_plan(order, algo, parts);

        ExecutionDecision {
            order_id: order.order_id,
            destination,
            slice_plan,
        }
    }
}
