# Phase 1 tasks - a brain that understands content

Gate to enter: MEM-001..003 in_review or done. Spine of the phase: MEM-012 -> MEM-013 -> MEM-014 (facts, ops, content), with MEM-015 (PII) and MEM-025 (emitters + consent) as the other critical path. Ranking work (MEM-019..023) can proceed in parallel once MEM-012 lands.

---

## MEM-012 - brain_fact table + memory_kind taxonomy

refs R3, R4 | est 10h | deps MEM-002, MEM-003 | priority critical

Why: events and kind-frequency summaries cannot express durable knowledge; every serious memory system retrieves facts.

Files: new `migrations/0011_brain_fact.sql`, new `src/brain/fact.rs`, `src/brain/mod.rs`.

Steps:
1. Migration: `brain_fact(id uuid pk, tenant_id, subject_id, kind CHECK IN ('semantic','procedural','profile','resource'), body text, embedding halfvec(1024), embed_model_version, importance real default 0.5, trust real default 0.5, source_kind CHECK IN ('user','agent','tool','external'), access_count bigint default 0, last_accessed_at timestamptz, valid_at timestamptz not null, invalid_at timestamptz, revision int default 1, previous_id uuid, derived_from jsonb not null, pii_flags text[], quarantined bool default true, created_at, updated_at)`; partial HNSW on current rows (`invalid_at IS NULL`); FTS expression index on body; RLS fail-closed (MEM-002 form); tenant-leading composite indexes.
2. Rust model + CRUD helpers inside `tenant_tx`; `derived_from` always carries at least one `audit_row_id`.
3. Wire fact rows into recall as a third retriever stub returning empty until MEM-013 populates (keeps the diff reviewable).

Accept: migration applies clean on dev + fresh DB; RLS probe covers the table; model round-trips.

Tests: schema test, RLS test, CRUD round-trip with provenance assertions.

Review (human): the taxonomy and defaults (quarantined=true, trust=0.5) are policy; confirm they match the governance stance.

---

## MEM-013 - Fact write pipeline (extract, then ADD/UPDATE/DELETE/NOOP)

refs R5 | est 14h | deps MEM-012, MEM-007 | priority critical

Files: new `src/brain/fact_ops.rs`, `src/brain/ingest_worker.rs` (hook), gateway chat client reuse, prompts under `services/memory/prompts/`.

Steps:
1. Extraction: for each ingested event batch (and later content, MEM-014), call the gateway chat route with a pinned prompt returning candidate facts as JSON (schema-validated; reject on parse failure, count metric).
2. Update decision: per candidate, k-NN top-5 existing facts (same subject scope), one gateway tool-call returning `{op: add|update|delete|noop, target_id?, new_body?}`; apply inside a tenant tx.
3. UPDATE/DELETE are soft: new revision row with `previous_id`, or `invalid_at` set; nothing hard-deleted; every applied op emits a chained audit row (`memory.fact_op` event kind through the existing emit path, system actor).
4. Budget: batch prompts, cap ops per event, spend through the gateway only; on 402/down mark the event `pending_fact_extract` (new state column or side table) and retry like embeds.
5. Replays idempotent: op application keyed by (source event seq, candidate hash).

Accept: seeded conversation produces sensible facts with provenance; re-running the batch produces zero duplicate ops; all ops visible on the chain.

Tests: stubbed-gateway unit tests per op type; idempotency replay test; malformed-JSON rejection test; integration round-trip fact -> recall.

Review (human): read the extraction + op prompts (they define what the company remembers); check op audit rows render usefully in the console viewer.

---

## MEM-014 - Content-aware ingestion behind consent + flag

refs R6, F2 | est 12h | deps MEM-013, MEM-015 | priority critical

Files: `src/brain/ingest_worker.rs`, new `src/brain/content_fetch.rs`, config.

