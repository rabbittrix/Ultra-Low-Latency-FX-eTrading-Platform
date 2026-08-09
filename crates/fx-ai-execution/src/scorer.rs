//! In-process venue scoring (ported from the Python NumPy fallback).
//!
//! This is the default hot-path scorer for ultra-low-latency execution: no HTTP hop,
//! no Python venv. Optional ONNX / remote inference can wrap the same types later.

use crate::types::{
    ExecutionRecommendation, InferRequest, InferResponse, VenueFeatures, VenueScore,
};

/// Logistic weights matching `ai/ai-execution-service` `_fallback_scores`.
const WEIGHTS: [f64; 7] = [-0.9, 0.25, -1.15, -0.45, -0.95, -0.25, -0.35];
const BIAS: f64 = 0.2;

fn feature_row(v: &VenueFeatures, quantity: f64) -> [f64; 7] {
    [
        v.spread_bps / 50.0,
        (1.0 + v.depth.max(0.0)).ln() / 20.0,
        v.recent_reject_rate,
        v.latency_ewma_us / 1000.0,
        v.toxicity_hint,
        v.mid_move_bps.abs() / 10.0,
        (1.0 + quantity.max(0.0)).ln() / 25.0,
    ]
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

fn score_venue(v: &VenueFeatures, quantity: f64) -> VenueScore {
    let x = feature_row(v, quantity);
    let z: f64 = WEIGHTS
        .iter()
        .zip(x.iter())
        .map(|(w, xi)| w * xi)
        .sum::<f64>()
        + BIAS;
    let fill = sigmoid(z).clamp(0.05, 0.99);
    let rej = (1.0 - fill).clamp(0.0, 1.0);
    let impact = (v.spread_bps * 0.35 + v.toxicity_hint * 8.0).max(0.0);
    let lat = (v.latency_ewma_us * (1.0 + rej)).max(20.0);
    let score = fill * 1000.0 - lat - impact * 50.0 - rej * 200.0;
    VenueScore {
        venue_id: v.venue_id.clone(),
        fill_probability: fill,
        expected_latency_us: lat,
        rejection_likelihood: rej,
        market_impact_bps: impact,
        score,
    }
}

/// Synchronous in-process inference (same contract as the former Python `/v1/infer`).
pub fn infer_local(req: &InferRequest) -> InferResponse {
    let mut venues: Vec<VenueScore> = req
        .venues
        .iter()
        .map(|v| score_venue(v, req.quantity))
        .collect();
    venues.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let ranked_venue_ids: Vec<String> = venues.iter().map(|v| v.venue_id.clone()).collect();
    InferResponse {
        venues,
        recommendation: ExecutionRecommendation {
            ranked_venue_ids,
            notes: "model=rust_local_logistic; ranked_by=score".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VenueFeatures;

    #[test]
    fn ranks_venues_and_clamps_fill() {
        let req = InferRequest {
            instrument: "EURUSD".into(),
            side: "buy".into(),
            quantity: 1_000_000.0,
            venues: vec![
                VenueFeatures {
                    venue_id: "LP_A".into(),
                    spread_bps: 0.5,
                    depth: 5_000_000.0,
                    recent_reject_rate: 0.05,
                    latency_ewma_us: 80.0,
                    toxicity_hint: 0.1,
                    mid_move_bps: 0.2,
                },
                VenueFeatures {
                    venue_id: "ECN_SIM".into(),
                    spread_bps: 5.0,
                    depth: 100_000.0,
                    recent_reject_rate: 0.4,
                    latency_ewma_us: 400.0,
                    toxicity_hint: 0.8,
                    mid_move_bps: 2.0,
                },
            ],
        };
        let resp = infer_local(&req);
        assert_eq!(resp.venues.len(), 2);
        assert_eq!(resp.recommendation.ranked_venue_ids[0], "LP_A");
        for v in &resp.venues {
            assert!(v.fill_probability >= 0.05 && v.fill_probability <= 0.99);
        }
        assert!(resp.recommendation.notes.contains("rust_local"));
    }
}
