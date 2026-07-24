//! AI-driven predictive execution: venue features in, scores and ranking out.
//!
//! The default integration is HTTP JSON to the Python `ai-execution-service` (ONNX-backed when available).

pub mod client;
pub mod types;

pub use client::AiExecutionClient;
pub use types::{ExecutionRecommendation, InferRequest, InferResponse, VenueFeatures, VenueScore};
