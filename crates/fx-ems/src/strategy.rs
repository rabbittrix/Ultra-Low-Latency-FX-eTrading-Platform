use fx_oms::OmsOrder;
use fx_utils::Quantity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionAlgo {
    Direct,
    Twap,
    Vwap,
    Pov,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicePlan {
    pub parent_order_id: fx_utils::OrderId,
    pub slices: Vec<Quantity>,
}

pub fn build_slice_plan(order: &OmsOrder, algo: ExecutionAlgo, parts: u64) -> SlicePlan {
    if matches!(algo, ExecutionAlgo::Direct) || parts <= 1 {
        return SlicePlan {
            parent_order_id: order.order_id,
            slices: vec![order.quantity],
        };
    }

    let per_slice = order.quantity.0 / parts.max(1);
    let mut slices = Vec::with_capacity(parts as usize);
    let mut remaining = order.quantity.0;

    for _ in 0..parts {
        let q = per_slice.min(remaining);
        slices.push(Quantity(q));
        remaining = remaining.saturating_sub(q);
    }

    if remaining > 0 {
        slices.push(Quantity(remaining));
    }

    SlicePlan {
        parent_order_id: order.order_id,
        slices,
    }
}
