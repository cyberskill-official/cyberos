//! FR-AI-019 — Self-hosted BGE-M3 provider adapter.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};
use serde::Deserialize;
use tracing::warn;

use super::bge_batch_buffer::BatchBuffer;
use super::{
    ChatCompleteRequest, EmbedRequest, EmbedResponse, Provider, ProviderResponse,
    ProviderStreamResponse, RouterError,
};
use crate::policy::ProviderKind;

pub use super::EmbedTask;

/// Sidecar config file parsed from `config/embeddings.yaml`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct EmbeddingsConfig {
    pub bge_sidecars: Vec<BgeSidecarConfig>,
}

/// One region-local BGE sidecar.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BgeSidecarConfig {
    pub region: String,
    pub url: String,
}

/// BGE provider implementation.
#[derive(Debug, Clone)]
pub struct BgeProvider {
    sidecars: Arc<HashMap<String, String>>,
    batch_buffer: BatchBuffer,
    http_client: reqwest::Client,
    last_device_by_url: Arc<Mutex<HashMap<String, String>>>,
}

static BGE_REQUESTS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_bge_requests_total",
        "BGE requests by tenant, batch size, device, and outcome",
        &["tenant_id", "batch_size_bucket", "device", "outcome"]
    )
    .unwrap()
});

static BGE_LATENCY_MS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "ai_bge_latency_ms",
        "BGE sidecar latency in ms",
        &["device", "batch_size_bucket"],
        vec![10.0, 25.0, 50.0, 100.0, 300.0, 1_000.0]
    )
    .unwrap()
});

static BGE_FALLBACK_TO_CPU: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_bge_fallback_to_cpu_total",
        "BGE sidecar device transitions from cuda to cpu",
        &["sidecar_url"]
    )
    .unwrap()
});

impl BgeProvider {
    /// Build a provider from an explicit region -> URL map.
    pub fn new(sidecars: HashMap<String, String>) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            sidecars: Arc::new(sidecars),
            batch_buffer: BatchBuffer::new(http_client.clone()),
            http_client,
            last_device_by_url: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load a provider from an embeddings YAML config file.
    pub fn from_config_path(path: &Path) -> Result<Self, RouterError> {
        Ok(Self::new(load_sidecar_urls(path)?))
    }

    /// Load from `CYBEROS_AI_EMBEDDINGS_CONFIG`, or the crate-local default config.
    pub fn from_default_config() -> Result<Self, RouterError> {
        let path = std::env::var("CYBEROS_AI_EMBEDDINGS_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/embeddings.yaml")
            });
        Self::from_config_path(&path)
    }

    /// Marginal cost for self-hosted embeddings.
    pub fn cost_for(&self, _model: &str, _tokens: u32) -> f64 {
        0.0
    }

    /// Health-check every configured sidecar.
    pub async fn health_check_all_sidecars(&self) -> Result<(), RouterError> {
        for (region, url) in self.sidecars.iter() {
            let health_url = format!("{}/health", url.trim_end_matches('/'));
            let response = self
                .http_client
                .get(&health_url)
                .send()
                .await
                .map_err(|source| RouterError::TerminalProviderError {
                    provider: ProviderKind::Bge,
                    status: 503,
                    message: format!("{region}: {source}"),
                    retry_after_secs: None,
                })?;
            if !response.status().is_success() {
                return Err(RouterError::TerminalProviderError {
                    provider: ProviderKind::Bge,
                    status: response.status().as_u16(),
                    message: format!("{region}: health check failed"),
                    retry_after_secs: None,
                });
            }
        }
        Ok(())
    }

    async fn call_embed_inner(
        &self,
        req: &EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        let Some(url) = self.sidecars.get(&req.region).cloned() else {
            return Err(RouterError::NoSidecarForRegion {
                region: req.region.clone(),
            });
        };
        let started = Instant::now();
        let response = self
            .batch_buffer
            .submit(&url, req.clone(), model, deadline)
            .await;
        match &response {
            Ok(resp) => {
                self.observe_device(&url, &resp.device);
                BGE_REQUESTS
                    .with_label_values(&[
                        &req.tenant_id,
                        batch_bucket(req.texts.len()),
                        &resp.device,
                        "ok",
                    ])
                    .inc();
                BGE_LATENCY_MS
                    .with_label_values(&[&resp.device, batch_bucket(req.texts.len())])
                    .observe(started.elapsed().as_millis() as f64);
            }
            Err(RouterError::TerminalProviderError { status: 413, .. }) => {
                BGE_REQUESTS
                    .with_label_values(&[
                        &req.tenant_id,
                        batch_bucket(req.texts.len()),
                        "unknown",
                        "input_too_long",
                    ])
                    .inc();
            }
            Err(RouterError::NoSidecarForRegion { .. }) => {
                BGE_REQUESTS
                    .with_label_values(&[
                        &req.tenant_id,
                        batch_bucket(req.texts.len()),
                        "unknown",
                        "sidecar_unreachable",
                    ])
                    .inc();
            }
            Err(_) => {
                BGE_REQUESTS
                    .with_label_values(&[
                        &req.tenant_id,
                        batch_bucket(req.texts.len()),
                        "unknown",
                        "sidecar_unreachable",
                    ])
                    .inc();
            }
        }
        response
    }

    fn observe_device(&self, url: &str, device: &str) {
        let mut guard = self
            .last_device_by_url
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.get(url).map(String::as_str) == Some("cuda") && device == "cpu" {
            BGE_FALLBACK_TO_CPU.with_label_values(&[url]).inc();
            warn!(
                sidecar_url = %url,
                severity = "sev-2",
                "ai_bge_gpu_failed"
            );
        }
        guard.insert(url.to_string(), device.to_string());
    }
}

#[async_trait]
impl Provider for BgeProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Bge
    }

    async fn call_chat(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "BGE provider only supports embeddings".to_string(),
        })
    }

    async fn call_embed(
        &self,
        req: &EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        self.call_embed_inner(req, model, deadline).await
    }

    async fn call_chat_streaming(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderStreamResponse, RouterError> {
        Err(RouterError::StreamingNotImplemented)
    }
}

/// Parse `config/embeddings.yaml` into a region -> URL map.
pub fn load_sidecar_urls(path: &Path) -> Result<HashMap<String, String>, RouterError> {
    let yaml = std::fs::read_to_string(path).map_err(|source| RouterError::InvalidResponse {
        reason: format!(
            "failed to read embeddings config {}: {source}",
            path.display()
        ),
    })?;
    let parsed: EmbeddingsConfig =
        serde_yaml::from_str(&yaml).map_err(|source| RouterError::InvalidResponse {
            reason: format!(
                "failed to parse embeddings config {}: {source}",
                path.display()
            ),
        })?;
    let mut out = HashMap::new();
    for sidecar in parsed.bge_sidecars {
        if sidecar.region.trim().is_empty() || sidecar.url.trim().is_empty() {
            return Err(RouterError::InvalidResponse {
                reason: "BGE sidecar region/url must be non-empty".to_string(),
            });
        }
        out.insert(sidecar.region, sidecar.url);
    }
    Ok(out)
}

fn batch_bucket(size: usize) -> &'static str {
    match size {
        0 | 1 => "1",
        2..=8 => "2_8",
        9..=16 => "9_16",
        17..=32 => "17_32",
        _ => "gt_32",
    }
}
