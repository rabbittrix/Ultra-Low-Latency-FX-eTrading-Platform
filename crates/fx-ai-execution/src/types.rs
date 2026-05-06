use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueFeatures {
    pub venue_id: String,
    pub spread_bps: f64,
    pub depth: f64,
    pub recent_reject_rate: f64,
    pub latency_ewma_us: f64,
    pub toxicity_hint: f64,
    pub mid_move_bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferRequest {
    pub instrument: String,
    pub side: String,
    pub quantity: f64,
    pub venues: Vec<VenueFeatures>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueScore {
    pub venue_id: String,
    pub fill_probability: f64,
    pub expected_latency_us: f64,
    pub rejection_likelihood: f64,
    pub market_impact_bps: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecommendation {
    pub ranked_venue_ids: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferResponse {
    pub venues: Vec<VenueScore>,
    pub recommendation: ExecutionRecommendation,
}
