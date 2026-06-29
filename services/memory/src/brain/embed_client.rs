//! FR-MEMORY-123 §1 #2,#13 / DEC-2723 — the ONLY embedding path: the ai-gateway embeddings endpoint.
//!
//! The brain worker NEVER calls a model provider directly. It POSTs to the ai-gateway, which owns model
//! routing, the residency pin, the ZDR flag, and the tenant spend cap (FR-AI-022). The gateway charges the
//! embedding against the tenant cap and returns `402` when exhausted — at which point the worker marks the
//! row `pending_embed_retry` and backs off (it has NO code path that falls back to a provider, by design).
//!
//! Contract (FR §3, matching the ai-gateway router's `EmbedRequest`/`EmbedResponse` stub types):
//!   POST {AI_GATEWAY_URL}/v1/embeddings
//!   header  x-tenant-id: <tenant_uuid>        (how the gateway resolves the TenantPolicy)
//!   body    { "input": ["<text>"], "model": "bge-m3" }
//!   ->
//!   200  { "embeddings": [[...1024 f32...]], "model": "bge-m3", "embed_model_version": "bge-m3@..." }
//!   402  spend cap exhausted -> SpendCapExhausted (mark pending, back off; DO NOT call a provider)
//!   5xx / timeout / connect error -> GatewayDown (mark pending, retry next tick)
//!
//! Backoff (§1 #13) is exponential — 100ms, 250ms, 500ms, 1s, 2s — up to 5 attempts before the caller marks
//! the row pending. A `402` is terminal for this pass (the cap will not clear by retrying in-loop), so it
//! short-circuits the backoff immediately.
//!
//! NOTE (honest dependency edge): the ai-gateway's `/v1/embeddings` HTTP route is the FR-AI-019 + FR-AI-022
//! contract this FR depends on. At the time of writing the gateway exposes `/v1/chat` and the embeddings
//! route is a documented-but-unwired contract (router `EmbedRequest`/`EmbedResponse` types exist). This
//! client targets the contract exactly so it works the moment the gateway lights the route up; it is
//! exercised in tests via a stub gateway (`EmbedClient::stub`).

use std::time::Duration;
use uuid::Uuid;

use super::EMBED_DIM;

/// The embeddings model alias the brain requests (FR-AI-019 bge-m3). The gateway maps the alias to the
/// in-region model per the tenant policy and echoes the resolved `embed_model_version`.
pub const BRAIN_EMBED_MODEL: &str = "bge-m3";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const BACKOFF_MS: [u64; 5] = [100, 250, 500, 1000, 2000];

/// An embedding failure the ingest worker handles by marking the row pending + backing off (§1 #13). It has
/// NO variant that represents a direct-provider fallback — that path does not exist.
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    /// The gateway returned `402`: the tenant spend cap is exhausted. Mark `pending_embed_retry`; do NOT
    /// call a provider directly (DEC-2723).
    #[error("spend cap exhausted")]
    SpendCapExhausted,
    /// The gateway is unreachable / 5xx / timed out after the backoff budget. Mark `pending_embed_retry`.
    #[error("ai-gateway unavailable: {0}")]
    GatewayDown(String),
    /// The gateway answered but the embedding shape was wrong (e.g. dim != 1024). Surfaced so the dim-
    /// mismatch failure mode (§10) is distinguishable from a transport failure.
    #[error("malformed embedding response: {0}")]
    Malformed(String),
}

/// The result of an embedding call: the vector plus the gateway-echoed model version (recorded per row for
/// re-embed migrations, §1 #14).
#[derive(Debug, Clone)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model_version: String,
}

#[derive(serde::Serialize)]
struct GatewayEmbedRequest<'a> {
    input: [&'a str; 1],
    model: &'a str,
}

#[derive(serde::Deserialize)]
struct GatewayEmbedResponse {
    embeddings: Vec<Vec<f32>>,
    #[serde(default)]
    embed_model_version: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

/// The embeddings client. Targets the ai-gateway; the tenant policy (region, ZDR, alias, spend cap) is
/// resolved gateway-side from the `x-tenant-id` header — this client carries no policy of its own (so it can
/// never drift from FR-AI-022).
#[derive(Clone, Debug)]
pub struct EmbedClient {
    gateway_url: String,
    http: reqwest::Client,
    /// When set (tests / offline rebuild), the gateway call is bypassed and a deterministic stub vector is
    /// returned. NEVER set in production — the env constructor leaves it `None`, so the only path is the
    /// gateway. A `stub_402` stub additionally forces the spend-cap branch for the residency/spend test.
    stub: Option<StubMode>,
}

#[derive(Clone, Copy, Debug)]
enum StubMode {
    /// Deterministic hashed vector (lets ingest + recall round-trip without a live gateway).
    Deterministic,
    /// Always return SpendCapExhausted (proves over-cap -> pending, no direct provider call).
    Force402,
}

impl EmbedClient {
    /// Construct from the environment. `AI_GATEWAY_URL` (preferred) or `BRAIN_EMBED_GATEWAY_URL` names the
    /// gateway base; default `http://127.0.0.1:8080` (the gateway's `AI_GATEWAY_BIND` default). The client
    /// always routes through the gateway — there is no provider-direct env toggle.
    pub fn from_env() -> Self {
        let gateway_url = std::env::var("AI_GATEWAY_URL")
            .or_else(|_| std::env::var("BRAIN_EMBED_GATEWAY_URL"))
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let http = reqwest::Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            gateway_url,
            http,
            stub: None,
        }
    }

