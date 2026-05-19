//! FR-AI-019 — bge-m3 embeddings client.
//!
//! The actual ONNX/sentence-transformers model lives in a sidecar process
//! (per FR-AI-019 §3 — keeps the Rust binary lean and lets ops scale GPU
//! capacity independently of the memory service). This module is the HTTP
//! client that talks to it.
//!
//! Sidecar contract:
//!   POST {EMBED_URL}/embed
//!   { "texts": ["...", "..."], "model": "bge-m3" }
//!   →
//!   { "embeddings": [[0.012, -0.0083, ...], ...], "model": "bge-m3", "dim": 1024 }
//!
//! On error (sidecar down, timeout, malformed response), `embed_batch`
//! returns an error and the ingest pipeline records the row WITHOUT an
//! embedding. The 60-second background re-embedder will retry. This means
//! FR-AI-019 is **fail-open** at ingest time — a flaky sidecar doesn't
//! halt ingest; it just degrades search quality temporarily.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_DIM: usize = 1024;
const BGE_M3: &str = "bge-m3";

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("sidecar URL not configured (set MEMORY_EMBED_URL)")]
    NotConfigured,
    #[error("network: {0}")]
    Network(String),
    #[error("sidecar returned {0}")]
    BadStatus(u16),
    #[error("malformed sidecar response: {0}")]
    Malformed(String),
    #[error("empty batch — call with at least one text")]
    EmptyBatch,
}

#[derive(Debug, Clone, Serialize)]
struct EmbedRequest<'a> {
    texts: &'a [&'a str],
    model: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    dim: usize,
}

#[derive(Debug, Clone)]
pub struct EmbeddingClient {
    pub base_url: String,
    pub timeout: Duration,
}

impl EmbeddingClient {
    /// Construct from the `MEMORY_EMBED_URL` env var. Returns `NotConfigured`
    /// if the env var is missing — callers treat this as "skip embeddings".
    pub fn from_env() -> Result<Self, EmbedError> {
        let url = std::env::var("MEMORY_EMBED_URL").map_err(|_| EmbedError::NotConfigured)?;
        Ok(Self {
            base_url: url,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Embed a batch of texts. Returns one `Vec<f32>` per input, all of length
    /// `DEFAULT_DIM` (1024). The sidecar may return a different `dim` if its
    /// config changed — we surface that mismatch as `Malformed`.
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedError> {
        if texts.is_empty() {
            return Err(EmbedError::EmptyBatch);
        }
        let req = EmbedRequest {
            texts,
            model: BGE_M3,
        };
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| EmbedError::Network(e.to_string()))?;
        let url = format!("{}/embed", self.base_url.trim_end_matches('/'));
        let resp = client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EmbedError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(EmbedError::BadStatus(resp.status().as_u16()));
        }
        let body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| EmbedError::Malformed(e.to_string()))?;
        if body.embeddings.len() != texts.len() {
            return Err(EmbedError::Malformed(format!(
                "expected {} embeddings, got {}",
                texts.len(),
                body.embeddings.len()
            )));
        }
        for (i, e) in body.embeddings.iter().enumerate() {
            if e.len() != DEFAULT_DIM {
                return Err(EmbedError::Malformed(format!(
                    "embedding {i} has dim {} (expected {DEFAULT_DIM})",
                    e.len()
                )));
            }
        }
        Ok(body.embeddings)
    }

    /// Convenience: embed a single string.
    pub async fn embed_one(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let mut v = self.embed_batch(&[text]).await?;
        Ok(v.remove(0))
    }
}

/// Format an embedding vector as the pgvector literal Postgres expects:
///   `[0.012,-0.0083,...]`
/// Used when binding via `sqlx::query("... $1::vector ...")` with a TEXT param.
pub fn to_pgvector_literal(v: &[f32]) -> String {
    let mut s = String::with_capacity(v.len() * 8 + 2);
    s.push('[');
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        // pgvector parses standard float syntax; use 6 sig figs (matches bge-m3 precision).
        s.push_str(&format!("{x:.6}"));
    }
    s.push(']');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_pgvector_literal_round_trip_shape() {
        let v = vec![0.123_456_f32, -0.0083_f32, 1.0_f32];
        let s = to_pgvector_literal(&v);
        assert!(s.starts_with('['));
        assert!(s.ends_with(']'));
        assert_eq!(s.matches(',').count(), 2);
    }

    #[test]
    fn to_pgvector_literal_empty() {
        assert_eq!(to_pgvector_literal(&[]), "[]");
    }

    #[test]
    fn to_pgvector_literal_handles_negative_and_zero() {
        let s = to_pgvector_literal(&[-1.5, 0.0, 1.5]);
        assert!(s.contains("-1.500000"));
        assert!(s.contains("0.000000"));
        assert!(s.contains("1.500000"));
    }
}
