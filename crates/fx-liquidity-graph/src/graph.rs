//! Mutable liquidity graph: adjacency list, edge CRUD, mock venue wiring.

use crate::types::{LiquidityEdge, LiquidityNode, VenueClass};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Real-time directed graph of liquidity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiquidityGraph {
    pub instrument: String,
    nodes: HashMap<String, LiquidityNode>,
    /// Adjacency: from_venue -> list of edges
    adj: HashMap<String, Vec<LiquidityEdge>>,
}

impl LiquidityGraph {
    pub fn new(instrument: impl Into<String>) -> Self {
        Self {
            instrument: instrument.into(),
            nodes: HashMap::new(),
            adj: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: LiquidityNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.adj.entry(id).or_default();
    }

    pub fn upsert_node(&mut self, node: LiquidityNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.adj.entry(id).or_default();
    }

    pub fn add_edge(&mut self, edge: LiquidityEdge) {
        self.adj.entry(edge.from.clone()).or_default().push(edge);
    }

    pub fn nodes(&self) -> impl Iterator<Item = &LiquidityNode> {
        self.nodes.values()
    }

    pub fn edges_from(&self, from: &str) -> &[LiquidityEdge] {
        self.adj.get(from).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn all_edges(&self) -> Vec<&LiquidityEdge> {
        self.adj.values().flat_map(|v| v.iter()).collect()
    }

    /// Seed a demo graph: internal book + LP_A/B + ECN_SIM with cross-connections.
    pub fn mock_global_liquidity(instrument: &str) -> Self {
        let mut g = Self::new(instrument);
        g.upsert_node(LiquidityNode {
            id: "INTERNAL".into(),
            class: VenueClass::InternalBook,
            label: "Internal Matcher".into(),
        });
        g.upsert_node(LiquidityNode {
            id: "LP_A".into(),
            class: VenueClass::LiquidityProvider,
            label: "LP Bank A".into(),
        });
        g.upsert_node(LiquidityNode {
            id: "LP_B".into(),
            class: VenueClass::LiquidityProvider,
            label: "LP Bank B".into(),
        });
        g.upsert_node(LiquidityNode {
            id: "ECN_SIM".into(),
            class: VenueClass::Ecn,
            label: "Simulated ECN".into(),
        });
        g.upsert_node(LiquidityNode {
            id: "CLIENT".into(),
            class: VenueClass::InternalBook,
            label: "Client Ingress".into(),
        });

        // CLIENT can reach all external sources (synthetic ingress edges).
        g.add_edge(LiquidityEdge {
            from: "CLIENT".into(),
            to: "INTERNAL".into(),
            price: 1.10010,
            available_size: 5_000_000.0,
            latency_us: 80.0,
            fill_probability: 0.92,
            toxicity: 0.08,
        });
        g.add_edge(LiquidityEdge {
            from: "CLIENT".into(),
            to: "LP_A".into(),
            price: 1.10005,
            available_size: 10_000_000.0,
            latency_us: 450.0,
            fill_probability: 0.88,
            toxicity: 0.12,
        });
        g.add_edge(LiquidityEdge {
            from: "CLIENT".into(),
            to: "LP_B".into(),
            price: 1.10000,
            available_size: 8_000_000.0,
            latency_us: 520.0,
            fill_probability: 0.85,
            toxicity: 0.14,
        });
        g.add_edge(LiquidityEdge {
            from: "CLIENT".into(),
            to: "ECN_SIM".into(),
            price: 1.09995,
            available_size: 20_000_000.0,
            latency_us: 900.0,
            fill_probability: 0.78,
            toxicity: 0.18,
        });

        // Multi-hop: INTERNAL ↔ ECN bridge (internalizer hedging path).
        g.add_edge(LiquidityEdge {
            from: "INTERNAL".into(),
            to: "ECN_SIM".into(),
            price: 1.09990,
            available_size: 15_000_000.0,
            latency_us: 300.0,
            fill_probability: 0.80,
            toxicity: 0.10,
        });

        g
    }

    /// Apply AI/refiner scores to edges targeting a venue (matches `edge.to`).
    pub fn apply_venue_fill_probs(&mut self, venue_id: &str, fill_prob: f64) {
        let fp = fill_prob.clamp(0.01, 1.0);
        for edges in self.adj.values_mut() {
            for e in edges.iter_mut() {
                if e.to == venue_id {
                    e.fill_probability = fp;
                }
            }
        }
    }
}
