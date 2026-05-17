---
# ───── Machine-readable frontmatter (parsed by fr-audit + future fr-catalog renderer) ─────
id: FR-AI-020
title: "BGE-reranker-v2-m3 cross-encoder for KB reranking (per-region sidecar; CPU fallback)"
module: AI
priority: COULD
status: accepted
verify: T
phase: P0
milestone: P0 · slice 4
slice: 4
owner: Stephen Cheng
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-AI-001, FR-AI-002, FR-AI-006, FR-AI-008, FR-AI-009, FR-AI-016, FR-AI-019]
depends_on: [FR-AI-019]
blocks: [FR-KB-006]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#rerank
  - website/docs/modules/brain.html#kb-retrieval
source_decisions:
  - DEC-094 (rerank lift +15-25% precision-at-10 on multilingual KB benchmarks)
  - Cost ceiling: rerank-via-Cohere $0.001/search; at 10K searches/tenant/mo × 50 tenants = $500/mo
  - archive/2026-05-14/RESEARCH_REVIEW.md §6.4 (BGE-reranker-v2-m3 vs Cohere-rerank-3 on Vietnamese/English mix)

# ───── Build envelope ─────
language: rust 1.81 (adapter) + python 3.11 (sidecar)
service: cyberos/services/ai-gateway/embeddings/
new_files:
  - services/ai-gateway/embeddings/sidecar/bge_rerank_sidecar.py
  - services/ai-gateway/embeddings/sidecar/rerank_health.py
  - services/ai-gateway/embeddings/Dockerfile.rerank.gpu
  - services/ai-gateway/embeddings/Dockerfile.rerank.cpu
  - services/ai-gateway/embeddings/checksums/bge-reranker-v2-m3.sha256
  - services/ai-gateway/src/router/rerank_provider.rs
  - services/ai-gateway/src/router/rerank_batch_buffer.rs
  - services/ai-gateway/tests/rerank_test.rs
  - services/ai-gateway/tests/rerank_quality_test.rs                # known-relevance fixtures
  - services/ai-gateway/tests/rerank_fallback_test.rs
