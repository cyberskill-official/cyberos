---
task_id: TASK-AI-019
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (220 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-014 / TASK-AI-017 depth (~990 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-019 was expanded from 220 lines to ~990 lines matching TASK-AI-014 / TASK-AI-017 depth.

The expansion added 7 §1 normative clauses (#5 per-tenant fairness in batch buffer, #11 model SHA-256 checksum verification at startup, #12 max-input-length validation with 413 response, #13 model identity reporting, #14 per-region sidecar URL config, #15 mid-run GPU failover detection, #16 expanded OTel metric set), 6 substantive §2 rationale paragraphs (per-tenant fairness anti-starvation argument, marginal-vs-amortised cost separation, supply-chain defence frame, model-identity-as-version-pin lock-step principle, per-region deployment satisfies TASK-AI-016 by construction, GPU-failover-invisible-without-observation argument, SHOULD-priority economic-vs-correctness frame), full Rust adapter + adaptive batch buffer with per-tenant round-robin in §3, full Python sidecar with checksum verification + max-token validation + device reporting, expanded §4 from 9 to 17 acceptance criteria, full Rust test bodies in §5 (single embed + batch + GPU latency proptest + 413 + cost zero + no-sidecar-region + 32-concurrent-batches-into-one + per-tenant-fairness anti-starvation), Docker compose with healthcheck in §6, expanded §7 with code/concept/operational dep split, expanded §8 with cost-rate YAML + OBS event examples, 19 failure modes in §10 (vs. 5 in first pass), 10 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Sidecar URL hardcoded `http://localhost:5060` with no service discovery or per-region routing

- **severity:** error
- **rule_id:** spec-completeness / cross-task coupling
- **location:** §3 sidecar URL, §1 (no per-region clause)
- **status:** resolved

#### Description

The first-pass §3 used `http://localhost:5060` as the sidecar URL. This is fine for a single-instance dev deployment but breaks two task-coupling invariants:

1. **TASK-AI-016 (residency pinning)**: An `Eu1` tenant's embedding request can't be served by an `ap-southeast-1` sidecar — the data crosses the residency border. The first-pass task doesn't acknowledge per-region deployment.
2. **Service discovery**: Production deployments use container DNS (`bge-sidecar-sg-1:5060`) or service-mesh resolution. Hardcoded `localhost` is wrong outside dev.

A code-gen agent reading the task would build the localhost-only adapter; the production deployment would have to retrofit per-region routing.

#### Suggested fix

Add §1 #14 normative requirement: read sidecar URLs from `embeddings.yaml` keyed by region. Show the YAML format. Update §3 adapter to take `Arc<HashMap<Region, String>>`. Add `BgeError::NoSidecarForRegion` variant. Add AC #14 (per-region selection) and AC #15 (no-sidecar-in-region surfaces as residency violation). Add §2 rationale paragraph on TASK-AI-016 satisfaction-by-construction.

### ISS-002 — `cost: 0` claim doesn't acknowledge amortised infra cost (~$360/mo)

- **severity:** error
- **rule_id:** spec-completeness / cost-accounting clarity
- **location:** §1 #4 (claim "report cost: 0.0"), §2 (no rationale)
- **status:** resolved

#### Description

The first-pass §1 #4 said: *"MUST report `cost: 0.0` to the cost-ledger (self-hosted = no marginal cost)."*

This is correct for the marginal cost (one additional BGE call costs $0 because the GPU is rented monthly). But it's misleading for total cost-of-service: the ~$360/mo GPU rental is a real expense. A finance audit asking "show me embeddings cost for tenant X" sees $0 across all tenants — but the company spent $4320/year on infrastructure.

Without explicit handling, a future engineer might:
(a) Try to amortise the cost across calls (arbitrary choice — by-tenant? by-token?).
(b) Try to amortise it into the per-call cost-ledger semantics (breaks the "per-call dollars to provider" definition).
(c) Just leave it invisible (current state).

#### Suggested fix

1. Make §1 #6 explicit about the marginal-vs-amortised split: cost-ledger reports marginal $0; amortised infra cost tracked OUTSIDE the per-call ledger.
2. Add §2 rationale paragraph explaining why conflation is wrong (arbitrary amortisation choice corrupts cost-ledger semantics).
3. Reference TASK-AI-022 as the location where amortised infra accounting will surface.
4. Add a notes field in cost_rates.yaml making the zero explicit + citing the amortisation venue.
5. Document the operational accounting expectation in §11.

### ISS-003 — Model checksum not verified at startup; corrupted/swapped model loads silently

- **severity:** error
- **rule_id:** security / supply-chain
- **location:** §1 (no checksum clause), §3/§6 (no verification)
- **status:** resolved

#### Description

The first-pass loaded the model directly via `SentenceTransformer("BAAI/bge-m3")` — pulling from HuggingFace. No verification that the loaded model matches an audited version.

Supply-chain attack surface: a compromised HuggingFace mirror could swap a model for a maliciously-tuned variant (e.g., one that produces consistently-biased embeddings steering retrieval to attacker-favoured content). The result is invisible to operators — the model loads, embeddings come back, retrieval looks plausible.

This is a real risk: HuggingFace has had model-tampering incidents in 2024.

#### Suggested fix

1. Add §1 #11 normative requirement: SHA-256 checksum verification at sidecar startup against a pinned `embeddings/checksums/bge-m3.sha256` file.
2. Add `checksum.py` to `new_files`.
3. Add `verify_model_checksum` call in the sidecar's `@app.on_event("startup")`.
4. Mismatch → sidecar refuses to start; gateway refuses to bind.
5. Add AC #6 asserting checksum-mismatch refuses startup.
6. Add §10 row + §11 note documenting the supply-chain defence.
7. Document model-update process in §11: PR replaces both model file AND checksum file together.

### ISS-004 — Per-tenant fairness in batch buffer not specified; one tenant can starve others

- **severity:** error
- **rule_id:** correctness / multi-tenancy fairness
- **location:** §1 #3 (batch buffer mentioned), §3/§6 (no fairness logic)
- **status:** resolved

#### Description

The first-pass said *"batch up to 32 concurrent calls for 50ms then sends a batch (or sends earlier if 32 reached)."* No mention of dispatch order within the batch.

Failure mode: Tenant A submits 100 concurrent embed requests at t=0 (every 1ms). At t=10ms, Tenant B submits 1 request. The naive FIFO batch buffer fills the first 32 slots with Tenant A's requests; Tenant B's request waits in the queue behind 50+ A-requests. Tenant B observes ~1500ms latency for what should be a 50ms call.

In a multi-tenant SaaS, this is unacceptable — single noisy tenants degrade UX for everyone.

#### Suggested fix

1. Add §1 #5 normative requirement: per-tenant round-robin dispatch within each batch.
2. Show `assemble_batch` skeleton in §3 with the round-robin algorithm.
3. Add AC #12 asserting fairness (Tenant A flood + Tenant B sneak-in → both in same batch).
4. Add §5 test `per_tenant_fairness_no_starvation`.
5. Add §2 rationale paragraph on the cost (slightly less efficient batching) being worth the fairness guarantee.

### ISS-005 — Max input length not validated; texts > 8192 tokens silently truncated

- **severity:** warning
- **rule_id:** correctness / silent failure
- **location:** §1 (no length clause), §3 sidecar (no validation)
- **status:** resolved

#### Description

BGE-M3's max sequence length is 8192 tokens. The `sentence-transformers` library SILENTLY truncates inputs longer than this — producing embeddings for "the first 8192 tokens of your text" rather than "your text."

A user embedding a 50-page document would get a partial-document vector with no error indication. The downstream memory Layer 2 retrieval would silently return based on the truncated portion. The retrieval failure is then attributed to "model quality" rather than "input was too long."

#### Suggested fix

1. Add §1 #12 normative requirement: validate token count BEFORE embedding; over-length → HTTP 413.
2. Show validation in the Python sidecar in §3.
3. Add `BgeError::InputTooLong { text_index, actual, max }` variant in Rust adapter.
4. Add AC #9 asserting 413 response with index identification.
5. Add §5 test `input_over_8192_tokens_returns_413`.
6. Add §10 row + §11 note for chunking-upstream guidance.

### ISS-006 — GPU detection at startup only; mid-run GPU failure invisible

- **severity:** warning
- **rule_id:** observability / robustness
- **location:** §1 #1 (startup detection), §3 sidecar
- **status:** resolved

#### Description

The first-pass detected GPU at `@app.on_event("startup")` — `device = "cuda" if torch.cuda.is_available() else "cpu"`. Once set, the sidecar uses that device for the entire process lifetime.

But PyTorch can fall back to CPU at runtime (driver crash, GPU memory pressure, kernel unavailable). The fallback is invisible from the API: embeddings still come back, just slower. Latency degrades from ~50ms to ~300ms; nobody knows to investigate until the SLO breach is observed downstream.

#### Suggested fix

1. Add §1 #15 normative requirement: report `device` field in EVERY response (not just at startup); detect mid-run cuda→cpu transitions.
2. Update sidecar to report current device (re-checking `torch.cuda.is_available()` per response is cheap).
3. Add gateway-side detection: if `device` flips between consecutive responses, emit sev-2 OBS event.
4. Add metric `ai_bge_fallback_to_cpu_total` with sev-2 alarm on increment.
5. Add AC #16 asserting the failover detection.
6. Adjust SLO budget alerting (300ms p95 instead of 50ms when on CPU).
7. Add §10 row + §11 note on GPU observability.

## §3 — Strengths preserved through expansion

- §3 introduces the `Region`-keyed sidecar URL map — making the per-region deployment a typed primitive, not an ops convention.
- §1 #6 + §2 cleanly separate marginal cost (per-call $0) from amortised infra cost (tracked elsewhere). Future engineers reading the spec understand WHY the cost ledger reports zero rather than wondering if it's a bug.
- §1 #11 + checksum file is the supply-chain defence; the PR-discipline (model file + checksum file change in one PR) makes tampering a multi-step attack instead of a one-step.
- §1 #5 + per-tenant round-robin dispatch is the multi-tenancy fairness primitive. Without it, the cache + breaker + retry chain can produce per-tenant tail-latency that's invisible to aggregate metrics.
- §1 #12 + 413 response converts "silently wrong embedding" into "loud HTTP error." The downstream memory consumer can choose to chunk OR reject; it doesn't unknowingly retrieve from a truncated vector.
- §1 #15 + GPU-failover detection is the observability primitive that converts "silent latency degradation" into "operator-actionable sev-2 event." Standard PyTorch behaviour is invisible; we make it visible.
- §10 inventory grew from 5 rows to 19 — including the checksum-mismatch path, the per-tenant-starvation path, the model-version drift between regions path, the memory-stored-vector-version-mismatch path, and the tokenizer-mismatch path. Each row has an unambiguous detection mechanism.
- §11 documents the L4 break-even economics ($360/mo amortises within 2 active tenants), the BGE-M3-vs-alternatives evaluation rationale (multilingual MTEB scores, MIT license, dimension grain), and the slice-4 deployment scope (ONNX CPU fallback for dev workflows).

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the task itself:

- **ISS-001 RESOLVED**: §1 #14 added with `embeddings.yaml` per-region URL config; §3 adapter takes `Arc<HashMap<Region, String>>`; `BgeError::NoSidecarForRegion` variant; ACs #14 + #15 added; §5 test `no_sidecar_in_region_returns_residency_violation`; §2 rationale paragraph.

- **ISS-002 RESOLVED**: §1 #6 explicit about marginal-vs-amortised split; §2 rationale paragraph; cost_rates.yaml entry includes `notes` field documenting the amortisation; §11 note explicitly references TASK-AI-022 as the venue for amortised tracking.

- **ISS-003 RESOLVED**: §1 #11 SHA-256 checksum verification requirement; `checksum.py` in `new_files`; sidecar startup performs verification; AC #6 asserts mismatch refuses startup; §10 row for `ai_bge_checksum_failed_total` sev-1; §11 note on PR-discipline (model + checksum change in one PR).

- **ISS-004 RESOLVED**: §1 #5 per-tenant fairness requirement; `assemble_batch` round-robin algorithm in §3; AC #12 + §5 `per_tenant_fairness_no_starvation` test; §2 rationale paragraph on the cost-vs-fairness trade-off.

- **ISS-005 RESOLVED**: §1 #12 max-input-length validation; sidecar pre-validates token count using model tokeniser; HTTP 413 response with `text_index` + `actual_tokens` + `max_tokens`; `BgeError::InputTooLong` variant; AC #9 + §5 `input_over_8192_tokens_returns_413` test; §10 row.

- **ISS-006 RESOLVED**: §1 #15 mid-run GPU failover detection; sidecar reports current `device` per response; gateway-side detection of cuda→cpu transitions emits sev-2 OBS event `ai_bge_gpu_failed`; metric `ai_bge_fallback_to_cpu_total`; AC #16 + §10 row + §11 note.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-019 audit (final). Status: PASS at 10/10.*