    /// A test client that returns a deterministic vector derived from the text (no network). Lets the ingest
    /// + recall round-trip + summaries tests run on Postgres+pgvector without a live ai-gateway.
    pub fn stub() -> Self {
        Self {
            gateway_url: String::new(),
            http: reqwest::Client::new(),
            stub: Some(StubMode::Deterministic),
        }
    }

    /// A test client that always reports the spend cap as exhausted (§1 #15 / AC #15): proves the worker
    /// marks `pending_embed_retry` and never bypasses the gateway.
    pub fn stub_force_spend_cap() -> Self {
        Self {
            gateway_url: String::new(),
            http: reqwest::Client::new(),
            stub: Some(StubMode::Force402),
        }
    }

    /// Embed one body through the ai-gateway (§1 #2). On `402` returns `SpendCapExhausted` immediately
    /// (terminal for this pass); on a transport error retries with exponential backoff up to 5 attempts then
    /// returns `GatewayDown`. The dim is validated against [`EMBED_DIM`] (§10 dim-mismatch failure).
    pub async fn embed(&self, tenant_id: Uuid, body: &str) -> Result<Embedding, EmbedError> {
        if let Some(mode) = self.stub {
            return match mode {
                StubMode::Force402 => Err(EmbedError::SpendCapExhausted),
                StubMode::Deterministic => Ok(Embedding {
                    vector: deterministic_vector(body),
                    model_version: "stub-bge-m3@test".to_string(),
                }),
            };
        }

        let url = format!("{}/v1/embeddings", self.gateway_url.trim_end_matches('/'));
        let req = GatewayEmbedRequest {
            input: [body],
            model: BRAIN_EMBED_MODEL,
        };

        let mut last_err = String::new();
        for (attempt, delay_ms) in BACKOFF_MS.iter().enumerate() {
            match self
                .http
                .post(&url)
                .header("x-tenant-id", tenant_id.to_string())
                .json(&req)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    // 402 -> spend cap exhausted (terminal for this pass; retrying in-loop won't clear it).
                    if status.as_u16() == 402 {
                        return Err(EmbedError::SpendCapExhausted);
                    }
                    if status.is_success() {
                        let parsed: GatewayEmbedResponse = resp
                            .json()
                            .await
                            .map_err(|e| EmbedError::Malformed(e.to_string()))?;
                        let vector = parsed.embeddings.into_iter().next().ok_or_else(|| {
                            EmbedError::Malformed("no embedding in response".into())
                        })?;
                        if vector.len() != EMBED_DIM {
                            return Err(EmbedError::Malformed(format!(
                                "embedding dim {} != {EMBED_DIM}",
                                vector.len()
                            )));
                        }
                        let model_version = parsed
                            .embed_model_version
                            .or(parsed.model)
                            .unwrap_or_else(|| BRAIN_EMBED_MODEL.to_string());
                        return Ok(Embedding {
                            vector,
                            model_version,
                        });
                    }
                    // 5xx (and any other non-2xx, non-402) -> retry with backoff.
                    last_err = format!("gateway status {status}");
                }
                Err(e) => {
                    last_err = e.to_string();
                }
            }
            // Don't sleep after the final attempt.
            if attempt + 1 < BACKOFF_MS.len() {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
            }
        }
        Err(EmbedError::GatewayDown(last_err))
    }

    /// The model alias this client requests; the per-row `embed_model_version` comes from the gateway's
    /// echo on each call (so a mixed-version migration is observable, §1 #14).
    pub fn model_alias(&self) -> &'static str {
        BRAIN_EMBED_MODEL
    }
}

/// A deterministic, normalised pseudo-embedding of `text` for tests: same text -> same vector, different
/// text -> different vector, so cosine ranking is meaningful in the round-trip tests. NOT used in production
/// (only the `stub()` client calls it). A simple FNV-1a hash seeds each dimension.
fn deterministic_vector(text: &str) -> Vec<f32> {
    let mut v = vec![0f32; EMBED_DIM];
    let mut h: u64 = 0xcbf29ce484222325;
    for (i, b) in text.bytes().enumerate() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
        let idx = (h as usize ^ i) % EMBED_DIM;
        v[idx] += 1.0;
    }
    // L2-normalise so cosine distance behaves; guard the zero vector (empty text).
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    } else {
        v[0] = 1.0;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_is_deterministic_and_normalised() {
        let c = EmbedClient::stub();
        let a = c.embed(Uuid::nil(), "hello world").await.unwrap();
        let b = c.embed(Uuid::nil(), "hello world").await.unwrap();
        assert_eq!(a.vector, b.vector);
        assert_eq!(a.vector.len(), EMBED_DIM);
        let norm = a.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-3, "stub vector must be unit-norm");
    }

    #[tokio::test]
    async fn stub_distinguishes_different_text() {
        let c = EmbedClient::stub();
        let a = c.embed(Uuid::nil(), "shipped the proj sync").await.unwrap();
        let b = c
            .embed(Uuid::nil(), "completely unrelated text")
            .await
            .unwrap();
        assert_ne!(a.vector, b.vector);
    }

    #[tokio::test]
    async fn force_spend_cap_stub_returns_402_variant() {
        let c = EmbedClient::stub_force_spend_cap();
        let err = c.embed(Uuid::nil(), "over budget").await.unwrap_err();
        assert!(matches!(err, EmbedError::SpendCapExhausted));
    }

    #[test]
    fn empty_text_yields_unit_vector_not_nan() {
        let v = deterministic_vector("");
        assert_eq!(v.len(), EMBED_DIM);
        assert!(v.iter().all(|x| x.is_finite()));
        assert!((v.iter().map(|x| x * x).sum::<f32>().sqrt() - 1.0).abs() < 1e-6);
    }
}