modified_files:
  - services/ai-gateway/src/router/mod.rs                         # add RerankProvider variant
  - services/ai-gateway/src/router/provider.rs                    # add call_rerank to trait
  - services/ai-gateway/config/cost_rates.yaml                    # bge-reranker-v2-m3 input=0 output=0
  - services/ai-gateway/config/embeddings.yaml                    # add rerank sidecar URLs per region
  - services/ai-gateway/src/brain_writer.rs                       # canonical::invocation_rerank builder
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,embeddings}/**
  - bash: docker compose -f services/ai-gateway/embeddings/docker-compose.yml up -d
  - bash: cargo test -p cyberos-ai-gateway rerank
disallowed_tools:
  - send rerank to managed APIs when bge-rerank sidecar is healthy (router prefers self-hosted per cost rule)
  - bypass tenant residency on rerank inference (per-region deployment per §1 #11; FR-AI-016 enforces)
  - skip cost-ledger emission on rerank calls (still emit ai.invocation row even with cost=0)
  - load the BGE-reranker model without checksum verification (per §1 #9)
  - silently truncate candidate lists > 100 (must return 413 instead per §1 #6)
  - emit cost > 0 in `ai.invocation` rows for BGE-rerank calls (marginal cost = 0; amortised infra cost tracked separately per FR-AI-019 §1 #6)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "1.0h: Python sidecar (FastAPI + FlagReranker for BAAI/bge-reranker-v2-m3)"
  - "0.5h: Model checksum verification at startup (mirrors FR-AI-019 §1 #11 pattern)"
  - "0.5h: GPU detection + CPU fallback variant"
  - "1.0h: Adapter (`RerankProvider`) implementing the `Provider` trait extension"
  - "0.5h: Per-region sidecar URL config (mirrors FR-AI-019 §1 #14)"
  - "1.0h: Batch buffer with per-tenant fairness (each call is one query × N candidates; batching across queries non-trivial — see §2)"
  - "0.5h: Token-budget validation (query + sum(candidate tokens) ≤ 8192 × 100; reject 413)"
  - "0.5h: Mid-run GPU failover detection (mirrors FR-AI-019 §1 #15)"
  - "0.5h: Score normalisation (sigmoid → [0,1]) AND raw-logit reporting (caller can choose)"
  - "0.5h: ai.invocation BRAIN row builder for rerank (canonical::invocation_rerank)"
  - "0.5h: Skipped-rerank signalling (when sidecar unavailable, KB caller MUST know rerank was skipped)"
  - "0.5h: Tests — single rerank, batch, quality (known-relevance fixtures), fallback, GPU/CPU latency, token-budget rejection, fairness"
  - "0.5h: OTel metrics + alarms"
risk_if_skipped: "KB retrieval relies on raw embedding similarity (cosine) without rerank lift. Top-10 KB results include embedding-similar-but-actually-irrelevant noise; auto-runbook quality (FR-OBS-007) and KB Q&A precision both degrade by ~15-25%. Mitigation: priority is COULD — KB module functions without rerank, just at lower precision. Descope-friendly under FR-AI-016 if BRAIN/KB ships behind schedule. Cost of NOT shipping: KB precision floor degrades; managed-rerank fallback (Cohere ~$0.001/search × 50 tenants × 10K searches/mo = $500/mo) becomes the operational alternative."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MAY** offer a BGE-reranker-v2-m3 cross-encoder service as a sidecar (sibling to FR-AI-019's embedding sidecar). The reranker and surrounding contract obey the following:

1. **MUST** be invokable via the `rerank.fast` alias from FR-AI-006. `RerankProvider` is registered as a `Provider` enum variant in FR-AI-008's router.
2. **MUST** accept a `(query, candidates)` request where `candidates` is a list of up to **100 documents** (hard cap per §1 #6). Returns scored pairs: `Vec<(original_index, score)>` sorted descending by score, with the original-index preservation so callers can map back to their source list.
3. **MUST** complete a batch of (query × 50 candidates) within **100ms p95** on GPU; CPU fallback budget widens to **600ms p95**. SLO assertions in `rerank_test.rs` with 1000-sample latency measurements.
4. **MUST** report `cost: 0.0` to the cost-ledger per call (self-hosted = no marginal cost; amortised infra tracked separately per FR-AI-019 §1 #6 pattern).
5. **MUST** emit one `ai.invocation` BRAIN audit row per call, via the `canonical::invocation_rerank` builder added to FR-AI-003's BRAIN bridge. Row payload: `tenant_id`, `agent_persona`, `model: "bge-reranker-v2-m3"`, `model_sha256`, `actual_usd: 0.0`, `latency_ms`, `query_token_count`, `candidate_count`, `total_token_count`, `device`, `request_id`. Mirrors FR-AI-002's invocation-row schema with rerank-specific fields.
6. **MUST** validate the candidate list size: more than 100 candidates returns HTTP 413 PAYLOAD_TOO_LARGE with body `{"error":"too_many_candidates","max":100,"actual":<n>}`. The KB module is responsible for pre-filtering to ≤ 100 before invoking rerank; this gate makes the contract explicit rather than silently truncating.
7. **MUST** validate the total token budget: `query_tokens + sum(candidate_tokens) <= 8192 × 100 = 819,200` (the cross-encoder's per-pair input is `(query, candidate)` with each pair limited to ~512 tokens; total budget is the practical limit). Over-budget → HTTP 413 with breakdown of `query_tokens`, `total_candidate_tokens`, `max_total`. Caller must trim or chunk.
8. **MUST** be deployed **per-region** matching the FR-AI-019 sidecar pattern. URL config lives in `services/ai-gateway/config/embeddings.yaml` under `bge_rerank_sidecars`. Per-region deployment satisfies FR-AI-016 residency pinning by construction; absence of a sidecar in the required region surfaces as `RouterError::NoSidecarForRegion` (handled by FR-AI-016 §1 #6).
9. **MUST** verify the BGE-reranker-v2-m3 model SHA-256 checksum at sidecar startup against `embeddings/checksums/bge-reranker-v2-m3.sha256`. Mismatch refuses startup; gateway refuses to bind. Same supply-chain defence rationale as FR-AI-019 §1 #11.
10. **MUST** report `model_name`, `model_sha256`, `sidecar_version`, `device` in every response. Downstream consumers (KB module) record these for retrieval-version pinning.
11. **MUST** detect mid-run GPU failure: if the sidecar's `device` field flips between consecutive responses, emit sev-2 OBS event `ai_rerank_gpu_failed` and adjust SLO alerting (600ms p95 instead of 100ms).
12. **MUST** signal "rerank skipped" to the caller when the sidecar is unavailable AND the circuit breaker is open. The handler returns a `RerankResponse` with `skipped: true` AND `scores: []` (empty); the KB caller then falls back to raw embedding-similarity ranking. Without this signal, the KB caller cannot distinguish "rerank ran and found nothing" from "rerank didn't run at all" — both look like empty scores.
13. **MUST** support both raw cross-encoder logit scores AND sigmoid-normalised `[0, 1]` scores. The request includes a `normalize: bool` field (default `true` for KB-friendly ranking). Both score types are deterministic given the same model and input.
14. **MUST** apply per-tenant fairness in the batch buffer (mirrors FR-AI-019 §1 #5): when multiple tenants have rerank queries pending, dispatch order is round-robin per tenant within each batch. A single tenant's flood cannot starve another tenant's request.
15. **SHOULD** support both monolingual and bilingual (Vi+En) rerank inputs. BGE-reranker-v2-m3 handles cross-lingual pairs natively; the request body's `query` and `candidates` can mix languages without explicit signalling.
16. **SHOULD** emit OTel metrics:
    - `ai_rerank_calls_total{tenant_id, candidate_bucket, device, outcome}` (counter; outcome ∈ ok | skipped | too_many | breaker_open).
    - `ai_rerank_latency_ms{device, candidate_bucket}` (histogram; SLO 100ms p95 GPU / 600ms p95 CPU).
    - `ai_rerank_candidates_per_call` (histogram).
    - `ai_rerank_total_tokens_per_call` (histogram; for FR-AI-022 cost-attribution dashboards).
    - `ai_rerank_skipped_total{tenant_id, reason}` (counter; reason ∈ breaker_open | sidecar_unreachable).
    - `ai_rerank_fallback_to_cpu_total{sidecar_url}` (counter; sev-2 alarm).
    - `ai_rerank_checksum_failed_total` (counter; sev-1).

---

## §2 — Why this design (rationale for humans)

**Why cross-encoder reranking adds value over pure embedding similarity?** Embedding similarity (cosine of two embedding vectors) is a one-shot independence assumption: query is embedded once; each candidate is embedded once; their similarity is computed at fixed dimensions. This produces approximate-good results — typically 60–70% precision-at-10 on multilingual benchmarks. Cross-encoder reranking JOINTLY processes (query, candidate) pairs through the model, attending to query-candidate interactions in both directions. The lift is consistent: +15-25% precision-at-10 on the BEIR multilingual subset. For KB Q&A, that's the difference between "the right answer is in the top 3" and "the right answer is in the top 10."

**Why BGE-reranker-v2-m3 specifically?** Three reasons aligning with FR-AI-019: (1) Multilingual — handles Vi+En and Vi-only equally well; English-only rerankers underperform on VN content. (2) MIT license — safe to self-host commercially. (3) State-of-art on MTEB Vietnamese rerank benchmarks for open-source models < 1B params. The alternatives (Cohere rerank-3, Voyage rerank-1) are paid APIs at ~$0.001 per search.

**Why the COULD priority?** KB module retrieves usably without rerank — just at lower precision. The user-visible degradation is "the answer is in position 5 instead of position 1" — annoying but not broken. If GPU capacity is constrained at slice 4 (FR-AI-019 saturates the L4), this FR can defer to slice 5 without breaking anything. The COULD priority makes the descope decision explicit and reversible.

**Why per-tenant fairness in the rerank batch buffer (§1 #14)?** Same reasoning as FR-AI-019 §1 #5: multi-tenant fairness is a SaaS correctness requirement. A tenant submitting 50 concurrent rerank queries (e.g., bulk ingestion path) must not starve another tenant's interactive query.

**Why batching rerank is harder than batching embeddings?** Embeddings: each text is independent; batch of 32 texts → 32 independent vector outputs. Rerank: each call is `(query, candidates)` where the query is fixed within the call. Batching across calls would require model-side support for "multiple queries × multiple candidates" — which the BGE reranker doesn't directly expose. The current design batches AT THE PAIR LEVEL inside a single call (32 (q,c) pairs computed simultaneously) but doesn't merge cross-call batches. Per-tenant fairness is achieved at the QUEUE level (which call dispatches first), not at the pair level.

**Why hard-cap at 100 candidates with 413 (§1 #6)?** Above 100 candidates, the latency budget breaks (~3s on GPU for query × 200 candidates) and the precision benefit plateaus (rerank-of-rerank-noise doesn't help). The KB module is expected to pre-filter using embedding similarity to ≤ 100 candidates BEFORE calling rerank; this is the established two-stage retrieval pattern. The 413 makes the contract explicit; silently truncating would hide the upstream bug (KB module didn't pre-filter properly).

**Why a token budget (§1 #7)?** Cross-encoder pairs are `(query, candidate)` strings concatenated and tokenised together; per-pair limit is ~512 tokens (BGE-reranker-v2-m3 max sequence). Total budget = `query_tokens + sum(candidate_tokens)` × pair-overhead. For a query of 50 tokens and 100 candidates of 200 tokens each, total = 50×100 + 100×200 = 25K tokens — well under the 819K budget. The budget exists to catch pathological inputs (10K-token candidates, 1K-token queries) before they trigger sidecar OOM.

**Why a `skipped: true` signal in the response (§1 #12)?** The KB module's caller code checks `if rerank_response.scores.is_empty() { fall_back_to_embedding() }`. Without the `skipped` flag, this conditional triggers in two distinct cases: (a) rerank ran and genuinely found no relevant candidates (scores all below a relevance floor — caller might want to refuse the query), and (b) rerank wasn't called because the sidecar is down (caller should fall back to embedding). The `skipped` flag distinguishes these — case (a) returns `skipped: false, scores: []`; case (b) returns `skipped: true, scores: []`.

**Why both raw logit AND sigmoid-normalised scores (§1 #13)?** Normalised scores `[0, 1]` are friendly for UI display and for "show me everything above 0.7" queries. Raw logits preserve more information at the upper end (the difference between 8.0 and 12.0 logit is meaningful; the difference between 0.999 and 0.99999 sigmoid is harder to interpret). KB Q&A workflows typically want normalised; observability and debugging want raw. Supporting both adds one bool field; restricting to one would force every consumer to convert if they wanted the other.

**Why marginal cost = 0 in cost-ledger (§1 #4)?** Same reasoning as FR-AI-019 §1 #6: the GPU is rented monthly; one additional rerank call costs $0 marginally. Amortised infra cost is tracked OUTSIDE the per-call cost ledger via FR-AI-022's accounting channel. This keeps cost-ledger semantics clean ("per-call dollars to provider") and avoids arbitrary amortisation choices.

**Why the audit row (§1 #5) reports `total_token_count` even though cost is zero?** Token counts are operational signals: the OBS dashboard tracks token volume per tenant for capacity planning. A tenant suddenly reranking 10x more tokens is a signal worth seeing (legitimate growth or a buggy ingestion loop). Recording the count even with zero cost preserves the operational visibility.

**Why is this FR a sibling sidecar rather than the same sidecar as FR-AI-019?** Architectural cleanliness: embedding and rerank are different model families with different memory footprints (BGE-M3 ~2GB; BGE-reranker-v2-m3 ~600MB). A single sidecar serving both would have larger startup time, larger memory pressure, and more complex health-check semantics. The two sidecars share an L4 GPU (the GPU has enough VRAM for both models) but run as independent processes. Operational independence (one can crash without affecting the other) is worth the small infrastructure overhead.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Rust adapter

```rust
// services/ai-gateway/src/router/rerank_provider.rs

use std::sync::Arc;
use std::time::Instant;

pub const MAX_CANDIDATES: usize = 100;
pub const MAX_TOTAL_TOKENS: u32 = 819_200;     // §1 #7

pub struct RerankProvider {
    sidecar_urls: Arc<HashMap<Region, String>>,
    batch_buffer: Arc<rerank_batch_buffer::BatchBuffer>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct RerankRequest {
    pub query: String,
    pub candidates: Vec<String>,
    pub tenant_id: String,
    #[serde(default = "default_normalize")]
    pub normalize: bool,
}

fn default_normalize() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
pub struct RerankResponse {
    pub scores: Vec<(usize, f32)>,             // (original index, score); sorted desc
    pub skipped: bool,                         // §1 #12: true ⇒ sidecar unavailable, KB falls back
    pub model_name: String,                    // "bge-reranker-v2-m3"
    pub model_sha256: String,
    pub sidecar_version: String,
    pub device: String,                        // "cuda" | "cpu"
    pub elapsed_ms: u32,
    pub query_token_count: u32,
    pub total_candidate_tokens: u32,
}

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
    #[error("no sidecar configured for region {region:?}")]
    NoSidecarForRegion { region: Region },
    #[error("breaker open for sidecar {url}")]
    BreakerOpen { url: String },
}

#[async_trait]
impl Provider for RerankProvider {
    async fn call_rerank(
        &self, req: &RerankRequest, region: &Region, deadline: Instant,
    ) -> Result<RerankResponse, RouterError> {
        // §1 #6: hard cap on candidates.
        if req.candidates.len() > MAX_CANDIDATES {
            return Err(RouterError::Rerank(RerankError::TooManyCandidates {
                max: MAX_CANDIDATES, actual: req.candidates.len(),
            }));
        }

        let url = self.sidecar_urls.get(region)
            .ok_or_else(|| RouterError::Rerank(RerankError::NoSidecarForRegion { region: *region }))?;

        // §1 #12: breaker-open path returns skipped=true response, NOT an error.
        if circuit_breaker::is_open(url) {
            metrics::skipped(&req.tenant_id, "breaker_open");
            return Ok(RerankResponse::skipped(req));
        }

        let response = self.batch_buffer.submit(req.clone(), url, deadline).await?;
        Ok(response)
    }

    fn cost_for_rerank(&self, _candidates: usize, _total_tokens: u32) -> f64 { 0.0 }   // §1 #4
}

impl RerankResponse {
    pub fn skipped(req: &RerankRequest) -> Self {
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
```

### Python sidecar

```python
# services/ai-gateway/embeddings/sidecar/bge_rerank_sidecar.py
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from FlagEmbedding import FlagReranker
from .checksum import verify_model_checksum
from .rerank_health import HealthState
import time, torch, hashlib

CHECKSUM_PATH = "checksums/bge-reranker-v2-m3.sha256"
MAX_CANDIDATES = 100
MAX_TOTAL_TOKENS = 819_200
SIDECAR_VERSION = "1.0.0"

app = FastAPI()
health = HealthState()

class RerankRequest(BaseModel):
    query: str
    candidates: list[str]
    tenant_id: str
    normalize: bool = True

class RerankResponse(BaseModel):
    scores: list[tuple[int, float]]
    skipped: bool = False
    model_name: str
    model_sha256: str
    sidecar_version: str
    device: str
    elapsed_ms: int
    query_token_count: int
    total_candidate_tokens: int

@app.on_event("startup")
async def startup():
    verify_model_checksum("BAAI/bge-reranker-v2-m3", CHECKSUM_PATH)
    use_fp16 = torch.cuda.is_available()
    app.state.reranker = FlagReranker("BAAI/bge-reranker-v2-m3", use_fp16=use_fp16)
    app.state.device = "cuda" if torch.cuda.is_available() else "cpu"
    app.state.model_sha256 = hashlib.sha256(open(CHECKSUM_PATH, "rb").read()).hexdigest()[:16]
    health.set_ready()

@app.post("/rerank", response_model=RerankResponse)
async def rerank(req: RerankRequest):
    if not health.is_ready():
        raise HTTPException(503, "sidecar warming up")

    # §1 #6: candidate count cap (defence in depth — Rust adapter checks too)
    if len(req.candidates) > MAX_CANDIDATES:
        raise HTTPException(413, detail={
            "error": "too_many_candidates", "max": MAX_CANDIDATES, "actual": len(req.candidates),
        })

    # §1 #7: token budget check.
    tokeniser = app.state.reranker.tokenizer
    query_tokens = len(tokeniser.encode(req.query, add_special_tokens=False))
    candidate_tokens = [len(tokeniser.encode(c, add_special_tokens=False)) for c in req.candidates]
    total_tokens = query_tokens + sum(candidate_tokens)
    if total_tokens > MAX_TOTAL_TOKENS:
        raise HTTPException(413, detail={
            "error": "token_budget_exceeded",
            "query_tokens": query_tokens, "total_candidate_tokens": sum(candidate_tokens),
            "max_total": MAX_TOTAL_TOKENS,
        })

    # Compute scores.
    pairs = [(req.query, c) for c in req.candidates]
    t0 = time.monotonic()
    scores = app.state.reranker.compute_score(pairs, normalize=req.normalize)
    elapsed = int((time.monotonic() - t0) * 1000)

    # Sort descending; preserve original indices.
    indexed = sorted(enumerate(scores), key=lambda x: -x[1])

    current_device = "cuda" if torch.cuda.is_available() and app.state.device == "cuda" else "cpu"
    return RerankResponse(
        scores=indexed, skipped=False,
        model_name="bge-reranker-v2-m3", model_sha256=app.state.model_sha256,
        sidecar_version=SIDECAR_VERSION, device=current_device, elapsed_ms=elapsed,
        query_token_count=query_tokens, total_candidate_tokens=sum(candidate_tokens),
    )

@app.get("/health")
async def health_endpoint():
    if not health.is_ready():
        raise HTTPException(503, "sidecar warming up")
    try:
        _ = app.state.reranker.compute_score([("test", "test")])
        return {"status": "ok", "device": app.state.device, "sidecar_version": SIDECAR_VERSION}
    except Exception as e:
        raise HTTPException(503, detail=f"test rerank failed: {e}")
```

### Sidecar HTTP shape

```text
POST http://bge-rerank-sg-1:5070/rerank
Content-Type: application/json
{
  "query": "Decree 13 personal data",
  "candidates": ["Decree 13 governs personal data...", "Article 7 PDPL says...", "..."],
  "tenant_id": "org:cyberskill",
  "normalize": true
}

→ 200 OK
{
  "scores": [[2, 0.94], [0, 0.71], [1, 0.12]],
  "skipped": false,
  "model_name": "bge-reranker-v2-m3",
  "model_sha256": "9f3a8d2b1e0c4f7a",
  "sidecar_version": "1.0.0",
  "device": "cuda",
  "elapsed_ms": 87,
  "query_token_count": 8,
  "total_candidate_tokens": 1240
}

→ 413 Payload Too Large (too many candidates)
{ "error": "too_many_candidates", "max": 100, "actual": 150 }

→ 413 Payload Too Large (token budget)
{ "error": "token_budget_exceeded", "query_tokens": 8, "total_candidate_tokens": 850000, "max_total": 819200 }
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Returns N scores for N candidates** — `RerankRequest` with 50 candidates returns `scores.len() == 50`.
2. **Scores descending** — `scores[0].1 >= scores[1].1 >= ... >= scores[N-1].1`.
3. **Indices preserved** — `scores[i].0 ∈ {0, ..., N-1}`; no duplicates; permutation of `0..N`.
4. **Quality: known relevance** — Test with a fixture (`rerank_quality_test.rs`) that has known-relevant + known-irrelevant docs; relevant docs MUST appear in the top-3 with score > 0.5 (when `normalize=true`).
5. **GPU latency p95 ≤ 100ms** — 1000 calls of `(query × 50 candidates)`; `percentile(latencies, 0.95) <= 100`.
6. **CPU fallback latency p95 ≤ 600ms** — 1000 calls; `<= 600`.
7. **Cost ledger reports zero** — `RerankProvider::cost_for_rerank(any, any) == 0.0`.
8. **Audit row emitted** — Every rerank call emits exactly one `ai.invocation` BRAIN row with rerank-specific fields populated.
9. **Too many candidates returns 413** — `RerankRequest` with 150 candidates → `RerankError::TooManyCandidates { max: 100, actual: 150 }`.
10. **Token budget exceeded returns 413** — `RerankRequest` with query × candidates totaling > 819,200 tokens → `RerankError::TokenBudgetExceeded`.
11. **Per-region sidecar selection** — Request with `region=Eu1` selects `http://bge-rerank-eu-1:5070`; absence of EU sidecar → `NoSidecarForRegion`.
12. **Skipped signal when breaker open** — Force-open the circuit breaker; subsequent `call_rerank` returns `RerankResponse { skipped: true, scores: [], .. }` (NOT an error).
13. **Skipped signal when sidecar unreachable** — Stop sidecar; circuit breaker eventually opens; subsequent calls return `skipped: true`.
14. **Normalize=true returns [0,1] scores** — All scores in `[0.0, 1.0]`.
15. **Normalize=false returns raw logits** — Scores can be > 1.0 or < 0.0; same ranking as normalized.
16. **Per-tenant fairness** — Tenant A submits 32 rerank requests; Tenant B sneaks 1 in mid-batch; both appear in early batches (B not starved).
17. **Mid-run GPU failover** — Sidecar reports `device: cuda` then `device: cpu`; gateway emits `ai_rerank_gpu_failed` sev-2; metric `ai_rerank_fallback_to_cpu_total` increments.
18. **Checksum mismatch refuses startup** — Bad checksum file → sidecar exits; gateway refuses to bind; metric `ai_rerank_checksum_failed_total` increments.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/rerank_test.rs
use cyberos_ai_gateway::router::rerank_provider::{RerankProvider, RerankRequest, RerankError};

#[tokio::test]
async fn returns_n_scores_descending() {
    let rp = test_rerank_provider();
    let candidates: Vec<String> = (0..50).map(|i| format!("doc {i}")).collect();
    let req = RerankRequest {
        query: "test query".into(), candidates, tenant_id: "t".into(), normalize: true,
    };
    let resp = rp.call_rerank(&req, &Region::Sg1, deadline_in(5)).await.unwrap();
    assert_eq!(resp.scores.len(), 50);
    for w in resp.scores.windows(2) { assert!(w[0].1 >= w[1].1); }
    let indices: HashSet<usize> = resp.scores.iter().map(|(i, _)| *i).collect();
    assert_eq!(indices.len(), 50);
}

#[tokio::test]
async fn too_many_candidates_returns_413() {
    let rp = test_rerank_provider();
    let candidates: Vec<String> = (0..150).map(|i| format!("doc {i}")).collect();
    let req = RerankRequest {
        query: "test".into(), candidates, tenant_id: "t".into(), normalize: true,
    };
    let err = rp.call_rerank(&req, &Region::Sg1, deadline_in(5)).await.expect_err("expected 413");
    assert!(matches!(err, RouterError::Rerank(RerankError::TooManyCandidates { max: 100, actual: 150 })));
}

#[tokio::test]
async fn token_budget_exceeded_returns_413() {
    let rp = test_rerank_provider();
    let huge: Vec<String> = (0..50).map(|_| "x ".repeat(10_000)).collect();   // ~500K tokens total
    let req = RerankRequest { query: "q".into(), candidates: huge, tenant_id: "t".into(), normalize: true };
    let err = rp.call_rerank(&req, &Region::Sg1, deadline_in(10)).await.expect_err("expected 413");
    assert!(matches!(err, RouterError::Rerank(RerankError::TokenBudgetExceeded { .. })));
}

#[tokio::test]
async fn breaker_open_returns_skipped_not_error() {
    circuit_breaker::force_open("http://bge-rerank-sg-1:5070");
    let rp = test_rerank_provider();
    let req = RerankRequest {
        query: "q".into(), candidates: vec!["c1".into(), "c2".into()],
        tenant_id: "t".into(), normalize: true,
    };
    let resp = rp.call_rerank(&req, &Region::Sg1, deadline_in(5)).await.unwrap();
    assert!(resp.skipped);
    assert!(resp.scores.is_empty());
    circuit_breaker::reset();
}

#[tokio::test]
async fn cost_for_rerank_returns_zero() {
    let rp = test_rerank_provider();
    assert_eq!(rp.cost_for_rerank(50, 5000), 0.0);
    assert_eq!(rp.cost_for_rerank(100, 800_000), 0.0);
}

#[tokio::test]
async fn audit_row_emitted_on_rerank() {
    let request_id = "req_test_rerank_001";
    let _ = handlers::rerank::handle(test_rerank_request(request_id)).await;
    let rows = brain_test_helper::find_rows("ai.invocation", request_id);
    assert_eq!(rows.len(), 1);
    let p = &rows[0].payload;
    assert_eq!(p["model"], "bge-reranker-v2-m3");
    assert_eq!(p["actual_usd"], 0.0);
    assert!(p["candidate_count"].as_u64().unwrap() > 0);
    assert!(p["total_token_count"].as_u64().unwrap() > 0);
}
```

```rust
// services/ai-gateway/tests/rerank_quality_test.rs
#[tokio::test]
async fn known_relevant_doc_in_top_3_with_score_above_0_5() {
    let rp = test_rerank_provider();
    let req = RerankRequest {
        query: "What does Decree 13 say about personal data?".into(),
        candidates: vec![
            "The weather in Hanoi is hot today.".into(),                                  // irrelevant
            "Decree 13/2023 establishes Vietnam's personal data protection framework.".into(),  // RELEVANT
            "Smart speakers are popular in 2026.".into(),                                 // irrelevant
            "Article 7 PDPL prohibits sale of personal data.".into(),                     // RELEVANT
            "Coffee prices rose 15% this quarter.".into(),                                // irrelevant
        ],
        tenant_id: "t".into(), normalize: true,
    };
    let resp = rp.call_rerank(&req, &Region::Sg1, deadline_in(5)).await.unwrap();
    let top_3_indices: Vec<usize> = resp.scores.iter().take(3).map(|(i, _)| *i).collect();
    assert!(top_3_indices.contains(&1) || top_3_indices.contains(&3),
            "neither relevant doc in top-3: {top_3_indices:?}");
    let top_score = resp.scores[0].1;
    assert!(top_score > 0.5, "top score {top_score} below 0.5 floor");
}

#[tokio::test]
async fn normalize_true_returns_zero_to_one() {
    let rp = test_rerank_provider();
    let req = RerankRequest {
        query: "test".into(), candidates: vec!["a".into(), "b".into(), "c".into()],
        tenant_id: "t".into(), normalize: true,
    };
    let resp = rp.call_rerank(&req, &Region::Sg1, deadline_in(5)).await.unwrap();
    for (_, score) in &resp.scores {
        assert!(*score >= 0.0 && *score <= 1.0, "score {score} outside [0, 1]");
    }
}

#[tokio::test]
async fn normalize_false_preserves_ranking_with_logits() {
    let rp = test_rerank_provider();
    let req_norm = RerankRequest {
        query: "test".into(), candidates: vec!["a".into(), "b".into(), "c".into()],
        tenant_id: "t".into(), normalize: true,
    };
    let mut req_raw = req_norm.clone();
    req_raw.normalize = false;
    let r_norm = rp.call_rerank(&req_norm, &Region::Sg1, deadline_in(5)).await.unwrap();
    let r_raw = rp.call_rerank(&req_raw, &Region::Sg1, deadline_in(5)).await.unwrap();
    let order_norm: Vec<usize> = r_norm.scores.iter().map(|(i, _)| *i).collect();
    let order_raw: Vec<usize> = r_raw.scores.iter().map(|(i, _)| *i).collect();
    assert_eq!(order_norm, order_raw, "ranking should be invariant to normalisation");
}
```

```bash
docker compose up -d bge-rerank-sg-1
cd services/ai-gateway
cargo test rerank
cargo test rerank -- --ignored   # GPU latency tests
```

---

## §6 — Implementation skeleton

See §3 for adapter, sidecar, HTTP shapes. Boot order:

```rust
// services/ai-gateway/src/lib.rs (additions)
pub async fn run() -> Result<(), Error> {
    // ... existing including FR-AI-019 ...
    let rerank_urls = embeddings_config::load_rerank_sidecar_urls("config/embeddings.yaml")?;
    let rerank_provider = RerankProvider::new(rerank_urls);
    rerank_provider.health_check_all_sidecars().await?;   // refuse to bind if any unhealthy
    router::register_provider(ProviderKind::Rerank, Arc::new(rerank_provider));
}
```

`canonical::invocation_rerank` builder:

```rust
pub mod canonical {
    pub fn invocation_rerank(
        req: &RerankRequest, resp: &RerankResponse, request_id: &str, agent_persona: &str,
    ) -> AuditRow {
        AuditRow {
            kind: "ai.invocation".into(),
            payload: serde_json::json!({
                "tenant_id": req.tenant_id,
                "agent_persona": agent_persona,
                "model": "bge-reranker-v2-m3",
                "model_sha256": resp.model_sha256,
                "actual_usd": 0.0,
                "latency_ms": resp.elapsed_ms,
                "candidate_count": req.candidates.len(),
                "query_token_count": resp.query_token_count,
                "total_token_count": resp.query_token_count + resp.total_candidate_tokens,
                "device": resp.device,
                "skipped": resp.skipped,
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-019** — Shares L4 GPU pod (sidecars are sibling Docker services). Pattern (per-region URL, checksum, health, batch buffer, fairness, GPU detection) is mirrored.
- **FR-AI-006** — alias map registers `rerank.fast` resolving to `RerankProvider`.
- **FR-AI-008** — router::Provider trait extended with `call_rerank`.
- **FR-AI-009** — circuit breaker wraps RerankProvider per-sidecar-instance.
- **FR-AI-016** — per-region deployment satisfies residency by construction.
- **FR-KB-007 (downstream placeholder)** — KB module calls rerank after embedding-similarity pre-filter.

### Concept dependencies (shared types)

- `RerankRequest`/`RerankResponse` are the canonical rerank-API shapes consumed by KB module.
- `MAX_CANDIDATES = 100` is the hard-cap primitive; KB pre-filters before calling.
- `MAX_TOTAL_TOKENS = 819_200` is the per-call token-budget primitive.
- `model_sha256` 16-hex is the rerank-version pin; KB records alongside ranked results.
- `skipped: bool` is the KB-fallback primitive.

### Operational / external

- Python: `fastapi`, `uvicorn`, `FlagEmbedding` (provides FlagReranker), `torch` (GPU build CUDA, CPU build standard).
- Rust: `reqwest`, `tokio`, `async-trait`, `serde`.
- Hardware: shared L4 GPU with FR-AI-019 (BGE-M3 ~2GB + BGE-reranker-v2-m3 ~600MB ≈ 2.6GB VRAM, well within L4's 24GB).
- Model artefact: `BAAI/bge-reranker-v2-m3` from HuggingFace; checksum pinned.

---

## §8 — Example payloads

See §3 for sidecar HTTP shapes.

### Cost-rate entry

```yaml
# services/ai-gateway/config/cost_rates.yaml (additions)
bge:
  bge-reranker-v2-m3:
    input: 0.0
    output: 0.0
    notes: "Self-hosted; marginal cost = 0. Amortised infra cost shared with bge-m3 (FR-AI-019)."
```

### Embeddings config (per-region)

```yaml
# services/ai-gateway/config/embeddings.yaml (additions)
bge_rerank_sidecars:
  - region: ap-southeast-1
    url: http://bge-rerank-sg-1:5070
  - region: eu-central-1
    url: http://bge-rerank-eu-1:5070
```

### Audit row `ai.invocation` for rerank

```json
{
  "kind": "ai.invocation",
  "ts_ns": 1747526400000000000,
  "payload": {
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "model": "bge-reranker-v2-m3",
    "model_sha256": "9f3a8d2b1e0c4f7a",
    "actual_usd": 0.0,
    "latency_ms": 87,
    "candidate_count": 50,
    "query_token_count": 8,
    "total_token_count": 1248,
    "device": "cuda",
    "skipped": false,
    "request_id": "req_01HZK..."
  }
}
```

### Audit row when skipped (sidecar down)

```json
{
  "kind": "ai.invocation",
  "payload": {
    "model": "bge-reranker-v2-m3",
    "skipped": true,
    "candidate_count": 50,
    "device": "unavailable",
    "actual_usd": 0.0,
    "request_id": "req_01HZK..."
  }
}
```

### KB caller code (consumer pattern)

```rust
// In KB module
let candidates = embedding_search(query, top_k=100).await?;
let resp = rerank_provider.call_rerank(&RerankRequest {
    query: query.into(),
    candidates: candidates.iter().map(|c| c.text.clone()).collect(),
    tenant_id: tenant.into(), normalize: true,
}, &region, deadline).await?;

let ranked: Vec<KbDoc> = if resp.skipped {
    candidates    // fall back to embedding similarity ordering
} else {
    resp.scores.iter().map(|(idx, _score)| candidates[*idx].clone()).collect()
};
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Cross-call batch merging (multiple `(query, candidates)` calls in one sidecar request) — slice 5+ if rerank throughput becomes the bottleneck.
- Per-tenant relevance threshold (some tenants want score > 0.7 floor; others want any-score-included) — KB module concern, not gateway.
- Rerank model variants (e.g., a finetuned BGE-reranker for specific domains) — out of scope; current model serves all tenants uniformly.
- Streaming top-K rerank (return scores as they're computed) — out of scope; latency tolerance is fine for batch-only.
- Multi-region rerank load-balancing — FR-AI-016 area; current model is strict per-region pinning.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Reranker sidecar down | Health check 503 OR connection refused | Circuit breaker opens; subsequent calls return `RerankResponse { skipped: true }`; KB falls back to embedding-similarity | Operator restarts sidecar; breaker recovers via half-open probe |
| Sidecar checksum mismatch at startup | `verify_model_checksum` raises | Sidecar exits non-zero; gateway refuses to bind; sev-1 alert | Operator investigates supply-chain; re-downloads model |
| Empty candidate list | Adapter check (length 0) | Returns `Ok(scores: [])` with `skipped: false` | By design (caller intent: "rank nothing returns nothing") |
| Too many candidates (>100) | Adapter pre-check + sidecar redundant check | `RerankError::TooManyCandidates` → 413; KB module pre-filters | KB module fixes its top-K parameter |
| Token budget exceeded | Sidecar pre-validation | 413 with token breakdown | KB module trims candidates or chunks |
| Mid-run GPU failure → CPU | `device` field flips between responses | sev-2 OBS event; SLO alerting adjusts to 600ms | Operator investigates GPU; restarts sidecar |
| GPU OOM on long candidates | Token budget catches before model invocation | 413 (not OOM) | By design |
| Cold start (first request 30s+) | Health endpoint 503 during warmup | Gateway waits up to 60s; refuses to bind if not ready | Self-resolves within 60s |
| Per-tenant starvation | `per_tenant_fairness` test | PR blocked | Fix `assemble_batch` round-robin |
| No sidecar in required region | URL lookup miss | `NoSidecarForRegion` cascades to FR-AI-016 | Operator deploys sidecar in region OR caller accepts fallback |
| Sidecar 500 mid-batch | reqwest non-2xx response | RerankError::SidecarError; circuit breaker counts | Self-resolves OR operator investigates |
| HuggingFace download fails at sidecar build | Dockerfile build fails | Image not produced; CI blocked | Operator investigates HuggingFace OR uses cached artefact |
| FlagEmbedding version mismatch | Sidecar startup error | Sidecar fails; gateway falls back | Pin FlagEmbedding version in requirements.txt |
| Score normalisation inconsistency | `normalize_true_returns_zero_to_one` test | PR blocked | Fix score conversion |
| KB caller doesn't check `skipped` field | Hard-to-detect; code review + linting | KB silently uses empty rerank scores | Code review enforces consumer pattern from §8 |
| Audit row missing rerank-specific fields | Integration test asserts field presence | Test fails → PR blocked | Add field to `canonical::invocation_rerank` |
| Sidecar version drift across regions | `EmbedResponse.sidecar_version` mismatch | sev-2 OBS event | Operator harmonises deployments |
| Tokeniser mismatch (sidecar vs adapter assumption) | Token count differs across regions | Inconsistent 413 behaviour | Pin tokeniser version |
| Concurrent submit + breaker open transition | Submit during transition | Either fast-fail with `BreakerOpen` OR success (race-acceptable) | By design |

---

## §11 — Notes

- Marked `COULD` priority — KB module retrieves usably without rerank (just at lower precision). Slice 4 ships rerank if GPU capacity allows; descope-friendly.
- Rerank latency dominates batch size; > 100 candidates degrades to multi-second territory. The 100-candidate cap (§1 #6) is the empirical sweet spot for the L4 GPU; KB module pre-filters using embedding similarity to ≤ 100 before invoking rerank. The two-stage retrieval pattern (embed-then-rerank) is the established best practice.
- BGE-reranker-v2-m3 is multilingual; cross-lingual pairs (Vi query against En candidates) work well — relevant for VN tenants whose KBs include English documentation.
- The `skipped: true` signal (§1 #12) is the KB-fallback contract. Without it, the caller cannot distinguish "rerank ran and found nothing relevant" from "rerank didn't run at all" — both look like empty scores. The signal converts ambiguity into clear caller logic.
- The shared L4 GPU between BGE-M3 (FR-AI-019) and BGE-reranker-v2-m3 saves ~$360/mo vs separate GPUs. Both models comfortably fit in 24GB VRAM (~2.6GB combined). Operational independence (sibling sidecars, not single sidecar) is preserved.
- Score normalisation (§1 #13) is a small but valuable feature. UI surfaces want `[0, 1]` (intuitive); observability surfaces want raw logits (more dynamic range at the upper end). Supporting both costs one bool field; restricting to one would force conversion in every consumer.
- The token budget (§1 #7) catches pathological inputs BEFORE they trigger sidecar OOM. Without it, a request with 100 × 10K-token candidates would crash the sidecar; with it, the call returns a clear 413 with the budget breakdown.
- The per-tenant fairness pattern (§1 #14) mirrors FR-AI-019 §1 #5. The two FRs share the multi-tenancy concern; consistent handling across embedding + rerank is operational hygiene.
- The amortised infra cost (~$360/mo for the L4) is shared between FR-AI-019 (embedding) and FR-AI-020 (rerank) — neither FR carries the full cost. FR-AI-022's accounting will surface this as a single `ai_infra_amortised_usd_per_day` metric, attributed to the GPU pool not the individual sidecar.
- Future expansion (slice 5+): per-domain finetuned rerankers (e.g., a legal-domain rerank tuned on Vietnamese legal corpora). The current FR ships the general-purpose reranker; domain variants would be additional sidecars with different model checksums.

---

*End of FR-AI-020. Status: draft (10/10 target).*
