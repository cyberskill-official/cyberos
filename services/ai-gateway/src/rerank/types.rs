//! TASK-AI-020 — Rerank type definitions.

use serde::{Deserialize, Serialize};

/// Maximum candidates per rerank request (TASK-AI-020 §1 #6).
pub const MAX_CANDIDATES: usize = 100;

/// Maximum total tokens per rerank request (TASK-AI-020 §1 #7).
/// = 8192 × 100 (cross-encoder per-pair limit × max pairs practical).
pub const MAX_TOTAL_TOKENS: u32 = 819_200;

/// Rerank request shape (TASK-AI-020 §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// The query text.
    pub query: String,
    /// Candidate documents to rank (max 100).
    pub candidates: Vec<String>,
    /// Tenant identifier for audit and fairness.
    pub tenant_id: String,
    /// If true, scores are sigmoid-normalised to [0, 1]. Default true.
    #[serde(default = "default_normalize")]
    pub normalize: bool,
}

fn default_normalize() -> bool {
    true
}

/// Rerank response shape (TASK-AI-020 §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// (original_index, score) pairs sorted descending by score.
    pub scores: Vec<(usize, f32)>,
    /// True when sidecar was unavailable; KB caller falls back to embedding similarity.
    pub skipped: bool,
    /// Model name (e.g. "bge-reranker-v2-m3").
    pub model_name: String,
    /// First 16 hex chars of model SHA-256.
    pub model_sha256: String,
    /// Sidecar version string.
    pub sidecar_version: String,
    /// Device used ("cuda" | "cpu" | "unavailable").
    pub device: String,
    /// Latency in milliseconds.
    pub elapsed_ms: u32,
    /// Number of tokens in the query.
    pub query_token_count: u32,
    /// Total tokens across all candidates.
    pub total_candidate_tokens: u32,
}

impl RerankResponse {
    /// Construct a "skipped" response when the sidecar is unavailable (§1 #12).
    pub fn skipped() -> Self {
        Self {
            scores: vec![],
            skipped: true,
            model_name: "bge-reranker-v2-m3".into(),
            model_sha256: "unknown-sidecar-down".into(),
            sidecar_version: "unknown".into(),
            device: "unavailable".into(),
            elapsed_ms: 0,
            query_token_count: 0,
            total_candidate_tokens: 0,
        }
    }
}

/// Error taxonomy for rerank operations (TASK-AI-020 §3).
#[derive(Debug, thiserror::Error)]
pub enum RerankError {
    #[error("too many candidates: max={max} actual={actual}")]
    TooManyCandidates { max: usize, actual: usize },

    #[error("token budget exceeded: query={q} candidates={c} max={m}")]
    TokenBudgetExceeded { q: u32, c: u32, m: u32 },

    #[error("sidecar unreachable at {url}: {reason}")]
    Unreachable { url: String, reason: String },

    #[error("sidecar timeout (> {budget_ms}ms)")]
    Timeout { budget_ms: u32 },

    #[error("no sidecar configured for region")]
    NoSidecarForRegion,

    #[error("breaker open for sidecar {url}")]
    BreakerOpen { url: String },
}

/// Cost for a rerank call. Self-hosted = $0 marginal (TASK-AI-020 §1 #4).
pub fn cost_for_rerank(_candidates: usize, _total_tokens: u32) -> f64 {
    0.0
}
