//! FR-AI-017 — Per-tenant response cache backed by Redis.
//!
//! Cache key = `ai_cache:v1:{tenant_id}:{sha256(redacted_prompt ␟ model ␟ persona)}`.
//! TTL per alias-class with ±10% jitter. Graceful degradation on Redis failure.

pub mod key;
pub mod redis_backend;
pub mod ttl;

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::router::types::{Choice, FinishReason, ProviderUsage};

pub use key::CacheKey;

/// Schema version — bumped on breaking payload changes; old entries treated as misses.
pub const CACHE_SCHEMA_VERSION: &str = "v1";

/// §1 #8: max payload size per cache entry.
pub const MAX_PAYLOAD_BYTES: usize = 1_048_576; // 1 MB

/// Redis timeout for all cache operations.
pub const REDIS_TIMEOUT_MS: u64 = 200;

/// Cached response payload stored in Redis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub schema_version: String,
    pub usage: ProviderUsage,
    pub choices: Vec<Choice>,
    pub finish_reason: FinishReason,
    pub cached_at: DateTime<Utc>,
    /// Original provider latency for observability.
    pub provider_ms: u64,
}

/// Outcome of a cache lookup.
#[derive(Debug)]
pub enum CacheLookupOutcome {
    /// Cache hit with the cached response and lookup latency.
    Hit(Box<CachedResponse>, Duration),
    /// No entry found.
    Miss,
    /// Stored schema version doesn't match current; treat as miss.
    SchemaMismatch,
    /// Redis error; handler should proceed to provider.
    Error(CacheError),
}

/// Outcome of a cache insert.
#[derive(Debug)]
pub enum CacheInsertOutcome {
    /// Successfully inserted.
    Inserted {
        ttl: Duration,
        jittered_ttl: Duration,
    },
    /// Intentionally not cached.
    Skipped(SkipReason),
    /// Redis or serialization error.
    Error(CacheError),
}

/// Reasons a response was not cached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// chat.long or unknown alias class.
    ChatLongOrUnknownAlias,
    /// Streaming response (§1 #6).
    Streaming,
    /// Provider returned an error (§1 #7).
    FailedResponse,
    /// Serialized payload exceeds 1MB (§1 #8).
    Oversize { actual_bytes: usize },
}

/// Cache operation errors.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("redis unreachable: {0}")]
    Unreachable(String),
    #[error("redis timeout (>200ms)")]
    Timeout,
    #[error("deserialisation failed: {0}")]
    Deserialisation(String),
}

/// Cache state for audit rows (§1 #17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheState {
    Hit,
    Miss,
    Skipped,
    Error,
}

// --- Metrics helpers ---

fn lookup_metric(tenant_id: &str, outcome: &str, latency: Duration) {
    tracing::debug!(
        tenant = %tenant_id,
        outcome = %outcome,
        latency_ms = latency.as_millis() as u64,
        "cache_lookup"
    );
}

fn error_metric(reason: &str) {
    tracing::warn!(reason = %reason, "cache_error");
}

fn oversize_metric(tenant_id: &str, bytes: usize) {
    tracing::warn!(
        tenant = %tenant_id,
        bytes = bytes,
        cap = MAX_PAYLOAD_BYTES,
        "cache_oversize_skipped"
    );
}

// --- Public API ---

/// Look up a cached response. Returns `Hit`, `Miss`, `SchemaMismatch`, or `Error`.
///
/// On `Error`, the handler MUST proceed to the provider (graceful degradation).
pub async fn lookup(key: &CacheKey) -> CacheLookupOutcome {
    redis_backend::lookup(key).await
}

/// Insert a successful provider response into the cache.
///
/// Skips: streaming, failed responses, chat.long/unknown aliases, oversize payloads.
pub async fn insert(
    key: &CacheKey,
    response: &crate::router::types::ProviderResponse,
    alias: &str,
) -> CacheInsertOutcome {
    // §1 #7: failed responses not cached.
    if !matches!(
        response.finish_reason,
        FinishReason::Stop | FinishReason::ToolCalls
    ) {
        return CacheInsertOutcome::Skipped(SkipReason::FailedResponse);
    }

    let Some(nominal_ttl) = ttl::ttl_for_alias(alias) else {
        return CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias);
    };

    let cr = CachedResponse {
        schema_version: CACHE_SCHEMA_VERSION.into(),
        usage: response.usage,
        choices: response.choices.clone(),
        finish_reason: response.finish_reason,
        cached_at: chrono::Utc::now(),
        provider_ms: response.latency_ms as u64,
    };

    let bytes = match serde_json::to_vec(&cr) {
        Ok(b) => b,
        Err(e) => {
            return CacheInsertOutcome::Error(CacheError::Deserialisation(e.to_string()));
        }
    };

    // §1 #8: 1MB cap.
    if bytes.len() > MAX_PAYLOAD_BYTES {
        oversize_metric(&key.tenant_id, bytes.len());
        return CacheInsertOutcome::Skipped(SkipReason::Oversize {
            actual_bytes: bytes.len(),
        });
    }

    let jittered = ttl::jittered_ttl(nominal_ttl, &mut rand::thread_rng());
    redis_backend::insert(key, &bytes, jittered).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::types::{AttemptRecord, Choice, ProviderResponse, ProviderUsage};

    fn test_provider_response() -> ProviderResponse {
        ProviderResponse {
            id: "test-1".into(),
            usage: ProviderUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                cached_input_tokens: 0,
            },
            choices: vec![Choice {
                index: 0,
                content: "Hello, world!".into(),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
            }],
            finish_reason: FinishReason::Stop,
            latency_ms: 150,
            cache_state: crate::router::types::CacheState::None,
            attempts: vec![],
        }
    }

    #[test]
    fn cached_response_roundtrip() {
        let cr = CachedResponse {
            schema_version: CACHE_SCHEMA_VERSION.into(),
            usage: ProviderUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                cached_input_tokens: 0,
            },
            choices: vec![Choice {
                index: 0,
                content: "test".into(),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
            }],
            finish_reason: FinishReason::Stop,
            cached_at: Utc::now(),
            provider_ms: 150,
        };
        let bytes = serde_json::to_vec(&cr).unwrap();
        let decoded: CachedResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.schema_version, CACHE_SCHEMA_VERSION);
        assert_eq!(decoded.choices[0].content, "test");
    }

    #[test]
    fn schema_version_constant() {
        assert_eq!(CACHE_SCHEMA_VERSION, "v1");
    }

    #[test]
    fn max_payload_is_1mb() {
        assert_eq!(MAX_PAYLOAD_BYTES, 1_048_576);
    }

    #[test]
    fn cache_state_serde_roundtrip() {
        let states = [
            CacheState::Hit,
            CacheState::Miss,
            CacheState::Skipped,
            CacheState::Error,
        ];
        for s in &states {
            let json = serde_json::to_string(s).unwrap();
            let decoded: CacheState = serde_json::from_str(&json).unwrap();
            assert_eq!(*s, decoded);
        }
    }
}
