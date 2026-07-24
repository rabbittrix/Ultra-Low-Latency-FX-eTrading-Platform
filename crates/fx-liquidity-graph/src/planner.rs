//! Path optimization: Dijkstra on directed graph with custom edge weights.

use crate::graph::LiquidityGraph;
use crate::types::{ExecutionPlan, SliceStrategy, VenueAllocation};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f64,
    node: usize,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Tunable weights for edge cost (lower is better).
#[derive(Debug, Clone)]
pub struct GraphPlanner {
    pub w_latency: f64,
    pub w_toxicity: f64,
    pub w_inv_fill: f64,
}

impl Default for GraphPlanner {
    fn default() -> Self {
        Self {
            w_latency: 1.0,
            w_toxicity: 400.0,
            w_inv_fill: 250.0,
        }
    }
}

impl GraphPlanner {
    /// Combined edge weight for routing.
    pub fn edge_weight(&self, e: &crate::types::LiquidityEdge) -> f64 {
        let inv_fill = 1.0 / e.fill_probability.max(1e-6);
        self.w_latency * e.latency_us + self.w_toxicity * e.toxicity + self.w_inv_fill * inv_fill
    }
}

fn index_nodes(graph: &LiquidityGraph) -> (HashMap<String, usize>, Vec<String>) {
    let mut ids: Vec<String> = graph.nodes().map(|n| n.id.clone()).collect();
    ids.sort();
    let map: HashMap<String, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    (map, ids)
}

/// Dijkstra from `source` to `goal` over edges in `graph`.
pub fn dijkstra_path(
    graph: &LiquidityGraph,
    planner: &GraphPlanner,
    source: &str,
    goal: &str,
) -> Option<(f64, Vec<String>)> {
    let (idx, id_list) = index_nodes(graph);
    let n = id_list.len();
    let s = *idx.get(source)?;
    let t = *idx.get(goal)?;

    let mut dist = vec![f64::INFINITY; n];
    let mut parent: Vec<Option<usize>> = vec![None; n];
    dist[s] = 0.0;
    let mut heap = BinaryHeap::new();
    heap.push(State { cost: 0.0, node: s });

    while let Some(State { cost, node }) = heap.pop() {
        if node == t {
            break;
        }
        if cost > dist[node] {
            continue;
        }
        let from_id = &id_list[node];
        for e in graph.edges_from(from_id) {
            let Some(&v) = idx.get(&e.to) else { continue };
            let w = planner.edge_weight(e);
            let next = cost + w;
            if next < dist[v] {
                dist[v] = next;
                parent[v] = Some(node);
                heap.push(State {
                    cost: next,
                    node: v,
                });
            }
        }
    }

    if dist[t].is_infinite() {
        return None;
    }

    let mut path = Vec::new();
    let mut cur = Some(t);
    while let Some(u) = cur {
        path.push(id_list[u].clone());
        cur = parent[u];
    }
    path.reverse();
    Some((dist[t], path))
}

/// Build an execution plan: primary path from CLIENT → best terminal venue, split by depth.
pub fn plan_execution(
    graph: &LiquidityGraph,
    planner: &GraphPlanner,
    instrument: &str,
    side: &str,
    quantity: f64,
    terminals: &[&str],
) -> Option<ExecutionPlan> {
    let mut best: Option<(f64, Vec<String>, String, f64)> = None;
    for term in terminals {
        if let Some((cost, path)) = dijkstra_path(graph, planner, "CLIENT", term) {
            if best.as_ref().map_or(true, |(c, _, _, _)| cost < *c) {
                // price from first hop CLIENT->*
                let first = graph.edges_from("CLIENT").iter().find(|e| e.to == *term);
                let px = first.map(|e| e.price).unwrap_or(1.0);
                best = Some((cost, path, (*term).to_string(), px));
            }
        }
    }

    let (path_cost, primary_path, terminal, px) = best?;

    // Multi-venue split: allocate 60% best path terminal, 40% next best LP by edge weight
    let mut allocations = Vec::new();
    let primary_qty = quantity * 0.6;
    allocations.push(VenueAllocation {
        venue_id: terminal.clone(),
        quantity: primary_qty,
        expected_price: px,
        hop: 1,
    });

    // Secondary: pick lowest-weight CLIENT edge not equal terminal
    let mut candidates: Vec<_> = graph
        .edges_from("CLIENT")
        .iter()
        .filter(|e| e.to != terminal)
        .collect();
    candidates.sort_by(|a, b| {
        planner
            .edge_weight(a)
            .partial_cmp(&planner.edge_weight(b))
            .unwrap_or(Ordering::Equal)
    });
    if let Some(e) = candidates.first() {
        let q = quantity - primary_qty;
        allocations.push(VenueAllocation {
            venue_id: e.to.clone(),
            quantity: q,
            expected_price: e.price,
            hop: 1,
        });
    }

    let slice_strategy = if quantity > 10_000_000.0 {
        SliceStrategy::TimeWeighted {
            slices: 4,
            interval_ms: 50,
        }
    } else {
        SliceStrategy::Immediate
    };

    // Slippage vs. best CLIENT price
    let best_px = graph
        .edges_from("CLIENT")
        .iter()
        .map(|e| e.price)
        .fold(f64::INFINITY, f64::min);
    let avg_px = allocations
        .iter()
        .map(|a| a.expected_price * a.quantity)
        .sum::<f64>()
        / quantity.max(1.0);
    let expected_slippage_bps = if side.eq_ignore_ascii_case("buy") {
        (avg_px - best_px) / best_px * 10_000.0
    } else {
        (best_px - avg_px) / best_px * 10_000.0
    };

    Some(ExecutionPlan {
        instrument: instrument.to_string(),
        side: side.to_string(),
        total_quantity: quantity,
        allocations,
        slice_strategy,
        expected_slippage_bps,
        primary_path,
        path_cost,
    })
}
