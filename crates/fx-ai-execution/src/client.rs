//! Non-blocking HTTP client for inference.

use crate::types::{InferRequest, InferResponse};
use reqwest::Url;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AiExecutionClient {
    base: Url,
    http: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum AiClientError {
    #[error("invalid base url: {0}")]
    BadUrl(String),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("inference failed: {0}")]
    BadStatus(String),
}

impl AiExecutionClient {
    pub fn new(base_url: &str) -> Result<Self, AiClientError> {
        let base: Url = base_url
            .parse()
            .map_err(|_| AiClientError::BadUrl(base_url.to_string()))?;
        let http = reqwest::Client::builder()
            .tcp_nodelay(true)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .map_err(AiClientError::Http)?;
        Ok(Self { base, http })
    }

    pub async fn infer(&self, req: &InferRequest) -> Result<InferResponse, AiClientError> {
        let url = self
            .base
            .join("/v1/infer")
            .map_err(|e| AiClientError::BadUrl(e.to_string()))?;
        let res = self.http.post(url).json(req).send().await?;
        if !res.status().is_success() {
            return Err(AiClientError::BadStatus(format!(
                "{} {}",
                res.status(),
                res.text().await.unwrap_or_default()
            )));
        }
        Ok(res.json().await?)
    }
}
