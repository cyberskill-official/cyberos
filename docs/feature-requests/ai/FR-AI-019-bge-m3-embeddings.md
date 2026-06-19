---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-019
title: "Self-hosted BGE-M3 embeddings (single L4 GPU sidecar) + ONNX-CPU fallback + adaptive batching"
module: AI
priority: SHOULD
status: ready_to_test
verify: T
phase: P0
milestone: P0 · slice 4
slice: 4
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-18
memory_chain_hash: null
related_frs: [FR-AI-001, FR-AI-006, FR-AI-007, FR-AI-008, FR-AI-009, FR-AI-016, FR-AI-020]
depends_on: []
blocks: [FR-AI-020, FR-MEMORY-101, FR-KB-005]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#embeddings
  - website/docs/modules/memory.html#layer-2-vector-index
source_decisions:
  - DEC-091 (self-hosted multilingual embeddings; license-permissive AND VN-aware)
  - Cost ceiling: at 50 tenants × 1M chunks × 100 tok/chunk = 5B tok/mo; managed = $1500/mo, self-host = $360/mo (L4)
  - archive/2026-05-14/RESEARCH_REVIEW.md §6.2 (BGE-M3 vs OpenAI vs Cohere on Vietnamese MTEB benchmarks)

# ───── Build envelope ─────
language: rust 1.81 (adapter) + python 3.11 (sidecar)
service: cyberos/services/ai-gateway/embeddings/
new_files:
  - services/ai-gateway/embeddings/sidecar/bge_m3_sidecar.py
  - services/ai-gateway/embeddings/sidecar/health.py
  - services/ai-gateway/embeddings/sidecar/batch.py
  - services/ai-gateway/embeddings/sidecar/checksum.py
  - services/ai-gateway/embeddings/sidecar/requirements.txt
  - services/ai-gateway/embeddings/Dockerfile.gpu
  - services/ai-gateway/embeddings/Dockerfile.cpu
  - services/ai-gateway/embeddings/docker-compose.yml
  - services/ai-gateway/embeddings/checksums/bge-m3.sha256
  - services/ai-gateway/src/router/bge_provider.rs
  - services/ai-gateway/src/router/bge_batch_buffer.rs
  - services/ai-gateway/tests/otel_test.rs
  - services/ai-gateway/tests/cache_test.rs
  - services/ai-gateway/tests/rerank_test.rs
