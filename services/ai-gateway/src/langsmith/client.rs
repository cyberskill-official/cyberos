//! LangSmith HTTP client for FR-OBS-004.

use std::time::Duration;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Url;

use crate::policy::Residency;

use super::payload::LangSmithPayload;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);
pub const RETRY_DELAYS_MS: &[u64] = &[100, 250, 500];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangSmithConfig {
    pub base_url: String,
    pub token: String,
    pub request_timeout: Duration,
}

impl LangSmithConfig {
    pub fn from_env(residency: Residency) -> Self {
        let region_key = format!(
            "LANGSMITH_URL_{}",
            residency_slug(residency)
                .replace('-', "_")
                .to_ascii_uppercase()
        );
        let base_url = std::env::var(&region_key)
            .or_else(|_| std::env::var("LANGSMITH_URL"))
            .unwrap_or_else(|_| default_base_url(residency).to_string());
        let token = std::env::var("LANGSMITH_API_TOKEN").unwrap_or_default();
        Self::new(base_url, token, DEFAULT_TIMEOUT)
    }

    pub fn new(
        base_url: impl Into<String>,
        token: impl Into<String>,
        request_timeout: Duration,
    ) -> Self {
        Self {
            base_url: trim_trailing_slash(base_url.into()),
            token: token.into(),
            request_timeout,
        }
    }

    pub fn endpoint_url(&self) -> Result<Url, LangSmithError> {
        let mut url = Url::parse(&self.base_url).map_err(|err| LangSmithError::Config {
            reason: format!("invalid_langsmith_url: {err}"),
        })?;
        url.path_segments_mut()
            .map_err(|_| LangSmithError::Config {
                reason: "langsmith_url_cannot_be_base".to_string(),
            })?
            .extend(["api", "v1", "traces"]);
        Ok(url)
    }

    pub fn validate_self_hosted(&self) -> Result<(), LangSmithError> {
        let url = Url::parse(&self.base_url).map_err(|err| LangSmithError::Config {
            reason: format!("invalid_langsmith_url: {err}"),
        })?;
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        if host == "langchain.com" || host.ends_with(".langchain.com") {
            return Err(LangSmithError::Config {
                reason: "langchain_saas_endpoint_forbidden".to_string(),
            });
        }
        Ok(())
    }
}

pub async fn post_with_retry(payload: &LangSmithPayload) -> Result<(), LangSmithError> {
    let config = LangSmithConfig::from_env(Residency::Sg1);
    post_with_retry_with_config(&config, payload).await
}

pub async fn post_with_retry_with_config(
    config: &LangSmithConfig,
    payload: &LangSmithPayload,
) -> Result<(), LangSmithError> {
    config.validate_self_hosted()?;
    if config.token.trim().is_empty() {
        return Err(LangSmithError::AuthFailed);
    }

    let client = reqwest::Client::builder()
        .timeout(config.request_timeout)
        .build()
        .map_err(|err| LangSmithError::Config {
            reason: format!("build_client: {err}"),
        })?;
    let endpoint = config.endpoint_url()?;
    let max_attempts = 3;
    let mut last_error: Option<LangSmithError> = None;

    for attempt in 0..max_attempts {
        if attempt > 0 {
            let delay = RETRY_DELAYS_MS
                .get(attempt - 1)
                .copied()
                .unwrap_or(*RETRY_DELAYS_MS.last().unwrap_or(&500));
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        let result = client
            .post(endpoint.clone())
            .header(AUTHORIZATION, format!("Bearer {}", config.token))
            .header(CONTENT_TYPE, "application/json")
            .header("Idempotency-Key", payload.trace_id.as_str())
            .json(payload)
            .send()
            .await;

        match result {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(response) if matches!(response.status().as_u16(), 401 | 403) => {
                return Err(LangSmithError::AuthFailed);
            }
            Ok(response) if response.status().is_client_error() => {
                return Err(LangSmithError::InvalidPayload {
                    status: Some(response.status().as_u16()),
                    reason: "client_error".to_string(),
                });
            }
            Ok(response) => {
                last_error = Some(LangSmithError::ServerError {
                    status: response.status().as_u16(),
                });
            }
            Err(err) => {
                last_error = Some(LangSmithError::Network {
                    reason: err.to_string(),
                });
            }
        }
    }

    Err(LangSmithError::DroppedAfterRetries {
        last_error: last_error
            .map(|err| err.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    })
}

pub fn default_base_url(residency: Residency) -> &'static str {
    match residency {
        Residency::Sg1 => "https://langsmith.sg-1.cyberos.world",
        Residency::Eu1 => "https://langsmith.eu-1.cyberos.world",
        Residency::Us1 => "https://langsmith.us-1.cyberos.world",
        Residency::Vn1 => "https://langsmith.vn-1.cyberos.world",
    }
}

fn residency_slug(residency: Residency) -> &'static str {
    match residency {
        Residency::Sg1 => "sg-1",
        Residency::Eu1 => "eu-1",
        Residency::Us1 => "us-1",
        Residency::Vn1 => "vn-1",
    }
}

fn trim_trailing_slash(mut value: String) -> String {
    while value.ends_with('/') {
        value.pop();
    }
    value
}

#[derive(Debug, thiserror::Error)]
pub enum LangSmithError {
    #[error("langsmith auth failed")]
    AuthFailed,
    #[error("langsmith invalid payload status={status:?} reason={reason}")]
    InvalidPayload { status: Option<u16>, reason: String },
    #[error("langsmith server error status={status}")]
    ServerError { status: u16 },
    #[error("langsmith network error: {reason}")]
    Network { reason: String },
    #[error("langsmith config error: {reason}")]
    Config { reason: String },
    #[error("langsmith dropped after retries: {last_error}")]
    DroppedAfterRetries { last_error: String },
}

impl LangSmithError {
    pub fn metric_outcome(&self) -> &'static str {
        match self {
            Self::AuthFailed | Self::InvalidPayload { .. } | Self::Config { .. } => {
                "invalid_payload"
            }
            Self::Network { .. } => "langsmith_unreachable",
            Self::ServerError { .. } | Self::DroppedAfterRetries { .. } => "dropped_after_retries",
        }
    }
}