Steps:
1. For events whose `content_ref` is `Pointer{store,id}` with store in the contract list (`chat_messages | proj_documents | email_objects | memory | attachments`), fetch the content via the owning store using a scoped read (service credential honoring that store's RLS; start with chat_messages only).
2. Pipe fetched text through the PII sidecar (MEM-015), then embed the redacted text for the event row (replacing envelope-only embedding for these kinds) and feed it to fact extraction (MEM-013).
3. Store nothing but the vector + pointer + pii_flags; never copy the raw body into brain tables (DEC-2701 intact).
4. Consent: only for subjects the gate allows (it already gated emit; re-check at ingest for defense in depth). Rollout flag `BRAIN_CONTENT_INGEST=chat_messages,...` default empty.
5. Backfill entry: extend `backfill.rs` with `--reembed-content` for already-ingested pointer events.

Accept: with the flag on in dev, a chat message becomes semantically recallable ("what did X say about Y") via event + fact hits; raw body absent from all brain tables (test greps).

Tests: integration with seeded chat store; PII redaction asserted (seed a phone number, assert vector text redacted via sidecar mock and pii_flags set); flag-off no-op test.

Review (human): this is the privacy-posture change of the whole program; verify flag default off in prod deploy, consent gate real (MEM-025) before enabling, and the store-scoped credential is read-only.

---

## MEM-015 - PII sidecar (Presidio + VN recognizers)

refs R78, F20 | est 14h | deps MEM-007 | priority critical

Files: new `services/pii-sidecar/` (Python, presidio-analyzer/anonymizer, mirrors embed-sidecar packaging), Rust client `src/brain/pii_client.rs`, labeled eval set under `services/pii-sidecar/eval/`, dev compose.

Steps:
1. Sidecar: `POST /v1/redact {text, lang} -> {redacted_text, entities[{type,offset,len}], flags[]}`; recognizers: Presidio EN defaults + custom VN (CCCD/CMND ids, VN phone formats, VN addresses, tax codes per FR-MEMORY-111); deterministic replacement tokens.
2. Labeled eval set (>=300 snippets EN+VN, half from `modules/memory` test corpus style); CI job scoring recall; target >=99.5% on the held-back split per FR-MEMORY-111.
3. Rust client with timeout + fail-closed policy: if the sidecar is down, content-aware ingestion marks the event pending rather than embedding unredacted text.
4. Port lessons from `modules/memory/cyberos/core/pre_ingest_pii.py`; leave the Python file-store path as is for now.

Accept: eval meets the FR bar; ingest fails closed without the sidecar; flags recorded per row.

Tests: sidecar pytest with the eval set; Rust client contract test; fail-closed integration test.

Review (human): spot-check 20 VN redactions manually (recognizer quality is a judgment call); confirm the sidecar has no egress (residency).

---

## MEM-016 - Abstractive summaries via gateway chat

refs R25, F3 | est 8h | deps MEM-007 | priority high

Files: `src/brain/summarize.rs`, prompt under `services/memory/prompts/`, config flag `BRAIN_SUMMARY_MODE=abstractive|extractive`.

Steps: build the digest from event kinds + (when MEM-014 is live) redacted content excerpts of the window's top contributors; prompt demands topics, decisions, open questions, each with `[l1:...]` citations, and forbids secrets/instructions; cap length; 402/down falls back to the extractive digest with `summary_state='pending_summary_retry'` semantics unchanged.

Accept: digests are human-useful and cite contributor rows; golden summary-path cases improve; extractive fallback intact.

Tests: stub-gateway digest shape test; fallback test; leak test (seed a secret-shaped string, assert absent).

Review (human): read a week of generated digests for tone and leakage before enabling beyond dev.

---

## MEM-017 - Incremental summarization + queue + race fix

refs R26, R36, R93, F29, F33 | est 10h | deps MEM-016 | priority high

Files: `src/brain/summarize.rs`, `src/brain/ingest_worker.rs`, new `migrations/0012_summary_queue.sql`.

Steps:
1. `brain_summary_queue(tenant_id, scope_kind, scope_id, dirty_count, first_dirty_seq, updated_at, pk(tenant,scope_kind,scope_id))`; ingest only UPSERTs dirty counts (no COUNT queries, no summarize on the hot path).
2. Maintenance tick drains the queue: scopes with `dirty_count >= summary_min_new_events`, reading only events with `source_seq > covered_seq_hi`, merging with the prior digest (rolling update).
3. Bound `time_window` scopes to their ISO week (`ts_ns` range) instead of all events.
4. Concurrency: per-scope advisory lock (`pg_advisory_xact_lock(hashtext(tenant||scope))`) plus retry-on-unique-violation for the version race.

Accept: per-event overhead is one UPSERT; re-summarize cost proportional to new events; concurrent supersede safe under a stress test.

Tests: queue drain integration; race test (two concurrent builders, one scope); week-bounding test; perf assertion on query count per ingested event.

Review (human): check summary staleness metric stays under the 10-minute SLO on dev load.

---

## MEM-018 - Summary hierarchy (leaf/mid/root profiles)

refs R27 | est 10h | deps MEM-017 | priority medium

Files: `src/brain/summarize.rs`, migration adding `level smallint default 0` + `parent_scope` to `brain_summary`, `src/brain/recall.rs`.

Steps: level 0 = existing scopes; level 1 = weekly per-subject/channel rollups built from level-0 digests; level 2 = one profile row per subject (and per channel) refreshed on cadence from level-1 rows; recall searches all current levels (level in explain); profile row is the default context injection for agent callers (documented contract).

Accept: profile per active subject exists and refreshes; recall surfaces the right level for broad vs narrow queries on golden cases.

Tests: hierarchy build integration; recall-level assertions; refresh-cadence test.

Review (human): read several generated profiles; they are what Lumi will quote about people, so tone and accuracy matter.

---

## MEM-019 - Lexical retriever leg + hybrid RRF

refs R11, F12 | est 8h | deps MEM-012 | priority high

Files: `src/brain/recall.rs`, migration for FTS expression indexes (`brain_summary.digest`, `brain_fact.body`), config weights.

Steps: add a lexical retriever (websearch_to_tsquery over digests + facts, pg_trgm for handles) as a third RRF leg beside summary-vector and event-vector; extend `rrf_fuse` to n legs with per-leg rank maps; keep the gateway-down fallback as today.

Accept: hybrid beats vector-only on the golden set (record delta); latency budget respected.

Tests: golden-set comparison in the eval runner; leg-presence assertions in explain.

Review (human): none beyond gate + golden delta.

---

## MEM-020 - Cross-encoder rerank stage

refs R12, F10 | est 8h | deps MEM-019 | priority high

Files: `services/embed-sidecar` (add `/v1/rerank` with bge-reranker-v2-m3), `src/brain/rerank_client.rs`, `src/brain/recall.rs`.

Steps: after fusion take top-50 (with snippets from MEM-006), one batched rerank call `{query, documents[]} -> scores[]`, blend `final = 0.7*rerank + 0.3*fused` (configurable), skip and mark `degraded_backends` when the sidecar is down; time it as its own leg.

Accept: golden-set lift recorded; p95 within SLO; degraded path clean.

Tests: sidecar contract test; blend unit test; degraded integration test.

Review (human): eyeball the latency cost against the quality lift; the blend weight is tunable later by MEM-049.

---

## MEM-021 - Park scoring + MMR

refs R2, R13, F1, F10 | est 8h | deps MEM-019 | priority high

Files: `src/brain/recall.rs`, new `src/brain/scoring.rs`, migration adding `access_count`/`last_accessed_at` to `brain_event_embedding` (facts already have them).

Steps: implement `final = 0.4*relevance + 0.3*importance + 0.3*recency` per FR-MEMORY-113, components normalized to [0,1]; recency = exponential decay on `last_accessed_at` (fallback `ts_ns`), decay constant from `modules/memory/cyberos/core/decay.py`; importance defaults 0.5 until MEM-024 fills it; apply after rerank, then MMR (lambda 0.7, port `mmr.py` selection loop) as the final filter; recall updates `last_accessed_at`/`access_count` for returned hits (batched UPDATE).

Accept: near-duplicate hits diversified on a seeded test; stale-but-relevant vs fresh ordering matches FR-113 examples; explain shows all terms.

Tests: scoring unit tests ported from the Python suites; MMR diversity test; access-stat update test.

Review (human): confirm weight defaults; they become the A/B baseline in MEM-049.

---

## MEM-022 - Contextual embedding prefix

refs R14 | est 4h | deps MEM-014 | priority medium

Files: `src/brain/ingest_worker.rs`, `src/brain/summarize.rs`, `src/brain/fact_ops.rs`, `backfill.rs`.

Steps: one helper builds "`[kind] subject=<display> channel=<name> at=<date>:`" prefixes applied before embedding events, facts, and digests (and the same at backfill); keep the prefix out of stored snippets; record prefix version in `embed_model_version` suffix or a new column so re-embeds are detectable.

Accept: golden delta recorded; backfill idempotent by content_hash (MEM-026).

Tests: prefix determinism unit test; ingest/backfill parity test.

Review (human): none beyond golden delta.

---

## MEM-023 - Query rewriting (temporal EN+VN, handle expansion)

refs R15 | est 6h | deps MEM-019 | priority medium

Files: new `src/brain/query_rewrite.rs`, `src/brain/recall.rs`.

Steps: deterministic parsers first (no LLM): relative-time expressions in EN and VN ("last week", "thang truoc", "hom qua") to `ts_since/ts_until`; `@handle` and known display names to subject UUIDs via `l2_entity`; strip the parsed spans from the lexical query; expose rewrites in explain. LLM paraphrase probe stays out of scope (icebox note) until evals justify it.

Accept: temporal golden cases pass; VN expressions covered by tests; rewrite visible in explain.

Tests: parser table tests EN+VN; end-to-end temporal recall test.

Review (human): skim the VN expression table for coverage gaps.

---

## MEM-024 - Write-time importance scoring

refs R31 | est 6h | deps MEM-013 | priority medium

Files: `src/brain/fact_ops.rs` (score at ADD/UPDATE), prompt with anchored rubric, metrics.

Steps: cheap gateway model rates 1-10 with three pinned anchor examples per band (guards drift-high); store normalized to [0,1] in `importance`; batch with extraction to avoid extra calls; distribution histogram to the ops metrics; events keep default importance until promoted.

Accept: scores populate on new facts; distribution not collapsed to the top band; spend delta negligible (batched).

Tests: rubric prompt snapshot test; stub scoring integration; distribution sanity test.

Review (human): review the rubric anchors; they encode what the company considers important.

---

## MEM-025 - Day-1 emitters + real consent gate

refs R103, F19 | est 14h | deps MEM-001 | priority critical

Files: `services/chat` (message_created/edited/deleted emitters with `content_ref::pointer("chat_messages", id)`), `services/auth` (sign-in events - may already chain; align kinds), `services/obs-*` (incident events), `modules/proj` if live; new `src/interaction/eval_gate.rs` implementing `ConsentGate` against the FR-EVAL-001 `monitoring_notice`/`subject_acknowledgment` tables, wrapped in `CachingGate` (60s).

Steps:
1. Implement the ledger-backed gate (deny unless a current-notice acknowledgment row exists); inject it where emitters construct the gate; delete production reachability of `AllowAll` (cfg(test) or feature-gated).
2. Wire chat emitters first (highest value): on message create/edit/delete, emit interaction events with pointers; respect the emit best-effort contract (never block the message path).
3. Auth + obs emitters next; keep attributes under the 2 KiB cap; verify the denylist rule (MEM-044 will harden).
4. Load-test the gate cache under a chat burst.

Accept: consented subjects' chat activity appears in the brain within ingest-lag SLO; unacknowledged subjects produce zero rows (test); AllowAll unreachable in release builds.

Tests: gate integration against seeded ledger; emitter tests per module; burst test asserting one ledger read per subject per TTL.

Review (human): governance checkpoint - confirm the monitoring notice text and acknowledgment flow shipped (brain-evaluation-plan Phase 0) before flipping emitters on in prod; this is a legal gate, not an engineering one.

---

## MEM-026 - Embedding lifecycle basics

refs R37, R38, R39, R40, R41, F28 | est 12h | deps MEM-007 | priority high

Files: migration 0013 (halfvec columns via `ALTER ... TYPE halfvec(1024)`, `content_hash text`, `embedded_at timestamptz`; recreate partial indexes with `embed_model_version` pinned in predicates), `src/brain/embed_client.rs` (batch input[]), `src/brain/ingest_worker.rs` (batch loop), `backfill.rs` (checkpointed, hash-idempotent), `docs/deploy/memory-embedding-migration-runbook.md`.

Steps: halfvec conversion measured on dev (index size + recall parity via golden set); batch embeds (default 64/call, env cap) for worker + retry + backfill; skip re-embed when `content_hash` unchanged; write the dual-column shadow migration runbook (add `embedding_v2`, backfill, concurrent index, golden eval old-vs-new, flag flip, drop after soak) and rehearse it once on dev with a fake model version.

Accept: index size roughly halves with golden parity; ingest lag improves measurably from batching; runbook rehearsed end to end on dev.

Tests: hash-skip test; batch path test; runbook rehearsal recorded in ledger.

Review (human): read the runbook; approve the halfvec cutover plan for prod data.

---

## MEM-027 - Warm tier reachable on drill

refs R20, F11 | est 4h | deps MEM-026 | priority medium

Files: `src/brain/recall.rs`, optional migration for a warm partial index (halfvec + reduced ef) if the seq-scan budget fails.

Steps: `drill=true` includes `tier IN ('hot','warm')` with a bounded LIMIT and its own leg timing; decide index vs scan from measured cost at dev scale; update `tiering.rs` docs so code and comments agree.

Accept: warm events retrievable on drill within budget; docs accurate.

Tests: tier-transition then drill-recall integration test.

Review (human): none beyond gate.

---

## MEM-028 - Recall API v1.1 (pagination, explain scores, feedback)

refs R21, R23, R24, F13 | est 8h | deps MEM-021 | priority medium

Files: `src/brain/mod.rs` (RecallQuery cursor fields, response next_cursor), `src/brain/recall.rs`, new feedback handler + route, migration for a `brain_recall_feedback` table (or chained events only - prefer chain + counters).

Steps: keyset cursor over `(final_score, source_seq)`; explain gains per-leg ranks, rerank score, decay/importance terms, rewrite info; `POST /v1/memory/feedback {audit_row_id|fact_id, used, helpful}` updates access stats and chains a `memory.recall_feedback` event; JWT + rate limits apply (MEM-001/004).

Accept: stable paging past 100 without duplicates; feedback events visible on the chain and reflected in counters.

Tests: paging property test; feedback round-trip test.

Review (human): confirm the feedback event kind is in the console viewer filter list.

---

## MEM-029 - Query-embed cache + shared EmbedClient

refs R22, F14 | est 3h | deps none | priority low

Files: `src/state.rs` (EmbedClient in AppState), `src/brain/handler.rs`, small LRU (e.g. `moka`) keyed by (model_version, sha256(text)), TTL 5m, size cap.

Accept: repeat queries skip the gateway (hit counter); client constructed once.

Tests: cache hit/expiry unit tests.

Review (human): none.

---

## MEM-030 - Bi-temporal l2_edge + invalidation

refs R7, F5 | est 8h | deps MEM-003 | priority high

Files: migration 0014 (`valid_at` default now, `invalid_at`, `expired_at` on `l2_edge`; indexes on current edges), `src/layer2/pgvector.rs` upsert helpers, new `src/layer2/edge_ops.rs`.

Steps: edge writes set `valid_at`; `invalidate_edge` sets `invalid_at`/`expired_at` with a chained audit row; point-in-time query helper (`edges_at(t)`); no DELETE path for edges anywhere.

Accept: contradicting edge supersedes with history queryable; nothing hard-deleted.

Tests: bi-temporal query tests; invalidation audit test.

Review (human): none beyond gate.

---

## MEM-031 - LLM entity extraction, graph leg, query router

refs R8, R16, R17, F6 | est 16h | deps MEM-030, MEM-013 | priority medium

Files: new `src/layer2/entity_llm.rs`, `src/brain/recall.rs` (graph leg + router), prompts.

Steps:
1. Per episode batch: gateway extraction of typed entities + relations (schema-validated); candidate match against `l2_entity` by embedding + FTS; model-adjudicated merge with `name_aliases[]` (jsonb properties); write bi-temporal edges (MEM-030); regex extractor remains the fallback tier.
2. Graph leg: entities matched in the query seed 1-2 hop expansion over current edges via recursive CTE; connected facts/events join the RRF fusion; optional node-distance-from-caller rerank term.
3. Router: heuristic (entity count >=2, relation cue words EN+VN) enables the graph leg; decision logged in explain.

Accept: multi-hop golden cases improve; entity dedup produces aliases not duplicates on a seeded corpus; router decisions visible.

Tests: extraction schema tests; dedup merge test; CTE hop test; golden multi-hop cases.

Review (human): spot-check merged entities for false merges (people with similar names are the risk).