modified_files:
  - services/ai-gateway/src/router/mod.rs               # add BgeProvider variant
  - services/ai-gateway/src/router/provider.rs          # Provider trait covers call_embed
  - services/ai-gateway/config/cost_rates.yaml          # add bge-m3 entries (cost = 0)
  - services/ai-gateway/Cargo.toml                      # add reqwest, tokio
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,embeddings}/**
  - bash: docker compose -f services/ai-gateway/embeddings/docker-compose.yml up -d
  - bash: cargo test -p cyberos-ai-gateway bge
disallowed_tools:
  - send embeddings to managed APIs when bge sidecar is healthy (router prefers self-hosted per cost rule)
  - bypass tenant residency on BGE inference (sidecar deployment is per-region; FR-AI-016 enforces)
  - load the BGE-M3 model without verifying checksum at startup (per §1 #11)
  - hardcode sidecar URL (use config-driven discovery per §1 #14)
  - emit cost > 0 in `ai.invocation` rows for BGE calls (the marginal cost IS zero; amortised infra cost is tracked separately)

# ───── Estimated work ─────
effort_hours: 12
sub_tasks:
  - "1.0h: Python sidecar (FastAPI + sentence-transformers + batch endpoint + health)"
  - "0.5h: Model checksum verification at sidecar startup (`checksum.py`)"
  - "1.0h: GPU detection + ONNX CPU fallback variant"
  - "1.0h: Sidecar batch processor (internal queue with timeout-or-32 fan-in)"
  - "0.5h: Sidecar Dockerfiles (GPU CUDA-base + CPU slim)"
  - "0.5h: docker-compose with healthcheck + restart policy"
  - "1.0h: Rust adapter (`BgeProvider`) implementing the `Provider` trait"
  - "1.0h: Rust-side adaptive batch buffer (32-or-50ms fan-in)"
  - "0.5h: Per-tenant fairness in batch buffer (round-robin per tenant_id; no starvation)"
  - "0.5h: cost_rates.yaml entry (bge-m3 input=0, output=0; comment explains amortised infra cost)"
  - "0.5h: Circuit-breaker integration (FR-AI-009 wraps BgeProvider)"
  - "0.5h: Health-check at startup; refuse to bind if sidecar unhealthy"
  - "0.5h: Mid-run GPU failure detection + live failover (sidecar reports `device: cpu` in subsequent responses)"
  - "0.5h: Max-input-length validation (8192 tokens BGE-M3 limit); reject longer inputs with 413"
  - "0.5h: Sidecar/model version in response (model_name, model_sha256, sidecar_version)"
  - "1.5h: Tests — single embed, batch, GPU/CPU latency, fallback, fairness, max-length-rejection, version-reporting"
  - "0.5h: OTel metrics emission"
risk_if_skipped: "Embeddings cost ~$0.00002–$0.0001/1k tokens via managed APIs (OpenAI text-embedding-3-small / Cohere embed-multilingual-v3). At memory Layer 2 ingest scale (50 tenants × 1M chunks × 100 tok avg = 5B tokens/month), that's $100/mo per tenant of pure embeddings spend — eating ~25% of the $4/user/month target. Self-host with L4 GPU = $360/mo total fixed cost; per-tenant marginal cost = $0; break-even at ~3 active tenants. Without this FR, FR-AI-020 (BGE rerank, also self-hosted) has no shared infrastructure — adding 2x the operational burden."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **SHOULD** offer a self-hosted BGE-M3 embeddings provider as a sidecar service, routed via the standard `Provider` trait. The provider and sidecar together obey the following:

1. **MUST** serve BAAI/bge-m3 (1024-dim, multilingual, 8192-token max input) on GPU (L4 or better) when CUDA is available; ONNX-quantised CPU fallback when not. The sidecar detects at startup which device is available and loads the corresponding model variant.
2. **MUST** be invokable via the `embed.standard` and `embed.code` aliases (FR-AI-006). `BgeProvider` is registered as one of the `Provider` enum variants in FR-AI-008's router.
3. **MUST** expose a single batch HTTP endpoint at the sidecar (`POST /embed`) that accepts a list of texts and returns a list of embeddings in the same order. Single-text requests are a degenerate batch of size 1; the API doesn't expose a separate single-embed endpoint.
4. **MUST** implement Rust-side adaptive batching in `BgeProvider`: concurrent `call_embed` invocations buffer for up to **50ms** OR until **32 texts** accumulate (whichever first), then send one HTTP request to the sidecar. The 50ms ceiling caps user-visible latency; the 32-text floor maximises GPU efficiency.
5. **MUST** apply per-tenant fairness in the batch buffer: when the buffer holds requests from multiple tenants, the dispatch order is round-robin per tenant_id within the batch. A single tenant submitting 100 concurrent embeds CANNOT starve another tenant's submitted-during-the-same-window request — both get served in the next batch.
6. **MUST** report `cost: 0.0` to the cost-ledger per BGE call (self-hosted = no marginal cost). The amortised infra cost (L4 GPU rental ~$360/month) is tracked OUTSIDE the per-call cost ledger via a separate accounting channel (FR-AI-022 follow-up). This split is documented in §2.
7. **MUST** integrate with the FR-AI-009 circuit breaker: sustained sidecar failures (5 consecutive HTTP 5xx OR 5 consecutive timeouts) trigger breaker opening; subsequent calls fail fast (no sidecar contact) until the half-open probe succeeds. The breaker state is per-sidecar-instance (not per-tenant).
8. **MUST** expose `/health` returning HTTP 200 when (a) the model is loaded, (b) the device is detected, (c) a 1-text test embedding completes successfully. Returns HTTP 503 during warmup, model-loading errors, or test-embedding failures. The gateway's startup check refuses to bind if `/health` is non-200 within 60s.
9. **MUST** support both `embed.standard` (BGE-M3 dense default) and `embed.code` (BGE-M3 with code-tuned prompt template prefix `"Code: "`) via a `task` field in the request body. The sidecar applies the prompt-template variation; the model itself is the same checkpoint.
10. **MUST** complete an embedding request within **50ms p95** for batches ≤ 32 texts on an L4 GPU; CPU-fallback budget widens to **300ms p95**. SLO assertions live in `bge_test.rs` with 1000-sample latency measurements.
11. **MUST** verify the BGE-M3 model SHA-256 checksum at sidecar startup against `embeddings/checksums/bge-m3.sha256`. Mismatch (corrupted download, supply-chain tamper) → sidecar refuses to start with `ChecksumMismatch` exit code; gateway refuses to bind. The checksum is pinned per FR amendment; model-version updates require both file replacement AND checksum file update in the same PR.
12. **MUST** validate input text length ≤ 8192 tokens (BGE-M3's hard limit) BEFORE embedding. Over-length inputs return HTTP 413 PAYLOAD_TOO_LARGE with body `{"error":"input_too_long","max_tokens":8192,"actual_tokens":<n>,"text_index":<i>}` identifying which input in the batch exceeded. The sidecar uses the model's tokeniser for the count; no upstream chunking happens here.
13. **MUST** report `model_name` (`"bge-m3"`), `model_sha256` (first 16 hex of model checksum), `sidecar_version` (semver from `pyproject.toml`), and `device` (`"cuda" | "cpu"`) in every response. Downstream consumers (memory Layer 2) record these for index-version pinning.
14. **MUST** read the sidecar URL from `services/ai-gateway/config/embeddings.yaml`:
    ```yaml
    bge_sidecars:
      - region: ap-southeast-1
        url: http://bge-sidecar-sg-1:5060
      - region: eu-central-1
        url: http://bge-sidecar-eu-1:5060
    ```
    Per-region deployment is required to satisfy FR-AI-016 residency pinning. The router selects the sidecar matching the resolved region; absence of a sidecar in the required region is a `ResidencyViolation` (handled by FR-AI-016).
15. **MUST** detect mid-run GPU failure: if the sidecar's `device` field flips from `"cuda"` to `"cpu"` between two consecutive responses, the gateway emits sev-2 OBS event `ai_bge_gpu_failed` and adjusts its latency-budget alerting (300ms p95 instead of 50ms). The failover itself is sidecar-internal (PyTorch fallback to CPU); the gateway just observes.
16. **SHOULD** emit OTel metrics:
    - `ai_bge_requests_total{tenant_id, batch_size_bucket, device, outcome}` (counter; outcome ∈ ok | input_too_long | sidecar_unreachable | breaker_open).
    - `ai_bge_latency_ms{device, batch_size_bucket}` (histogram; SLO 50ms p95 GPU / 300ms p95 CPU).
    - `ai_bge_gpu_utilization_pct{sidecar_url}` (gauge from `nvidia-smi` polling within the sidecar; 0 when CPU mode).
    - `ai_bge_fallback_to_cpu_total{sidecar_url}` (counter; sev-2 alarm on increment).
    - `ai_bge_batch_buffer_depth{tenant_id}` (gauge; high values indicate buffering saturation).
    - `ai_bge_checksum_failed_total` (counter; sev-1 — model corruption or supply-chain attack).

---

## §2 — Why this design (rationale for humans)

**Why self-host BGE-M3 specifically?** Three converging reasons. (1) Multilingual support — BGE-M3 handles Vietnamese natively; English-only models (text-embedding-3-small, text-embedding-3-large) underperform on VN content by ~10% on MTEB Vietnamese benchmarks. (2) License — MIT-licensed; safe to self-host commercially. (3) Performance — top-3 on MTEB-multilingual at 1024 dimensions, which is the right grain for HNSW indexing in memory Layer 2 (768 too coarse, 1536 too memory-heavy). The alternatives (OpenAI text-embedding-3-small at $0.00002/1k, Cohere embed-multilingual-v3 at $0.0001/1k) cost $20–$100/month per active tenant at our ingest volume.

**Why batch (§1 #4)?** GPU inference latency is dominated by warmup overhead (kernel launch, model dispatch). For BGE-M3 on L4: a single-text request is ~30ms (almost all warmup); a 32-text batch is ~40ms total. Batching gives ~25× throughput per dollar of GPU time. The 50ms buffer ceiling is the user-facing ceiling — long enough to accumulate batches in production, short enough to be invisible to humans.

**Why per-tenant fairness in the batch buffer (§1 #5)?** Without round-robin dispatch, a tenant submitting 100 concurrent embeds at 1ms intervals would fill the buffer entirely with their requests; another tenant's single request submitted at ms 25 would queue behind 75 others — observed latency 1500ms instead of 50ms. Round-robin within the batch ensures every tenant in the buffer at dispatch time gets at least one request through. The cost (slightly less efficient batching when many tenants are present) is worth the fairness guarantee.

**Why cost = 0 in the cost ledger (§1 #6)?** The marginal cost of one additional BGE embedding IS zero — the GPU is rented monthly, idle or busy. Reporting non-zero per-call cost would force an arbitrary amortisation choice (per-call? per-tenant? per-token?) that doesn't map to actual billing. The amortised infra cost (~$360/mo per L4) is tracked separately in operational accounting (FR-AI-022 will surface it as `ai_infra_amortised_usd_per_day` for cost dashboards). The cost ledger semantics ("per-call dollars to the provider") stay clean.

**Why CPU fallback (§1 #1)?** L4 GPUs aren't always available — dev machines, certain residency regions where AWS hasn't provisioned GPU instances yet (Vn-1 future), CI runners. ONNX-quantised BGE-M3 runs on CPU at ~6× slower (300ms vs 50ms), well within FR-AI-008's per-call budget. Without CPU fallback, dev workflows would require GPU access (expensive friction).

**Why model checksum verification (§1 #11)?** Models are downloaded artefacts (typically from HuggingFace). Supply-chain attack surface: a compromised HuggingFace mirror could swap a model for one that produces consistently-biased embeddings (steering retrieval to attacker-favoured content). The SHA-256 pin ensures the running model is byte-identical to the version we audited. Any drift (corrupted download, attempted swap) fails the startup check loud-and-early.

**Why expose `model_name` + `model_sha256` in every response (§1 #13)?** memory Layer 2 stores embeddings in a vector index. If we silently switch from `bge-m3@1.0` to `bge-m3@1.1`, old embeddings in the index are now in a different vector space — retrieval quality silently degrades. Embedding the model identity in every response gives downstream consumers the signal to invalidate / re-embed. This is the same lock-step versioning principle as FR-AI-014's persona handle and FR-AI-017's cache schema version.

**Why max-input-length validation (§1 #12)?** BGE-M3's tokeniser silently truncates inputs > 8192 tokens — producing embeddings for "the first 8192 tokens of your text" rather than "your text." A user embedding a 50-page document would get a partial-document vector with no error indication. Validating BEFORE embedding and returning HTTP 413 makes the failure loud; the caller can decide to chunk or reject.

**Why per-region sidecar deployment (§1 #14)?** FR-AI-016 (residency pinning) requires data to stay in-region. A request from a `Sg1` tenant must have its embedding computed by a sidecar in `ap-southeast-1`, not routed to an `eu-central-1` GPU. Per-region sidecars + region-aware router selection is the implementation. Tenants in regions without a sidecar (e.g., `Vn1` in slice 4) fail with `ResidencyViolation` per FR-AI-016 §1 #6.

**Why mid-run GPU-failure detection (§1 #15)?** PyTorch can fall back to CPU at runtime (e.g., GPU memory pressure, driver crash). The failover is invisible from the API surface — the sidecar still returns embeddings, just slower. Without observation, latency degrades silently and operators don't know to investigate. The sev-2 alarm + adjusted SLO budget triggers operator action while preserving service availability.

**Why is this `SHOULD` (priority) and not `MUST`?** Embeddings have a managed-API fallback path (Cohere, OpenAI) that works correctly, just expensively. The cost target ($4/user/month) is achievable without self-hosted embeddings if we accept lower-margin operations OR fewer tenants. Self-hosted is the *economic* requirement for scaling, not a *correctness* requirement. FR priority `SHOULD` reflects this: ship managed first if you must (slice 4 might do this for the first 10 tenants), then transition to self-hosted as scale justifies. The FR is sized for "ship it now" but the gate is economic, not regulatory.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Rust adapter

```rust
// services/ai-gateway/src/router/bge_provider.rs

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct BgeProvider {
    sidecar_urls: Arc<HashMap<Region, String>>,    // §1 #14: per-region
    batch_buffer: Arc<bge_batch_buffer::BatchBuffer>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    pub tenant_id: String,
    pub task: EmbedTask,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbedTask { Passage, Code }

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,                 // each is 1024-dim
    pub model_name: String,                        // "bge-m3"
    pub model_sha256: String,                      // first 16 hex
    pub sidecar_version: String,
    pub device: String,                            // "cuda" | "cpu"
    pub elapsed_ms: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum BgeError {
    #[error("sidecar unreachable at {url}: {reason}")]
    Unreachable { url: String, reason: String },
    #[error("sidecar timeout (> {budget_ms}ms)")]
    Timeout { budget_ms: u32 },
    #[error("input too long: index={text_index} actual_tokens={actual} max={max}")]
    InputTooLong { text_index: usize, actual: u32, max: u32 },
    #[error("sidecar returned {status}: {body}")]
    SidecarError { status: u16, body: String },
    #[error("no sidecar configured for region {region:?}")]
    NoSidecarForRegion { region: Region },
}

#[async_trait]
impl Provider for BgeProvider {
    async fn call_embed(
        &self, req: &EmbedRequest, model: &str, region: &Region, deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        let url = self.sidecar_urls.get(region)
            .ok_or_else(|| RouterError::NoSidecarForRegion(*region))?;
        let response = self.batch_buffer.submit(req.clone(), url, deadline).await?;
        Ok(response)
    }

    fn cost_for(&self, _model: &str, _tokens: u32) -> f64 { 0.0 }   // §1 #6
}
```

### Adaptive batch buffer

```rust
// services/ai-gateway/src/router/bge_batch_buffer.rs

pub const BATCH_FAN_IN_TIMEOUT_MS: u64 = 50;
pub const BATCH_MAX_SIZE: usize = 32;

pub struct BatchBuffer {
    pending: Mutex<HashMap<String, VecDeque<PendingRequest>>>,   // keyed by sidecar URL
}

struct PendingRequest {
    req: EmbedRequest,
    deadline: Instant,
    response_tx: oneshot::Sender<Result<EmbedResponse, BgeError>>,
}

impl BatchBuffer {
    /// §1 #4 + §1 #5: timeout-or-32 fan-in with per-tenant fairness.
    pub async fn submit(&self, req: EmbedRequest, url: &str, deadline: Instant)
                        -> Result<EmbedResponse, BgeError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.entry(url.into()).or_default().push_back(PendingRequest {
                req, deadline, response_tx: tx,
            });
        }
        // Background dispatcher picks up and groups by URL with round-robin per tenant.
        rx.await.unwrap()
    }
}

/// Round-robin per-tenant ordering for fairness.
fn assemble_batch(queue: &mut VecDeque<PendingRequest>) -> Vec<PendingRequest> {
    let mut by_tenant: HashMap<String, VecDeque<PendingRequest>> = HashMap::new();
    while let Some(req) = queue.pop_front() {
        by_tenant.entry(req.req.tenant_id.clone()).or_default().push_back(req);
    }
    let mut batch = Vec::with_capacity(BATCH_MAX_SIZE);
    while batch.len() < BATCH_MAX_SIZE && !by_tenant.is_empty() {
        let tenants: Vec<String> = by_tenant.keys().cloned().collect();
        for t in tenants {
            if let Some(q) = by_tenant.get_mut(&t) {
                if let Some(req) = q.pop_front() { batch.push(req); }
                if q.is_empty() { by_tenant.remove(&t); }
                if batch.len() >= BATCH_MAX_SIZE { break; }
            }
        }
    }
    batch
}
```

### Python sidecar

```python
# services/ai-gateway/embeddings/sidecar/bge_m3_sidecar.py
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer
from .checksum import verify_model_checksum
from .health import HealthState
import time, torch, hashlib

CHECKSUM_PATH = "checksums/bge-m3.sha256"
MAX_TOKENS = 8192
SIDECAR_VERSION = "1.0.0"

app = FastAPI()
health = HealthState()

class EmbedRequest(BaseModel):
    texts: list[str]
    tenant_id: str
    task: str = "passage"

class EmbedResponse(BaseModel):
    embeddings: list[list[float]]
    model_name: str
    model_sha256: str
    sidecar_version: str
    device: str
    elapsed_ms: int

@app.on_event("startup")
async def startup():
    # §1 #11: checksum verification before model load.
    verify_model_checksum("BAAI/bge-m3", CHECKSUM_PATH)
    device = "cuda" if torch.cuda.is_available() else "cpu"
    if device == "cpu":
        # CPU fallback uses ONNX-quantised variant.
        app.state.model = SentenceTransformer("BAAI/bge-m3", device=device, backend="onnx")
    else:
        app.state.model = SentenceTransformer("BAAI/bge-m3", device=device)
    app.state.device = device
    app.state.model_sha256 = hashlib.sha256(open(CHECKSUM_PATH, "rb").read()).hexdigest()[:16]
    health.set_ready()

@app.post("/embed", response_model=EmbedResponse)
async def embed(req: EmbedRequest) -> EmbedResponse:
    if not health.is_ready():
        raise HTTPException(503, "sidecar warming up")

    # §1 #12: max-token validation per text.
    tokeniser = app.state.model.tokenizer
    for i, t in enumerate(req.texts):
        token_count = len(tokeniser.encode(t, add_special_tokens=False))
        if token_count > MAX_TOKENS:
            raise HTTPException(413, detail={
                "error": "input_too_long", "max_tokens": MAX_TOKENS,
                "actual_tokens": token_count, "text_index": i,
            })

    # §1 #9: code task uses prompt-template prefix.
    inputs = req.texts if req.task == "passage" else [f"Code: {t}" for t in req.texts]
    t0 = time.monotonic()
    embeddings = app.state.model.encode(
        inputs, batch_size=32, normalize_embeddings=True, convert_to_tensor=False,
    )
    elapsed = int((time.monotonic() - t0) * 1000)

    # §1 #15: report current device (catches mid-run GPU failover).
    current_device = "cuda" if torch.cuda.is_available() and app.state.device == "cuda" else "cpu"
    return EmbedResponse(
        embeddings=embeddings.tolist(),
        model_name="bge-m3", model_sha256=app.state.model_sha256,
        sidecar_version=SIDECAR_VERSION, device=current_device, elapsed_ms=elapsed,
    )

@app.get("/health")
async def health_endpoint():
    if not health.is_ready():
        raise HTTPException(503, "sidecar warming up")
    # Test embedding to catch silent model-load failures.
    try:
        _ = app.state.model.encode(["test"], batch_size=1)
        return {"status": "ok", "device": app.state.device,
                "sidecar_version": SIDECAR_VERSION}
    except Exception as e:
        raise HTTPException(503, detail=f"test embedding failed: {e}")
```

### Sidecar HTTP shape

```text
POST http://bge-sidecar-sg-1:5060/embed
Content-Type: application/json
{ "texts": ["query 1", "query 2"], "tenant_id": "org:cyberskill", "task": "passage" }

→ 200 OK
{
  "embeddings": [[0.012, ..., -0.034], [0.087, ..., 0.022]],
  "model_name": "bge-m3",
  "model_sha256": "4b8c0d2f1a7e9c3b",
  "sidecar_version": "1.0.0",
  "device": "cuda",
  "elapsed_ms": 28
}

→ 413 Payload Too Large
{ "error": "input_too_long", "max_tokens": 8192, "actual_tokens": 9214, "text_index": 1 }
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Single embedding returns 1024-dim** — `call_embed(["test"])` returns `embeddings[0].len() == 1024`.
2. **Batch of 32 returns 32 embeddings** — same shape; `embeddings.len() == 32`.
3. **GPU latency p95 ≤ 50ms** — 1000 batch-of-16 calls; `percentile(latencies, 0.95) <= 50`.
4. **CPU fallback latency p95 ≤ 300ms** — 1000 batch-of-16 calls on CPU sidecar; `<= 300`.
5. **Cost ledger reports zero** — `BgeProvider::cost_for("bge-m3", any_tokens) == 0.0`.
6. **Checksum mismatch refuses startup** — Inject a bad checksum file; sidecar fails to start with `ChecksumMismatch`; gateway refuses to bind.
7. **Health check returns 200 when ready** — After warmup, `GET /health` → 200 with `{"status":"ok","device":"...","sidecar_version":"..."}`.
8. **Health check returns 503 during warmup** — Within 5s of startup; HTTP 503.
9. **Input over 8192 tokens returns 413** — Send 9000-token text; HTTP 413 with body identifying `text_index` and `actual_tokens`.
10. **Adaptive batch buffer: 32 fast** — 32 concurrent calls within 1ms → 1 sidecar request; observed by mock-sidecar request count.
11. **Adaptive batch buffer: 5 with timeout** — 5 concurrent calls within 50ms window → 1 sidecar request after the 50ms timeout.
12. **Per-tenant fairness** — Tenant A submits 32 requests at t=0; Tenant B submits 1 request at t=10ms. Both A and B's requests appear in the same batch (B not starved); batch order is round-robin within tenants.
13. **Circuit breaker integrates** — 5 consecutive sidecar 5xx responses → breaker opens; subsequent calls fail fast with `BreakerOpen`; FR-AI-008 routing falls back to next provider.
14. **Per-region sidecar selection** — Request with `region=Eu1` selects `http://bge-sidecar-eu-1:5060`; request with `region=Sg1` selects `http://bge-sidecar-sg-1:5060`.
15. **No sidecar in required region returns ResidencyViolation** — Request with `region=Vn1` (no Vn1 sidecar deployed) → `RouterError::ResidencyViolation` (delegated to FR-AI-016).
16. **Mid-run GPU failover detected** — Sidecar reports `device: cuda` then `device: cpu` in two consecutive responses; gateway emits sev-2 OBS event `ai_bge_gpu_failed` once; metric `ai_bge_fallback_to_cpu_total` increments.
17. **Response carries model identity** — `EmbedResponse.model_name == "bge-m3"`; `model_sha256` is 16-hex; `sidecar_version` is semver; `device ∈ {"cuda", "cpu"}`.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/otel_test.rs
use cyberos_ai_gateway::router::bge_provider::{BgeProvider, EmbedRequest, EmbedTask};

#[tokio::test]
async fn single_embed_returns_1024_dim() {
    let bge = test_bge_provider();
    let req = EmbedRequest { texts: vec!["test".into()], tenant_id: "t".into(), task: EmbedTask::Passage };
    let resp = bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await.unwrap();
    assert_eq!(resp.embeddings.len(), 1);
    assert_eq!(resp.embeddings[0].len(), 1024);
    assert_eq!(resp.model_name, "bge-m3");
}

#[tokio::test]
async fn batch_of_32_returns_32() {
    let bge = test_bge_provider();
    let texts: Vec<String> = (0..32).map(|i| format!("text {i}")).collect();
    let req = EmbedRequest { texts, tenant_id: "t".into(), task: EmbedTask::Passage };
    let resp = bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await.unwrap();
    assert_eq!(resp.embeddings.len(), 32);
}

#[tokio::test]
#[ignore = "requires GPU sidecar; run with --ignored"]
async fn gpu_latency_p95_under_50ms() {
    let bge = test_bge_provider_gpu();
    let mut samples = vec![];
    for _ in 0..1000 {
        let t0 = std::time::Instant::now();
        let req = EmbedRequest {
            texts: (0..16).map(|i| format!("text {i}")).collect(),
            tenant_id: "t".into(), task: EmbedTask::Passage,
        };
        let _ = bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 <= 50, "GPU p95 = {p95}ms (budget 50)");
}

#[tokio::test]
async fn input_over_8192_tokens_returns_413() {
    let bge = test_bge_provider();
    let huge = "x ".repeat(10_000);   // ~10K tokens
    let req = EmbedRequest { texts: vec![huge], tenant_id: "t".into(), task: EmbedTask::Passage };
    let err = bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await.expect_err("expected 413");
    match err {
        RouterError::Bge(BgeError::InputTooLong { text_index: 0, actual, max }) => {
            assert_eq!(max, 8192); assert!(actual > 8192);
        }
        e => panic!("wrong variant: {e:?}"),
    }
}

#[tokio::test]
async fn cost_for_returns_zero() {
    let bge = test_bge_provider();
    assert_eq!(bge.cost_for("bge-m3", 1000), 0.0);
    assert_eq!(bge.cost_for("bge-m3", 1_000_000), 0.0);
}

#[tokio::test]
async fn no_sidecar_in_region_returns_residency_violation() {
    let bge = test_bge_provider_with_only_sg();
    let req = EmbedRequest { texts: vec!["test".into()], tenant_id: "t".into(), task: EmbedTask::Passage };
    let err = bge.call_embed(&req, "embed.standard", &Region::Vn1, deadline_in(5)).await.expect_err("expected residency");
    assert!(matches!(err, RouterError::Bge(BgeError::NoSidecarForRegion { .. })));
}
```

```rust
// services/ai-gateway/tests/cache_test.rs
#[tokio::test]
async fn 32_concurrent_calls_dispatch_in_one_batch() {
    let mock_sidecar = MockSidecar::start();
    let bge = test_bge_provider_with_url(&mock_sidecar.url());

    let mut joinset = tokio::task::JoinSet::new();
    for i in 0..32 {
        let bge = bge.clone();
        joinset.spawn(async move {
            let req = EmbedRequest {
                texts: vec![format!("text {i}")],
                tenant_id: format!("t{}", i % 4), task: EmbedTask::Passage,
            };
            bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await
        });
    }
    while let Some(r) = joinset.join_next().await { r.unwrap().unwrap(); }

    // AC #10: 32 concurrent → 1 sidecar call.
    assert_eq!(mock_sidecar.request_count(), 1);
    let body = mock_sidecar.last_request_body();
    assert_eq!(body["texts"].as_array().unwrap().len(), 32);
}

#[tokio::test]
async fn per_tenant_fairness_no_starvation() {
    let mock_sidecar = MockSidecar::start();
    let bge = test_bge_provider_with_url(&mock_sidecar.url());

    let mut joinset = tokio::task::JoinSet::new();
    // Tenant A floods 32 requests
    for i in 0..32 {
        let bge = bge.clone();
        joinset.spawn(async move {
            let req = EmbedRequest {
                texts: vec![format!("a{i}")], tenant_id: "tenant_a".into(), task: EmbedTask::Passage,
            };
            bge.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await
        });
    }
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    // Tenant B sneaks 1 in
    let bge_clone = bge.clone();
    let b_handle = tokio::spawn(async move {
        let req = EmbedRequest {
            texts: vec!["b".into()], tenant_id: "tenant_b".into(), task: EmbedTask::Passage,
        };
        bge_clone.call_embed(&req, "embed.standard", &Region::Sg1, deadline_in(5)).await
    });

    let _ = b_handle.await.unwrap();
    while let Some(r) = joinset.join_next().await { r.unwrap().unwrap(); }

    // First batch should contain a mix of tenant_a and tenant_b.
    let first_body = mock_sidecar.requests()[0].clone();
    let tenants_in_first: std::collections::HashSet<_> = first_body["texts"].as_array().unwrap().iter()
        .filter_map(|v| v.as_str().map(|s| if s.starts_with("a") { "tenant_a" } else { "tenant_b" }))
        .collect();
    assert!(tenants_in_first.contains("tenant_b"),
            "tenant_b starved out of first batch: {tenants_in_first:?}");
}
```

```bash
docker compose -f services/ai-gateway/embeddings/docker-compose.yml up -d
cd services/ai-gateway
cargo test bge
cargo test bge -- --ignored   # GPU latency tests (require L4)
```

---

## §6 — Implementation skeleton

See §3 for adapter, batch buffer, Python sidecar, HTTP shapes. Boot order:

```rust
// services/ai-gateway/src/lib.rs (additions)
pub async fn run() -> Result<(), Error> {
    // ... existing ...
    let bge_urls = embeddings_config::load_sidecar_urls("config/embeddings.yaml")?;
    let bge_provider = BgeProvider::new(bge_urls);
    bge_provider.health_check_all_sidecars().await?;   // §1 #8: refuse to bind if any unhealthy
    router::register_provider(ProviderKind::Bge, Arc::new(bge_provider));
    // ... bind HTTP ...
}
```

Docker compose:

```yaml
# services/ai-gateway/embeddings/docker-compose.yml
services:
  bge-sidecar-sg-1:
    build: { context: ., dockerfile: Dockerfile.gpu }
    ports: ["5060:5060"]
    deploy:
      resources:
        reservations: { devices: [{ driver: nvidia, count: 1, capabilities: [gpu] }] }
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5060/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: unless-stopped
  bge-sidecar-sg-1-cpu:
    build: { context: ., dockerfile: Dockerfile.cpu }
    ports: ["5061:5060"]
    healthcheck: { test: ["CMD", "curl", "-f", "http://localhost:5060/health"], interval: 10s }
    restart: unless-stopped
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-006** — alias map registers `embed.standard` and `embed.code` resolving to `BgeProvider`.
- **FR-AI-007** — cost_rates.yaml gets `bge-m3: { input: 0, output: 0 }` entry.
- **FR-AI-008** — router::Provider trait includes `call_embed`; BgeProvider is one variant.
- **FR-AI-009** — circuit breaker wraps BgeProvider (per-sidecar-instance state).
- **FR-AI-016** — region-pinned sidecar selection; absent sidecar in required region surfaces as residency violation.
- **FR-AI-020 (downstream)** — BGE rerank shares the sidecar infrastructure (different model checkpoint; same sidecar pattern).
- **FR-AI-022 (downstream)** — amortised infra cost dashboarding.

### Concept dependencies (shared types)

- `EmbedRequest`/`EmbedResponse` are the canonical embed-API shapes consumed by memory Layer 2.
- `EmbedTask::{Passage, Code}` is the prompt-template selector (passage = default; code = `"Code: "` prefix).
- `model_sha256` 16-hex is the embedding-version pin.

### Operational / external

- Python: `fastapi`, `uvicorn`, `sentence-transformers`, `torch` (GPU build for CUDA), `optimum` + `onnxruntime` (CPU build).
- Rust: `reqwest`, `tokio`, `async-trait`, `serde`.
- Hardware: NVIDIA L4 (or equivalent compute-capability 8.9+) for GPU sidecar; any x86_64 for CPU sidecar.
- Model artefact: `BAAI/bge-m3` from HuggingFace; checksum pinned at `embeddings/checksums/bge-m3.sha256`.

---

## §8 — Example payloads

See §3 for sidecar HTTP shapes. Cost-rate entry:

```yaml
# services/ai-gateway/config/cost_rates.yaml (additions)
bge:
  bge-m3:
    input: 0.0
    output: 0.0
    notes: "Self-hosted; marginal cost = 0. Amortised infra cost (~$360/mo per L4 GPU) tracked separately."
```

Cost-table lookup:

```rust
let rate = cost_table::lookup(&ProviderKind::Bge, "bge-m3").unwrap();
assert_eq!(rate.input, 0.0);
```

OBS event on GPU failover:

```text
sev-2  ai_bge_gpu_failed  sidecar_url=http://bge-sidecar-sg-1:5060
       previous_device=cuda  current_device=cpu  observed_at=2026-05-15T14:23:11Z
```

OBS event on checksum failure:

```text
sev-1  ai_bge_checksum_failed  expected=4b8c...  actual=9d6e...
       sidecar refusing to start; gateway refusing to bind
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Multi-GPU sidecar (use 4×L4 with tensor parallelism) — slice 5+ when scale demands.
- Model swap to BGE-M4 (when released) — handled by FR amendment + checksum bump + fixture refresh in memory Layer 2.
- Streaming embeddings (incremental embedding of long docs) — out of scope; chunking happens upstream.
- Per-tenant model variants (one tenant wants a fine-tuned BGE) — slice 5+; current sidecar serves one model per instance.
- Cross-region sidecar load-balancing during regional outage — FR-AI-016 area; current model is strict per-region pinning.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Sidecar process down | Health check 503 OR connection refused | Circuit breaker (FR-AI-009) opens; failover to managed (Bedrock Titan) | Operator restarts sidecar; breaker recovers via half-open probe |
| Sidecar checksum mismatch at startup | `verify_model_checksum` raises | Sidecar exits with non-zero; gateway refuses to bind; sev-1 alert | Operator investigates supply-chain integrity; re-downloads model |
| GPU OOM (input too long) | Pre-validation 413 (§1 #12) | Caller sees 413; chunks text upstream | By design |
| Mid-run GPU failure → CPU fallback | `device` field flips in response | sev-2 OBS event; SLO budget adjusts | Operator investigates GPU health; restarts sidecar to attempt cuda re-init |
| Cold start (first request 30s+) | Health endpoint 503 during warmup | Gateway waits up to 60s; refuses to bind if not ready | Self-resolves within 60s |
| CPU fallback latency > 300ms | `bge_test::cpu_latency_p95` fails OR live SLO breach | sev-3 alarm | Operator provisions GPU OR reduces batch size |
| Adaptive batch buffer 50ms timeout | Background dispatcher fires | Partial batch sent | By design |
| Per-tenant starvation (round-robin bug) | `per_tenant_fairness_no_starvation` test | PR blocked | Fix `assemble_batch` |
| No sidecar for required region | `bge_provider::call_embed` lookup miss | `RouterError::ResidencyViolation` cascades to FR-AI-016 | Operator deploys sidecar in region OR tenant accepts alternative |
| Sidecar OOM (concurrent batch overflow) | Sidecar 500; circuit breaker fires | Failover; alert | Operator scales sidecar OR caps concurrent batches |
| HuggingFace download fails at sidecar build | Dockerfile build fails | Image not produced; CI blocked | Operator investigates HuggingFace availability OR uses cached artefact |
| ONNX runtime mismatch (CPU sidecar) | Sidecar startup error | Sidecar fails; gateway falls back to managed | Pin `optimum`/`onnxruntime` versions in requirements.txt |
| Mock-sidecar test flaky | CI intermittent | Test rerun | Replace mock with deterministic stub |
| Output dimension drift (model-version mismatch) | `EmbedResponse.embeddings[0].len() != 1024` | sev-1 alert; downstream memory refuses | Operator confirms model checkpoint; re-pin checksum |
| Tokenizer mismatch (sidecar uses different tokeniser) | Token count for same text differs across sidecars | Inconsistent 413 behaviour | Pin tokeniser version in sidecar |
| Sidecar version drift (different sidecars in different regions) | `EmbedResponse.sidecar_version` mismatch across regions | sev-2 OBS event; consumers correlate | Operator harmonises deployments |
| Network partition between gateway and sidecar | reqwest timeout > deadline | `BgeError::Timeout`; circuit breaker counts | Standard network-failure recovery |
| Concurrent submit + breaker-open transition | Submit during transition window | Either breaker fast-fail OR successful (race-acceptable) | By design |
| memory Layer 2 stores embeddings; sidecar checksum changes (new model) | `model_sha256` in stored vectors differs from current sidecar's | Operator triggers re-embed pass | Standard schema-bump procedure |
| Tenant policy `embedding_model: "openai-text-embedding-3-small"` | Router does NOT route to BGE for that tenant | Managed provider used | By design (tenant choice) |

---

## §11 — Notes

- L4 GPU on AWS g6.xlarge at ~$0.50/hr × 720hr = $360/mo. Break-even vs managed (OpenAI text-embedding-3-small at $0.00002/1k tokens) at ~18M tokens/mo — well under a single active tenant's memory ingest. Two active tenants amortise the GPU within their first month.
- BGE-M3 multilingual support is the load-bearing reason for choosing it over English-only models. CyberSkill's home market is Vietnam; embedding quality on VN content is the primary KPI.
- ONNX-quantised CPU model is `bge-m3-onnx` (variant of the base model); 6× slower but fully compatible. Used for dev workflows and as a runtime fallback when GPU memory pressure forces CPU.
- The cost-ledger reports zero per-call (§1 #6) because the marginal cost IS zero. Amortised infra accounting is a SEPARATE ledger surfaced in FR-AI-022's dashboards. Conflating them would either over-charge per-call (if amortised) or under-report infra (if zeroed).
- Per-region sidecar deployment satisfies FR-AI-016 by construction. A `Vn1` tenant's embeddings cannot leave Vietnam if the sidecar is in `ap-southeast-1` because there's no Vn1 sidecar deployed (slice 4); the request fails with `ResidencyViolation` honestly rather than silently routing across borders.
- Model checksum verification (§1 #11) is the supply-chain defence. HuggingFace mirrors are widely used but not authoritative; pinning the SHA-256 ensures the running model is byte-identical to the audited version. Updates require simultaneous PR-reviewed model file + checksum file change.
- The 50ms batch buffer ceiling (§1 #4) is the user-visible-latency budget; the 32-text floor maximises GPU efficiency. Tuning is empirical: batch sizes < 8 underutilise the GPU; > 64 hit OOM on long inputs.
- Per-tenant fairness in the batch buffer (§1 #5) is the multi-tenancy correctness primitive. Without it, a single tenant's bursty workload can starve all other tenants — measurable as p99 latency spikes on the OBS dashboard.
- Mid-run GPU failover detection (§1 #15) is a small but valuable observability feature. PyTorch's silent GPU→CPU fallback is convenient for availability but invisible to operators; the device-field check makes it loud.
- memory Layer 2 (vector index) MUST record `model_sha256` alongside every stored embedding. A future FR (FR-MEMORY-014, placeholder) handles re-embedding when the sidecar's model_sha256 changes — current consumers should be defensive about this signal.

---

*End of FR-AI-019. Status: draft (10/10 target).*
