//! AI/ML service client for volatility prediction

use fx_utils::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request for volatility prediction
#[derive(Debug, Serialize)]
struct VolatilityRequest {
    instrument: String,
    historical_prices: Option<Vec<f64>>,
    lookback_period: i32,
}

/// Response from volatility prediction
#[derive(Debug, Deserialize)]
struct VolatilityResponse {
    #[allow(dead_code)]
    instrument: String,
    predicted_volatility: f64,
    #[allow(dead_code)]
    confidence: f64,
}

/// AI/ML service client
pub struct AiClient {
    base_url: String,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(100)) // 100ms timeout for low latency
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }

    /// Predict volatility for an instrument
    pub async fn predict_volatility(&self, instrument: &str) -> Result<f64> {
        let url = format!("{}/predict/volatility", self.base_url);
        let request = VolatilityRequest {
            instrument: instrument.to_string(),
            historical_prices: None,
            lookback_period: 20,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| fx_utils::Error::Internal(format!("AI service error: {}", e)))?;

        if !response.status().is_success() {
            return Err(fx_utils::Error::Internal(format!(
                "AI service returned error: {}",
                response.status()
            )));
        }

        let volatility_response: VolatilityResponse = response
            .json()
            .await
            .map_err(|e| fx_utils::Error::Internal(format!("Failed to parse response: {}", e)))?;

        Ok(volatility_response.predicted_volatility)
    }
}
