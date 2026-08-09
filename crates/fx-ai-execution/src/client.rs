//! Venue scoring backend: in-process (default) or optional HTTP to a remote service.

use crate::scorer::infer_local;
use crate::types::{InferRequest, InferResponse};
use reqwest::Url;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiExecutionMode {
    /// In-process logistic scorer (no Python / no network).
    Local,
    /// HTTP JSON to a remote AI service (`/v1/infer`).
    Http,
}

#[derive(Debug, Clone)]
pub struct AiExecutionClient {
    mode: AiExecutionMode,
    base: Option<Url>,
    http: Option<reqwest::Client>,
}

#[derive(Debug, Error)]
pub enum AiClientError {
    #[error("invalid base url: {0}")]
    BadUrl(String),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("inference failed: {0}")]
    BadStatus(String),
    #[error("HTTP mode selected but no base URL configured")]
    MissingHttpBase,
}

impl AiExecutionClient {
    /// Default: local in-process scoring.
    pub fn local() -> Self {
        Self {
            mode: AiExecutionMode::Local,
            base: None,
            http: None,
        }
    }

    /// Remote Python/ONNX (or any compatible) `/v1/infer` service.
    pub fn http(base_url: &str) -> Result<Self, AiClientError> {
        let base: Url = base_url
            .parse()
            .map_err(|_| AiClientError::BadUrl(base_url.to_string()))?;
        let http = reqwest::Client::builder()
            .tcp_nodelay(true)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_millis(500))
            .build()
            .map_err(AiClientError::Http)?;
        Ok(Self {
            mode: AiExecutionMode::Http,
            base: Some(base),
            http: Some(http),
        })
    }

    /// Backward-compatible constructor: builds an HTTP client for `base_url`.
    /// Prefer [`Self::local`] or [`Self::from_env`].
    pub fn new(base_url: &str) -> Result<Self, AiClientError> {
        Self::http(base_url)
    }

    /// `AI_EXECUTION_MODE=local|http` (default `local`).
    /// When `http`, uses `AI_EXECUTION_URL` (default `http://127.0.0.1:8093`).
    pub fn from_env() -> Result<Self, AiClientError> {
        let mode = std::env::var("AI_EXECUTION_MODE")
            .unwrap_or_else(|_| "local".into())
            .to_ascii_lowercase();
        if mode == "http" || mode == "remote" {
            let url = std::env::var("AI_EXECUTION_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8093".into());
            Self::http(&url)
        } else {
            Ok(Self::local())
        }
    }

    pub fn mode(&self) -> AiExecutionMode {
        self.mode
    }

    pub async fn infer(&self, req: &InferRequest) -> Result<InferResponse, AiClientError> {
        match self.mode {
            AiExecutionMode::Local => Ok(infer_local(req)),
            AiExecutionMode::Http => self.infer_http(req).await,
        }
    }

    async fn infer_http(&self, req: &InferRequest) -> Result<InferResponse, AiClientError> {
        let base = self.base.as_ref().ok_or(AiClientError::MissingHttpBase)?;
        let http = self.http.as_ref().ok_or(AiClientError::MissingHttpBase)?;
        let url = base
            .join("/v1/infer")
            .map_err(|e| AiClientError::BadUrl(e.to_string()))?;
        let res = http.post(url).json(req).send().await?;
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
