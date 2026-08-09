//! AI-driven predictive execution: venue features in, scores and ranking out.
//!
//! Default path is **in-process** Rust logistic scoring (`scorer`). Optional HTTP
//! client remains for a remote Python/ONNX service when `AI_EXECUTION_MODE=http`.

pub mod client;
pub mod scorer;
pub mod types;

pub use client::{AiClientError, AiExecutionClient, AiExecutionMode};
pub use scorer::infer_local;
pub use types::{ExecutionRecommendation, InferRequest, InferResponse, VenueFeatures, VenueScore};
