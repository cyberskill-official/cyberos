---
task_id: TASK-AI-020
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10        # the first-pass compressed version (155 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-019 depth (~1010 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-020 was expanded from 155 lines to ~1010 lines matching TASK-AI-019 depth.

The expansion added 9 §1 normative clauses (#5 audit row builder, #6 hard-cap candidates with 413, #7 token budget validation, #8 per-region sidecar config, #9 model checksum, #10 model identity in response, #11 mid-run GPU failover detection, #12 `skipped: true` signal for caller-fallback distinction, #13 normalize bool for raw logit vs sigmoid scores, #14 per-tenant fairness in batch buffer), 9 substantive §2 rationale paragraphs (cross-encoder lift mechanism, model choice rationale, COULD priority frame, fairness anti-starvation, batch-difficulty-vs-embeddings, hard-cap-with-413 vs silent-truncate, token-budget OOM defence, skipped-signal caller-clarity argument, normalize-both-modes pragmatism, marginal-vs-amortised cost split, audit-row token-count operational visibility, sibling-sidecar architectural cleanliness), full Rust adapter + sidecar with token budget + checksum verification + skipped response shape, expanded §4 from 6 to 18 acceptance criteria, full Rust test bodies in §5 (returns N descending + indices preserved + 413 candidates + 413 token-budget + breaker-open returns skipped + cost zero + audit row + quality fixture + normalize true [0,1] + normalize-invariant ranking), full canonical builder + KB consumer-pattern example in §6, expanded §7 with code/concept/operational dep split, 5 example payloads in §8 (cost-rate, embeddings config, audit row, skipped audit row, KB caller code), 19 failure modes in §10 (vs. 3 in first pass), 10 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Inherits all TASK-AI-019 issues uncorrected (per-region URL, checksum, max-input, fairness, GPU failover)

- **severity:** error
- **rule_id:** consistency / cross-task pattern propagation
- **location:** §1 (multiple missing clauses), §3 (no per-region config)
- **status:** resolved

#### Description

TASK-AI-020 is the sibling sidecar to TASK-AI-019. The first-pass missed every operational concern that TASK-AI-019's audit had identified:

- Per-region sidecar URLs (TASK-AI-019 ISS-001) — first-pass had `localhost:5060` equivalent; would break TASK-AI-016 residency by construction.
- Model checksum verification (TASK-AI-019 ISS-003) — no startup integrity check; supply-chain attack surface.
- Max-input validation (TASK-AI-019 ISS-005 + here as token budget) — silent OOM on long candidates.
- Per-tenant fairness in batch buffer (TASK-AI-019 ISS-004) — single tenant could starve others.
- Mid-run GPU failover detection (TASK-AI-019 ISS-006) — silent latency degradation.
- Cost-accounting clarity (TASK-AI-019 ISS-002) — `cost: 0` claim without amortised-infra rationale.

A reader of TASK-AI-020 would build a sidecar replicating all the bugs TASK-AI-019's audit fixed.

#### Suggested fix

Mirror TASK-AI-019's normative clauses systematically:
- §1 #8 per-region URL config (with `bge_rerank_sidecars` key in embeddings.yaml).
- §1 #9 model SHA-256 checksum (with `embeddings/checksums/bge-reranker-v2-m3.sha256`).
- §1 #7 token budget (with HTTP 413 + breakdown).
- §1 #14 per-tenant fairness (mirror TASK-AI-019 §1 #5).
- §1 #11 mid-run GPU failover detection (mirror TASK-AI-019 §1 #15).
- §1 #4 explicit marginal-vs-amortised cost split (cite TASK-AI-019 §1 #6).

Each clause gets a corresponding AC, test body, and §10 row.

### ISS-002 — AC #2 "Scores descending; relevant docs first" not testable as written

- **severity:** error
- **rule_id:** test-coverage / acceptance-criteria specificity
- **location:** §4 AC #2
- **status:** resolved

#### Description

First-pass §4 AC #2 said: *"Scores are descending; relevant docs first."*

"Relevant" is undefined — relevant by whose judgement? The model's? Some external benchmark? A code-gen agent has no concrete criterion to test against. The "scores descending" part is testable (`scores[i] >= scores[i+1]`); the "relevant docs first" part is a quality claim without a fixture.

#### Suggested fix

1. Split AC #2 into two ACs:
   - AC #2: "Scores descending" — `scores[0].1 >= scores[1].1 >= ...` (mechanical assertion).
   - AC #3: "Indices preserved" — `scores[i].0 ∈ {0, ..., N-1}`; permutation of `0..N`.
2. Add AC #4 "Quality: known relevance" with concrete fixture in `rerank_quality_test.rs`:
   - 5 candidates: 3 irrelevant + 2 relevant (about Decree 13 / PDPL).
   - Relevant docs MUST appear in top-3.
   - Top score MUST be > 0.5 when normalized.
3. The fixture is the testable artefact; "relevance" is no longer subjective.

### ISS-003 — AC #6 "audit row emitted (1 per call)" but no canonical builder shown

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §1 #5 (claim), §3 (no builder), §6 (no builder)
- **status:** resolved

#### Description

The first-pass §1 #5 said: *"MUST emit `ai.invocation` memory row per call (same audit as any other inference)."* But the row's payload schema for rerank is different from a standard chat invocation — it carries `candidate_count`, `query_token_count`, `total_token_count`, `skipped`, `model_sha256` that aren't in TASK-AI-002's vanilla invocation row. No builder shown.

This is the same pattern as TASK-AI-014 ISS-004 (`canonical::persona_loaded` builder missing) and TASK-AI-015 ISS-003 (`canonical::zdr_violation` builder missing). The owning-task-builds-the-builder principle applies: this task owns the rerank-specific row variant.

#### Suggested fix

Add `canonical::invocation_rerank` builder in §3 + §6 with full payload schema. Add `services/ai-gateway/src/memory_writer.rs` to `modified_files`. Show the row's full JSON structure in §8 (both success and skipped variants).

### ISS-004 — Candidate truncation ("Truncate to top-100; warn log") contradicts validation discipline

- **severity:** error
- **rule_id:** consistency / silent-failure vs explicit-error
- **location:** §10 first-pass row, §1 #2 (limit mentioned)
- **status:** resolved

#### Description

The first-pass §10 had:

> *"Too many candidates (>100) | Adapter check | Truncate to top-100 by embedding score; warn log | Self-resolves"*

This contradicts the validation discipline established in TASK-AI-019 §1 #12 (over-length input returns 413, doesn't silently truncate). Silent truncation hides upstream bugs (KB module didn't pre-filter properly); 413 makes the caller fix the issue.

Also: "truncate to top-100 by embedding score" assumes the rerank adapter knows the embedding scores — it doesn't (the adapter only sees the candidate list, not their original embedding similarities). The truncation logic is undefined.

#### Suggested fix

Replace silent truncation with hard cap + 413:
1. §1 #6 normative requirement: > 100 candidates → 413 with body identifying max + actual.
2. `RerankError::TooManyCandidates { max, actual }` variant.
3. AC #9 asserting 413 response.
4. §5 test `too_many_candidates_returns_413`.
5. §10 row updated to "Adapter pre-check + sidecar redundant check → 413; KB module fixes its top-K parameter."
6. §11 note documenting that KB module's pre-filter is the boundary; rerank refuses to do upstream's job.

### ISS-005 — Token budget for cross-encoder pairs (query × N) not specified

- **severity:** warning
- **rule_id:** robustness / OOM defence
- **location:** §1 (no clause), §3 (no validation)
- **status:** resolved

#### Description

Cross-encoder rerank tokenises `(query, candidate)` pairs together. For N candidates, the total token count is roughly `N × (query_tokens + avg_candidate_tokens)`. With BGE-reranker-v2-m3's per-pair limit of ~512 tokens, the practical total is `100 × ~512 = ~50K tokens` worst-case for the candidate side, plus `100 × query_tokens` for the query side.

Pathological inputs (50-page candidate documents at 10K tokens each × 100 candidates = 1M tokens) would crash the sidecar with OOM. The first-pass had no token budget; only candidate count was capped.

#### Suggested fix

1. Add §1 #7 normative requirement: `query_tokens + sum(candidate_tokens) <= MAX_TOTAL_TOKENS = 819,200`.
2. Add `RerankError::TokenBudgetExceeded { q, c, m }` variant.
3. Sidecar pre-validates token count using the model's tokeniser.
4. AC #10 + §5 test `token_budget_exceeded_returns_413`.
5. §10 row + §11 note documenting that KB module trims candidates or chunks if budget exceeded.

### ISS-006 — Skipped-rerank fallback not signalled to caller

- **severity:** warning
- **rule_id:** API contract clarity
- **location:** §10 row "Failover skips rerank; raw embedding scores used", §1/§3 (no signal)
- **status:** resolved

#### Description

The first-pass §10 had:

> *"Reranker sidecar down | Health check | Failover skips rerank; raw embedding scores used | KB tolerates degraded precision."*

But the API contract (§3) doesn't expose any signal to the caller indicating "rerank was skipped." A `RerankResponse` with empty `scores` could mean two things:
(a) Rerank ran and found NO candidates above some threshold.
(b) Rerank wasn't called because the sidecar is down.

The caller (KB module) needs to distinguish: case (a) might warrant refusing the query (no relevant content); case (b) should fall back to embedding-similarity ordering. Without the signal, both cases trigger the same fallback path — losing information.

#### Suggested fix

1. Add §1 #12 normative requirement: `RerankResponse.skipped: bool`.
2. When circuit breaker is open OR sidecar unreachable, return `RerankResponse { skipped: true, scores: [], device: "unavailable", .. }` instead of an error.
3. Add `RerankResponse::skipped(req)` constructor in §3.
4. AC #12 + AC #13 asserting skipped=true on breaker-open and sidecar-unreachable.
5. §5 test `breaker_open_returns_skipped_not_error`.
6. §6 KB-consumer-pattern example showing the `if resp.skipped { fall_back() }` discipline.
7. §11 note explaining the caller-clarity rationale.

## §3 — Strengths preserved through expansion

- §3 mirrors TASK-AI-019's adapter pattern point-for-point — `RerankProvider` shape matches `BgeProvider`, errors are analogous, batch buffer follows the same per-tenant fairness pattern. Operational consistency across the two sibling sidecars makes both easier to maintain.
- §1 #5 introduces the rerank-specific audit row with `candidate_count` + `query_token_count` + `total_token_count` — preserving operational visibility (token volume per tenant) even though cost is zero.
- §1 #12 + the `skipped: true` signal makes the caller-fallback path unambiguous. KB module's `if resp.skipped { fall_back_to_embedding() }` pattern works correctly without guessing.
- §1 #13 supports both raw logit AND sigmoid-normalised scores via a single `normalize` bool — UI consumers get clean [0,1]; observability gets full dynamic range.
- §10 inventory grew from 3 rows to 19 — including the model-version-drift-across-regions path, the KB-caller-doesnt-check-skipped path, the token-budget-vs-OOM path, and the score-normalisation-inconsistency path. Each row has an unambiguous detection mechanism.
- §11 documents the COULD priority rationale (KB works without rerank) AND the shared-L4 economic rationale (one GPU serves both sidecars; ~$360/mo amortises across both tasks).

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the task itself:

- **ISS-001 RESOLVED**: Six new §1 clauses added mirroring TASK-AI-019 (per-region URL #8, token budget #7, fairness #14, checksum #9, model identity #10, GPU failover #11); cost-accounting clarity in #4 + §2; cross-task pattern propagated; corresponding ACs + §5 tests + §10 rows.

- **ISS-002 RESOLVED**: AC #2 split into mechanical assertions (descending + indices preserved); AC #4 introduces the known-relevance fixture in `rerank_quality_test.rs` with concrete docs (Decree 13 / PDPL); §5 test asserts top-3 contains relevant + top score > 0.5.

- **ISS-003 RESOLVED**: `canonical::invocation_rerank` builder in §3 + §6; `services/ai-gateway/src/memory_writer.rs` in `modified_files`; full row JSON in §8 (success + skipped variants); AC #8 asserts emission; §5 test `audit_row_emitted_on_rerank`.

- **ISS-004 RESOLVED**: Hard cap with 413 instead of silent truncation; §1 #6 normative; `RerankError::TooManyCandidates`; AC #9 + §5 test; §10 row updated; §11 note about KB-module-as-pre-filter boundary.

- **ISS-005 RESOLVED**: §1 #7 token budget normative; sidecar pre-validates; `RerankError::TokenBudgetExceeded`; AC #10 + §5 test `token_budget_exceeded_returns_413`; §10 row.

- **ISS-006 RESOLVED**: §1 #12 `skipped: true` signal; `RerankResponse::skipped(req)` constructor; circuit-breaker-open and sidecar-unreachable both return `skipped` response (not error); ACs #12 + #13; §5 test `breaker_open_returns_skipped_not_error`; §6 KB consumer-pattern example; §11 note.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-020 audit (final). Status: PASS at 10/10.*
