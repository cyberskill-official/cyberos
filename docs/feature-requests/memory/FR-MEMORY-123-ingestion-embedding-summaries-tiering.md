---
id: FR-MEMORY-123
title: "BRAIN ingestion + embedding + rolling summaries + hot/warm/cold tiering + access-scoped recall — the interaction-event log becomes a fast, persistent, citable brain (HNSW sub-second recall, summaries-first, audit chain stays system of record)"
module: MEMORY
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · BRAIN slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-101, FR-MEMORY-108, FR-MEMORY-113, FR-MEMORY-114, FR-MEMORY-117, FR-MEMORY-121, FR-MEMORY-122, FR-AI-019, FR-AI-020, FR-AI-022, FR-AUTH-003, FR-AUTH-004, FR-EVAL-001, FR-OBS-001]
depends_on: [FR-MEMORY-121, FR-MEMORY-122]
blocks: [FR-EVAL-003]

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#persistent-fast-retrieval-memory-design
  - website/docs/modules/memory.html#brain
source_decisions:
  - DEC-2720 (WIDE day-1 capture feeds the brain; the ingest worker consumes the FR-MEMORY-121 interaction-event log produced by the FR-MEMORY-122 emitters; no event type is special-cased out of ingestion)
  - DEC-2721 (the l1_audit_log hash chain stays the SYSTEM OF RECORD; the vector index + summaries are a DERIVED fast lens — rebuildable from Layer 1, never authoritative, Layer 1 wins on any conflict; reuse FR-MEMORY-101 DEC-070 invariant)
  - DEC-2722 (ACCESS-RESTRICTED brain: embeddings + summaries inherit tenant RLS AND the per-subject access rules from FR-EVAL-001; recall MUST NOT leak across the access boundary even when the vector neighbour is semantically closest)
  - DEC-2723 (residency-pinned, spend-capped embeddings via the ai-gateway — the worker never calls a model provider directly; the gateway enforces region + ZDR + the tenant spend cap; reuse FR-AI-022 policy surface)
  - DEC-2724 (summaries-first recall: query the rolling per-subject / per-channel / per-window summaries first; drill into raw hot events only on demand — keeps long-term memory compact + recall fast as the log grows)
  - DEC-2725 (hot/warm/cold tiering by age bounds cost + latency: hot = recent raw events fully indexed; warm = older events behind summaries with embeddings retained; cold = archived raw, summary-only index, raw retrievable on demand)
  - DEC-2726 (every recall result carries a provenance pointer back to the exact l1_audit_log row(s) it was derived from, so FR-EVAL-003 can cite exact events; a summary cites the event range it compacted)

language: rust 1.81
service: cyberos/services/memory/
new_files:
  - services/memory/src/brain/mod.rs
  - services/memory/src/brain/ingest_worker.rs
  - services/memory/src/brain/event_cursor.rs
  - services/memory/src/brain/embed_client.rs
  - services/memory/src/brain/summarize.rs
  - services/memory/src/brain/tiering.rs
  - services/memory/src/brain/recall.rs
  - services/memory/src/brain/access_scope.rs
  - services/memory/src/brain/provenance.rs
  - services/memory/src/brain/backfill.rs
  - services/memory/src/brain/metrics.rs
  - services/memory/migrations/0006_brain_event_embeddings.sql
  - services/memory/migrations/0007_brain_summaries.sql
  - services/memory/migrations/0008_brain_tier_cursor.sql
  - services/memory/tests/brain_ingest_test.rs
  - services/memory/tests/brain_recall_access_scope_test.rs
  - services/memory/tests/brain_summaries_test.rs
  - services/memory/tests/brain_tiering_test.rs
  - services/memory/tests/brain_provenance_test.rs
  - services/memory/tests/brain_backfill_rebuild_test.rs
  - services/memory/tests/brain_residency_spend_test.rs
modified_files:
  - services/memory/src/main.rs                 # spawn the brain ingest worker alongside the layer2 loop
  - services/memory/src/handlers/mod.rs         # mount POST /v1/memory/recall
modified_files_python:
  - modules/ai-gateway/gateway/embeddings.py    # note: residency + spend-cap path the worker calls (no new behaviour; documented contract)
allowed_tools:
  - file_read: services/memory/**
  - file_read: modules/ai-gateway/**
  - file_write: services/memory/{src,tests,migrations}/**
  - bash: cd services/memory && cargo test brain
  - bash: cd services/memory && cargo clippy --all-targets -- -D warnings
  - bash: psql -h localhost -U postgres -c "CREATE EXTENSION IF NOT EXISTS vector;"
disallowed_tools:
  - treat the embedding index or summaries as the source of truth (per DEC-2721 — Layer 1 wins on any conflict)
  - return a recall result whose subject the caller is not entitled to under FR-EVAL-001 (per DEC-2722 — access boundary is load-bearing)
  - call a model provider directly for embeddings, bypassing the ai-gateway residency + spend cap (per DEC-2723)
  - emit a recall result without a provenance pointer to its source audit row(s) (per DEC-2726)
  - delete or mutate any l1_audit_log row during ingest, summarisation, or tiering (read-only over the system of record)

effort_hours: 26
sub_tasks:
  - "0.5h: brain/mod.rs — public API surface + shared types (BrainEvent, RecallQuery, RecallHit, Provenance)"
  - "1.5h: 0006_brain_event_embeddings.sql — brain_event_embedding table (HNSW index, tier column, RLS, provenance columns)"
  - "1.5h: 0007_brain_summaries.sql — brain_summary table (scope kind, window, embedding, covered_seq_range, version, RLS)"
  - "0.5h: 0008_brain_tier_cursor.sql — brain_ingest_cursor + brain_tier_watermark per tenant"
  - "1.0h: brain/event_cursor.rs — per-tenant cursor over the FR-MEMORY-121 event stream; restart resume"
  - "2.0h: brain/embed_client.rs — ai-gateway embeddings call (FR-AI-019 via FR-AI-022 policy); residency + spend-cap headers; backoff + pending_embed_retry"
  - "3.0h: brain/ingest_worker.rs — consume events → embed → UPSERT into pgvector with provenance → advance cursor; idempotent on (tenant_id, source_seq)"
  - "3.0h: brain/summarize.rs — rolling per-subject / per-channel / per-window summaries via the ai-gateway; covered_seq_range; supersede prior version; re-summarise on new events in window"
  - "2.5h: brain/tiering.rs — age-based hot→warm→cold transitions; warm keeps embedding drops raw-hot index; cold archives raw + keeps summary-only index; tier watermark per tenant"
  - "3.0h: brain/recall.rs — POST /v1/memory/recall: summaries-first semantic query (HNSW), drill into hot events on demand, RRF over summary + event hits, returns hits + provenance"
  - "2.0h: brain/access_scope.rs — FR-EVAL-001 per-subject access predicate applied AFTER tenant RLS; recall caller entitlement check; deny-by-default on unknown subject"
  - "1.0h: brain/provenance.rs — map every embedding row + summary to its l1_audit_log source row(s); chain_anchor carried for tamper detection on recall (reuse FR-MEMORY-101 verify)"
  - "2.0h: brain/backfill.rs — re-embed / re-summarise / index-rebuild path; --rebuild from the audit chain; model-version migration"
  - "1.0h: brain/metrics.rs — ingest lag, recall latency p50/p99, index size, summary count, tier distribution, spend, access-denied counters"
  - "1.0h: tests — brain_ingest_test (happy ingest, idempotent UPSERT, cursor resume, ingest lag SLO)"
  - "1.5h: tests — brain_recall_access_scope_test (tenant RLS + FR-EVAL-001 subject scope; closest neighbour the caller can't see is excluded; deny-by-default)"
  - "1.0h: tests — brain_summaries_test (rolling summary covers event range, supersede on new events, summaries-first recall path)"
  - "1.0h: tests — brain_tiering_test (hot→warm→cold transitions, cold raw retrievable on demand, latency/cost bound)"
  - "0.5h: tests — brain_provenance_test (every hit cites a source audit row; summary cites covered range; chain_anchor mismatch drops hit)"
  - "0.5h: tests — brain_backfill_rebuild_test (rebuild index + summaries from Layer 1; derived state matches; model-version re-embed)"
  - "0.5h: tests — brain_residency_spend_test (embeddings routed through gateway; spend cap honoured; over-cap → pending, not a direct provider call)"
risk_if_skipped: "Without this FR the captured interaction log (FR-MEMORY-121/122) is an append-only pile no one can query at the speed an evaluation needs — FR-EVAL-003 cannot retrieve evidence, so the whole BRAIN-evaluation plan stalls at Phase 2. Without summaries-first + tiering, recall latency and embedding cost grow linearly with the log forever; at company scale 'what did X commit to about project Y last quarter' becomes a full-table scan. Without the FR-EVAL-001 access scope on recall, the brain leaks one employee's record into another's evaluation context (the closest vector neighbour is returned regardless of who may see it) — a privacy and trust breach under Vietnam's PDPD and the signed NDA. Without provenance pointers, an assessment cites 'the brain said so' instead of an exact, tamper-evident audit row, which is indefensible in a performance or IP dispute. Without residency + spend-cap routing, embeddings leave the residency region and the tenant's AI spend is unbounded."
---

## §1 — Description (BCP-14 normative)

A long-lived Rust worker (`brain-ingest`, spawned in `services/memory` alongside the Layer-2 loop) **MUST** consume the FR-MEMORY-121 interaction-event log produced by the FR-MEMORY-122 emitters, embed each event into pgvector with an HNSW index, maintain rolling summaries, tier storage by age, and serve an access-scoped, provenance-carrying recall API. The l1_audit_log hash chain remains the system of record; everything this FR builds is a derived, rebuildable fast lens (DEC-2721). The contract:

1. **MUST** consume interaction events from the FR-MEMORY-121 stream via a per-tenant cursor (`brain_ingest_cursor (tenant_id PK, last_source_seq, updated_at)`); persist the last-consumed `source_seq` to Postgres so a restart resumes without re-embedding or skipping. WIDE day-1 capture (DEC-2720): every event kind the emitters produce is ingested; no kind is special-cased out.
2. **MUST** compute an embedding for every ingested event's body through the ai-gateway embeddings path (FR-AI-019 model, FR-AI-022 policy surface) — NEVER by calling a model provider directly (DEC-2723). The call MUST carry the tenant's residency pin and ZDR flag and MUST be charged against the tenant spend cap. Insert into `brain_event_embedding` with `(tenant_id, source_seq, audit_row_id, subject_id, channel_id, kind, ts_ns, embedding vector(1024), chain_anchor BYTEA, tier)`.
3. **MUST** build an HNSW index on `brain_event_embedding.embedding` (cosine ops) so semantic recall over the hot tier returns in sub-second time (p99 ≤ 1s on the slice-1 fixture; SLO via `brain_recall_latency_ms`).
4. **MUST** maintain rolling summaries (DEC-2724) in `brain_summary`, one row per `(scope_kind, scope_id, window)` where `scope_kind ∈ subject | channel | time_window`:
    - A summary compacts the events in its window into a short natural-language digest plus its own embedding (via the same ai-gateway path).
    - The summary row MUST record `covered_seq_range int8range` — the inclusive range of `source_seq` it compacted — and a monotonic `version`.
    - When new events land in an already-summarised window, the worker MUST re-summarise and write a NEW version that supersedes the prior (the prior is retained for audit, marked `superseded_by`); recall reads the current version.
5. **MUST** make recall summaries-first (DEC-2724): a recall query searches `brain_summary` embeddings first; raw hot-event embeddings are searched and merged only to satisfy `?drill=true` or when summary recall is below the configured confidence floor. This keeps recall fast and cheap as the log grows.
6. **MUST** tier `brain_event_embedding` rows by age (DEC-2725) via a `tier` column `hot | warm | cold` and a per-tenant `brain_tier_watermark`:
    - **hot** — recent raw events, fully HNSW-indexed, searched directly.
    - **warm** — older events: the embedding is retained (still vector-searchable on drill) but the raw event is represented in recall through its summary; the hot index need not cover it.
    - **cold** — archived: the raw event stays in Layer 1 (system of record) and in cold storage; only the summary embedding is indexed; the raw row is retrievable on demand by `audit_row_id` (DEC-2726) without being held in the hot index.
    Tier transitions are age-driven (`hot_max_age`, `warm_max_age` config, default 30d / 180d) and MUST be idempotent — re-running the tiering pass does not duplicate or lose rows.
7. **MUST** expose `POST /v1/memory/recall` returning ranked evidence. The request body accepts `{q, subject_scope?, channel_scope?, ts_since?, ts_until?, limit?, drill?, explain?}`; `limit` default 10, max 100.
8. **MUST** scope every recall by tenant RLS AND the FR-EVAL-001 per-subject access rules (DEC-2722). The caller's tenant_id (from the FR-AUTH-004 JWT) scopes the tables via `current_setting('app.tenant_id')`; the FR-EVAL-001 access predicate then filters to the subjects the caller is entitled to. A semantically-closest neighbour whose subject the caller may NOT see **MUST** be excluded from results (not merely deranked). Unknown subject → deny by default.
9. **MUST** return, per hit, `{audit_row_id, subject_id, channel_id, kind, ts_ns, snippet, score, source: "event" | "summary", provenance}` where `provenance` points back to the exact l1_audit_log row(s) the hit was derived from (DEC-2726): an event hit cites its single `audit_row_id`; a summary hit cites its `covered_seq_range` plus the top contributing `audit_row_id`s. Downstream FR-EVAL-003 cites these exact rows.
10. **MUST** verify `chain_anchor` for each returned hit against Layer 1 (reuse FR-MEMORY-101 §1 #4 / FR-MEMORY-108 §1 #10): recompute `SHA-256(canonical(Layer 1 row at source_seq))`; on mismatch, drop the hit and emit a sev-1 `memory_brain_chain_anchor_mismatch{tenant_id, source_seq}` event. The derived index is never trusted over a tampered chain.
11. **MUST** keep the audit chain the system of record (DEC-2721): ingest, summarisation, and tiering are READ-ONLY over `l1_audit_log`. On any conflict between a derived row and Layer 1, Layer 1 wins and the derived row is flagged `stale` for re-ingest. The worker MUST NOT write, delete, or mutate any audit row.
12. **MUST** be idempotent on `(tenant_id, source_seq)` (UPSERT) so restart-mid-batch re-processing produces no duplicate embedding rows; summary re-computation supersedes by version rather than duplicating.
13. **MUST** route embeddings + summary generation through the ai-gateway with residency + spend-cap discipline (DEC-2723): each call passes the tenant policy (region, ZDR, model alias) from FR-AI-022; when the tenant spend cap is exhausted the worker MUST mark the row `pending_embed_retry` / `pending_summary_retry` and back off, NOT fall back to a direct provider call. Backoff is exponential (100ms, 250ms, 500ms, 1s, 2s) up to 5 attempts before marking pending.
14. **MUST** provide a backfill + rebuild path (`--rebuild`, `--reembed --model <alias>`, `--resummarize <scope>`): rebuild re-derives `brain_event_embedding` and `brain_summary` from the Layer-1 chain from `source_seq` 0; re-embed migrates to a new embedding model version (recording `embed_model_version` per row); index rebuild (`REINDEX` of the HNSW index) runs without dropping recall availability beyond a documented window. Rebuild output MUST match a fresh ingest of the same Layer-1 range (derivability invariant, reuse FR-MEMORY-102 gate).
15. **MUST** emit OTel metrics:
    - `memory_brain_ingest_lag_seconds{tenant_id}` (histogram; event-append → embedding-visible).
    - `memory_brain_recall_latency_ms{tenant_id, path}` (histogram; `path ∈ summary | drill`; SLO p50 + p99 reported).
    - `memory_brain_index_size_rows{tenant_id, tier}` (gauge).
    - `memory_brain_summary_count{tenant_id, scope_kind}` (gauge).
    - `memory_brain_tier_rows_total{tenant_id, tier}` (gauge; hot/warm/cold distribution).
    - `memory_brain_embed_spend_units{tenant_id}` (counter; embedding spend charged via the gateway).
    - `memory_brain_recall_access_denied_total{tenant_id, reason}` (counter; `reason ∈ tenant_rls | subject_scope | unknown_subject`).
    - `memory_brain_ingest_failures_total{tenant_id, reason}` (counter; `reason ∈ embed_gateway_down | spend_cap_exhausted | postgres_error | chain_anchor_mismatch`).
16. **MUST** enforce tenant isolation via `tenant_id` + RLS (FR-AUTH-003 pattern) on `brain_event_embedding`, `brain_summary`, and both cursor tables — `USING` + `WITH CHECK`, `FORCE ROW LEVEL SECURITY`; the tables join the `TENANT_SCOPED_TABLES` registry.
17. **MUST** apply FR-MEMORY-117 store-ACL semantics to any summary the worker writes into the memory tree (subject summaries are written under an ACL-governed subtree, e.g. `company/brain/subjects/`); the worker runs under a reserved actor identity (`brain-ingest`) and MUST be rejected (with a `memory.acl_denied` aux row) if it lacks write capability on the target subtree.
18. **MUST** support graceful degrade on recall: if the ai-gateway embeddings path is down at query time (needed to embed the query), recall falls back to full-text over summaries (reuse FR-MEMORY-108 PGroonga path) and surfaces the degradation in `?explain=true`; when both summary search and event search are impossible, return `503`. Empty results are `200` with `[]` (search semantics, per FR-MEMORY-108 §1 #12).
19. **MUST** support graceful shutdown: on SIGTERM the in-flight ingest transaction completes, the cursor and tier watermark are saved, summary jobs drain, and the worker exits with no partial commit.
20. **SHOULD** weight recall by FR-MEMORY-114 write-time importance and FR-MEMORY-113 recency decay where available, so an evaluation surfaces material recent evidence ahead of stale noise; absence of those signals MUST NOT block recall (graceful fallback to raw cosine + RRF).

---

## §2 — Why this design (rationale for humans)

**Why consume the event log rather than re-define capture (DEC-2720).** FR-MEMORY-121 already fixes the interaction-event shape and FR-MEMORY-122 already emits it; this FR is Phase 2 — the brain, not the capture. Re-defining the event here would fork the schema. The worker's only input is the FR-MEMORY-121 stream, and it ingests every kind so the brain is complete by construction.

**Why the audit chain stays the system of record (DEC-2721, reuse DEC-070).** The vector index and summaries are lossy and model-dependent: an embedding is a projection, a summary is a paraphrase. Neither can be authoritative for a record that may inform pay or a legal dispute. Keeping `l1_audit_log` as the source of truth means the brain is always rebuildable from the tamper-evident chain, and a model swap or an index bug can never corrupt the record — it can only require a re-derive.

**Why summaries-first, then drill (DEC-2724).** At company scale the raw event log grows without bound; searching every raw embedding for every recall would make latency and embedding cost climb forever. Rolling per-subject / per-channel / per-window summaries are a compact, queryable abstraction: most recall ("how has X been contributing on project Y") is answered from summaries in one cheap hop, and the worker only pays to search raw hot events when the caller explicitly drills or the summary answer is weak. This is the standard retrieval-augmented pattern applied to our own data.

**Why hot/warm/cold tiering (DEC-2725).** Recency dominates relevance for evaluation, and indexing everything hot forever is the cost-and-latency trap. Hot keeps recent raw events fully indexed for precise, fresh recall; warm demotes older raw events behind their summaries (embedding retained for the rare drill, but out of the hot index that every query pays for); cold archives the raw rows (still in Layer 1, the truth) and keeps only the summary indexed, with the raw retrievable on demand by `audit_row_id`. The result bounds both the hot index size and the per-query latency regardless of how old the company gets.

**Why the FR-EVAL-001 access scope on recall, not just tenant RLS (DEC-2722).** Tenant RLS stops cross-company leakage, but within a company the brain holds every employee's record, and the closest vector neighbour to a query is returned regardless of subject. If recall ignored per-subject access, one person's evaluation context would surface another person's private interactions — a direct breach of the signed NDA and Vietnam's PDPD purpose-limitation. The access predicate from FR-EVAL-001 runs after RLS and EXCLUDES (not deranks) any subject the caller may not see; unknown subjects deny by default so a gap fails closed.

**Why provenance pointers back to audit rows (DEC-2726).** An assessment that says "the brain indicated low engagement" is indefensible; an assessment that cites three exact, hash-chained audit rows is evidence. Every recall hit therefore carries a pointer to its source row(s) — an event hit to its single row, a summary hit to the covered range plus the top contributors — so FR-EVAL-003 can quote the precise events and a reviewer (or the employee) can verify them against the immutable chain.

**Why route embeddings through the ai-gateway (DEC-2723).** The gateway already owns model routing, the residency pin, ZDR, and the tenant spend cap (FR-AI-022). The worker calling a provider directly would re-implement and inevitably drift from that policy, and could ship employee-interaction text out of the residency region or blow the spend cap silently. Over-cap MUST degrade to `pending_*` and back off, never to a direct call, so the policy boundary is unbreakable.

**Why idempotent UPSERT + version-superseding summaries (§1 #12, #4).** Restart-mid-batch is the recovery case: the cursor is saved at `source_seq` N and some rows up to N+1 may have been written before commit. UPSERT on `(tenant_id, source_seq)` makes re-processing a no-op for events; for summaries, writing a new version (and marking the old `superseded_by`) preserves the audit of how the summary evolved without duplicating the current row.

**Why chain_anchor verify on recall (§1 #10, reuse FR-MEMORY-101/108).** The derived index can lag a Layer-1 tamper. Recomputing the anchor from the current Layer-1 row at read time catches drift: a mismatch means the chain under this hit changed, so the hit is dropped and a sev-1 fires rather than feeding stale or tampered evidence into an evaluation.

**Why importance + recency weighting is a SHOULD, not a MUST (§1 #20).** FR-MEMORY-114 (write-time importance) and FR-MEMORY-113 (recency decay) sharpen ranking, but the brain must work before they are wired in this tenant. Treating them as graceful enhancements keeps the dependency edge soft: recall degrades to raw cosine + RRF without them, and improves automatically once they are present.

---

## §3 — API contract

### Migrations

```sql
-- services/memory/migrations/0006_brain_event_embeddings.sql
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE brain_event_embedding (
    tenant_id          UUID   NOT NULL,
    source_seq         BIGINT NOT NULL,              -- FR-MEMORY-121 event seq (the cursor key)
    audit_row_id       TEXT   NOT NULL,              -- provenance pointer into l1_audit_log
    subject_id         UUID   NOT NULL,              -- whose interaction (FR-EVAL-001 access subject)
    channel_id         UUID,                         -- where (chat channel / module surface), nullable
    kind               TEXT   NOT NULL,              -- event kind from FR-MEMORY-121
    ts_ns              BIGINT NOT NULL,
    embedding          vector(1024) NOT NULL,
    embed_model_version TEXT  NOT NULL,              -- for re-embed migrations (§1 #14)
    chain_anchor       BYTEA  NOT NULL,              -- SHA-256(canonical Layer-1 row) for read-time verify
    tier               TEXT   NOT NULL DEFAULT 'hot'
                       CHECK (tier IN ('hot','warm','cold')),
    embed_state        TEXT   NOT NULL DEFAULT 'complete'
                       CHECK (embed_state IN ('complete','pending_embed_retry')),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, source_seq)
);

-- hot-tier HNSW index for sub-second semantic recall (§1 #3); partial to keep the index small (§1 #6)
CREATE INDEX brain_event_embedding_hot_hnsw ON brain_event_embedding
    USING hnsw (embedding vector_cosine_ops) WHERE tier = 'hot';
CREATE INDEX brain_event_embedding_subject_idx ON brain_event_embedding (tenant_id, subject_id, ts_ns DESC);
CREATE INDEX brain_event_embedding_tier_idx    ON brain_event_embedding (tenant_id, tier);
CREATE INDEX brain_event_embedding_pending_idx ON brain_event_embedding (tenant_id)
    WHERE embed_state = 'pending_embed_retry';

ALTER TABLE brain_event_embedding ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_event_embedding FORCE  ROW LEVEL SECURITY;
CREATE POLICY brain_event_isolation ON brain_event_embedding
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
```

```sql
-- services/memory/migrations/0007_brain_summaries.sql
CREATE TABLE brain_summary (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID   NOT NULL,
    scope_kind       TEXT   NOT NULL CHECK (scope_kind IN ('subject','channel','time_window')),
    scope_id         TEXT   NOT NULL,                 -- subject_id | channel_id | window key (e.g. '2026-W26')
    window_start_ns  BIGINT NOT NULL,
    window_end_ns    BIGINT NOT NULL,
    covered_seq_range INT8RANGE NOT NULL,             -- inclusive range of source_seq compacted (§1 #4)
    digest           TEXT   NOT NULL,                 -- short natural-language summary
    embedding        vector(1024) NOT NULL,
    embed_model_version TEXT NOT NULL,
    version          BIGINT NOT NULL DEFAULT 1,
    superseded_by    UUID,                            -- newer version that replaces this one (NULL = current)
    top_contributors JSONB  NOT NULL DEFAULT '[]',    -- top audit_row_ids for provenance (§1 #9)
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, scope_kind, scope_id, version)
);

-- summaries-first recall index (§1 #5); current versions only
CREATE INDEX brain_summary_current_hnsw ON brain_summary
    USING hnsw (embedding vector_cosine_ops) WHERE superseded_by IS NULL;
CREATE INDEX brain_summary_scope_idx ON brain_summary (tenant_id, scope_kind, scope_id)
    WHERE superseded_by IS NULL;

ALTER TABLE brain_summary ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_summary FORCE  ROW LEVEL SECURITY;
CREATE POLICY brain_summary_isolation ON brain_summary
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
```

```sql
-- services/memory/migrations/0008_brain_tier_cursor.sql
CREATE TABLE brain_ingest_cursor (
    tenant_id       UUID PRIMARY KEY,
    last_source_seq BIGINT NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE brain_tier_watermark (
    tenant_id            UUID PRIMARY KEY,
    last_tiered_ts_ns    BIGINT NOT NULL,             -- events older than (now - hot_max_age) below this are tiered
    last_tier_run_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Rust API

```rust
// services/memory/src/brain/mod.rs
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct BrainEvent {
    pub source_seq:   i64,
    pub audit_row_id: String,
    pub subject_id:   Uuid,
    pub channel_id:   Option<Uuid>,
    pub kind:         String,
    pub ts_ns:        i64,
    pub body:         String,
    pub chain_anchor: [u8; 32],
}

#[derive(serde::Deserialize)]
pub struct RecallQuery {
    pub q: String,
    pub subject_scope: Option<Vec<Uuid>>,   // optional narrowing; still re-checked against FR-EVAL-001
    pub channel_scope: Option<Vec<Uuid>>,
    pub ts_since: Option<i64>,
    pub ts_until: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub drill: bool,                        // search raw hot events too (§1 #5)
    #[serde(default)]
    pub explain: bool,
}
fn default_limit() -> usize { 10 }

#[derive(serde::Serialize)]
pub struct RecallHit {
    pub audit_row_id: String,
    pub subject_id:   Uuid,
    pub channel_id:   Option<Uuid>,
    pub kind:         String,
    pub ts_ns:        i64,
    pub snippet:      String,
    pub score:        f32,
    pub source:       HitSource,            // event | summary
    pub provenance:   Provenance,           // §1 #9 — back-pointer(s) into l1_audit_log
}

#[derive(serde::Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HitSource { Event, Summary }

#[derive(serde::Serialize)]
pub struct Provenance {
    pub audit_row_ids:    Vec<String>,      // exact source rows (1 for event; top-k for summary)
    pub covered_seq_range: Option<(i64, i64)>, // present for summary hits
    pub chain_verified:   bool,
}

#[derive(serde::Serialize)]
pub struct RecallResults {
    pub items:             Vec<RecallHit>,
    pub explain:           Option<serde_json::Value>,
    pub degraded_backends: Vec<String>,
}
```

```rust
// services/memory/src/brain/ingest_worker.rs
pub async fn ingest_one_tenant(tenant_id: Uuid, pool: &PgPool, gw: &EmbedClient) -> anyhow::Result<()> {
    let cursor = event_cursor::get(pool, tenant_id).await?;
    let events = event_stream::read_after(tenant_id, cursor).await?;   // FR-MEMORY-121 source

    for ev in events {
        // §1 #2 / #13: embed via the ai-gateway under residency + spend-cap policy — never a direct provider call.
        let embedding = match gw.embed(tenant_id, &ev.body).await {
            Ok(e) => e,
            Err(EmbedError::SpendCapExhausted) => {
                metrics::ingest_failure(tenant_id, "spend_cap_exhausted");
                mark_pending(pool, tenant_id, ev.source_seq).await?;   // back off; do NOT bypass the gateway
                continue;
            }
            Err(EmbedError::GatewayDown) => {
                metrics::ingest_failure(tenant_id, "embed_gateway_down");
                mark_pending(pool, tenant_id, ev.source_seq).await?;
                continue;
            }
        };

        let mut tx = pool.begin().await?;
        rls_set_tenant(&mut tx, tenant_id).await?;
        sqlx::query(
            "INSERT INTO brain_event_embedding
               (tenant_id, source_seq, audit_row_id, subject_id, channel_id, kind, ts_ns,
                embedding, embed_model_version, chain_anchor, tier, embed_state)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'hot','complete')
             ON CONFLICT (tenant_id, source_seq) DO NOTHING")          // §1 #12 idempotent
            .bind(tenant_id).bind(ev.source_seq).bind(&ev.audit_row_id)
            .bind(ev.subject_id).bind(ev.channel_id).bind(&ev.kind).bind(ev.ts_ns)
            .bind(&embedding).bind(gw.model_version()).bind(&ev.chain_anchor)
            .execute(&mut *tx).await?;
        event_cursor::advance_in_tx(&mut tx, tenant_id, ev.source_seq).await?;
        tx.commit().await?;

        summarize::touch_windows(pool, tenant_id, &ev, gw).await?;     // §1 #4 re-summarise affected windows
        metrics::ingest_lag(tenant_id, now_ns() - ev.ts_ns);
    }
    Ok(())
}
```

```rust
// services/memory/src/brain/recall.rs  — summaries-first, access-scoped, provenance-carrying
pub async fn recall(q: RecallQuery, tenant_id: Uuid, caller: &Caller,
                    pool: &PgPool, gw: &EmbedClient) -> Result<RecallResults, RecallError> {
    if q.limit > 100 { return Err(RecallError::LimitTooLarge); }
    let mut degraded = vec![];

    // embed the query (graceful degrade to full-text over summaries if the gateway is down — §1 #18)
    let q_vec = match gw.embed(tenant_id, &q.q).await {
        Ok(v) => Some(v),
        Err(_) => { degraded.push("query_embed".into()); None }
    };

    // §1 #5: search current summaries first
    let mut hits = summary_search(pool, tenant_id, q_vec.as_ref(), &q).await?;
    // §1 #5: drill into hot raw events on demand or low confidence
    if q.drill || below_confidence_floor(&hits) {
        let event_hits = hot_event_search(pool, tenant_id, q_vec.as_ref(), &q).await?;
        hits = rrf::fuse(hits, event_hits, 60);
    }

    let mut out = Vec::new();
    for h in hits {
        // §1 #10: read-time chain_anchor verify against Layer 1 (reuse FR-MEMORY-101)
        if !chain_anchor_verify::matches_layer1(&h, pool).await? {
            metrics::chain_anchor_mismatch(tenant_id, h.source_seq);
            continue;
        }
        // §1 #8: tenant RLS already applied at SQL; now the FR-EVAL-001 per-subject access predicate.
        // A semantically-closest neighbour the caller may not see is EXCLUDED, not deranked. Deny-by-default.
        if !access_scope::caller_may_see(caller, h.subject_id, pool).await? {
            metrics::access_denied(tenant_id, access_scope::deny_reason(caller, h.subject_id));
            continue;
        }
        out.push(h.into_recall_hit());
        if out.len() >= q.limit { break; }
    }
    Ok(RecallResults { items: out, explain: q.explain.then(build_explain), degraded_backends: degraded })
}
```

```rust
// services/memory/src/brain/embed_client.rs — the ONLY embedding path (DEC-2723)
pub struct EmbedClient { gateway_url: String, /* tenant policy resolved server-side */ }

impl EmbedClient {
    /// Calls the ai-gateway embeddings endpoint, which applies the tenant's residency pin,
    /// ZDR flag, model alias, and spend cap (FR-AI-022). Never contacts a provider directly.
    pub async fn embed(&self, tenant_id: Uuid, body: &str) -> Result<Vec<f32>, EmbedError> {
        // POST {gateway}/v1/embeddings  { tenant_id, input: body }
        // 200 -> embedding ; 402 -> SpendCapExhausted ; 5xx/timeout -> GatewayDown
        todo!()
    }
    pub fn model_version(&self) -> &str { /* echoed by the gateway */ todo!() }
}

#[derive(thiserror::Error, Debug)]
pub enum EmbedError {
    #[error("spend cap exhausted")] SpendCapExhausted,
    #[error("gateway unavailable")] GatewayDown,
}
```

```python
# modules/ai-gateway/gateway/embeddings.py  (CONTRACT NOTE — no new behaviour in this FR)
# The brain worker POSTs {tenant_id, input} here. This endpoint already:
#   - resolves the tenant TenantPolicy (FR-AI-022): region pin, ZDR, model alias, spend cap
#   - routes to the FR-AI-019 BGE-M3 embedding model in-region
#   - charges the embedding against the tenant spend cap; returns 402 when exhausted
#   - echoes embed_model_version so the worker can record it for re-embed migrations
# This FR depends on that contract; it does not change it.
```

---

## §4 — Acceptance criteria

1. **Event ingested → embedding visible** — append a FR-MEMORY-121 event; within the ingest-lag SLO a `brain_event_embedding` row exists with the event's `audit_row_id` and `tier='hot'`.
2. **HNSW hot recall is sub-second** — `EXPLAIN` shows `Index Scan using brain_event_embedding_hot_hnsw`; p99 recall latency ≤ 1s on the slice-1 fixture.
3. **Idempotent ingest** — re-process the same `source_seq` (simulated restart-mid-batch) → exactly one row; cursor resumes without re-embedding earlier rows.
4. **Rolling summary covers an event range** — N events in one channel window → a `brain_summary` row whose `covered_seq_range` spans those events and whose `scope_kind='channel'`.
5. **Summary supersede on new events** — append an event into an already-summarised window → a new `version` is written, old row marked `superseded_by`, recall reads the new version only.
6. **Summaries-first recall** — a recall with `drill=false` answers from `brain_summary` (source=`summary`); `drill=true` additionally returns event-level hits (source=`event`).
7. **Tier transition hot→warm→cold** — age events past `hot_max_age` and `warm_max_age`; tiering pass moves them; hot HNSW index no longer covers warm/cold rows; re-running the pass is a no-op.
8. **Cold raw retrievable on demand** — a cold event is absent from the hot index but its raw Layer-1 row is fetchable by `audit_row_id`.
9. **Tenant RLS on recall** — caller in tenant A never receives tenant B's hits (RLS at DB).
10. **FR-EVAL-001 subject scope excludes the closest neighbour** — seed an event for subject S the caller may NOT see that is the top cosine neighbour of the query; recall EXCLUDES it (not deranks) and increments `memory_brain_recall_access_denied_total{reason="subject_scope"}`.
11. **Deny-by-default on unknown subject** — recall over a subject with no FR-EVAL-001 entitlement returns 0 hits for that subject; `reason="unknown_subject"` increments.
12. **Provenance on every hit** — each event hit cites exactly one `audit_row_id`; each summary hit cites a `covered_seq_range` plus ≥1 `top_contributors` audit_row_id.
13. **Chain_anchor mismatch drops the hit** — corrupt a Layer-1 row under a hit; recall drops it and emits sev-1 `memory_brain_chain_anchor_mismatch`.
14. **Audit chain is read-only** — over a full ingest + summarise + tier cycle, no `l1_audit_log` row is inserted, updated, or deleted by the worker (assert via row-count + chain HEAD unchanged).
15. **Residency + spend cap honoured** — embeddings are observed to flow through the ai-gateway (no direct provider call); when the gateway returns 402, the row is marked `pending_embed_retry` and the worker backs off rather than calling a provider.
16. **Rebuild matches fresh ingest** — `--rebuild` re-derives embeddings + summaries from Layer 1; the derived state matches a fresh ingest of the same range (derivability invariant).
17. **Re-embed model migration** — `--reembed --model <alias>` rewrites embeddings with the new `embed_model_version`; recall still answers; rows record the new version.
18. **Graceful query-embed degrade** — gateway down at query time → recall falls back to full-text over summaries and lists `query_embed` in `explain.degraded_backends`; empty results are `200` with `[]`.
19. **Store-ACL on summary writes** — the `brain-ingest` actor writing a subject summary into an ACL-`deny` subtree is rejected with a `memory.acl_denied` aux row (reuse FR-MEMORY-117).
20. **Metrics emit** — ingest lag, recall latency p50/p99 (per `path`), index size per tier, summary count, tier distribution, spend, and access-denied counters all emit.
21. **Graceful shutdown** — SIGTERM mid-batch commits the in-flight tx, saves cursor + tier watermark, drains summary jobs; restart resumes correctly.
22. **Importance + recency weighting when present** — with FR-MEMORY-114 / FR-MEMORY-113 signals available, recent high-importance evidence ranks ahead of stale noise; absent those signals, recall still returns (raw cosine + RRF).

---

## §5 — Verification

```rust
#[tokio::test]
async fn event_ingested_then_embedding_visible() {
    let env = BrainTestEnv::new().await;
    let ev = env.append_interaction_event(env.tenant(), env.subject_alice(), "chat.message", "shipped the proj sync").await;
    env.run_ingest_once().await;
    let row = sqlx::query("SELECT tier, audit_row_id FROM brain_event_embedding WHERE tenant_id=$1 AND source_seq=$2")
        .bind(env.tenant()).bind(ev.source_seq).fetch_one(env.pool()).await.unwrap();
    let tier: String = row.get("tier");
    assert_eq!(tier, "hot");
}

#[tokio::test]
async fn recall_excludes_closest_neighbour_outside_access_scope() {
    // The single closest cosine neighbour belongs to a subject the caller may NOT see (FR-EVAL-001).
    let env = BrainTestEnv::new().await;
    let bob = env.subject_bob();                       // caller is NOT entitled to bob
    env.append_interaction_event(env.tenant(), bob, "chat.message", "exact query text match").await;
    env.run_ingest_once().await;

    let caller = env.caller_entitled_to(&[env.subject_alice()]);   // alice only
    let res = brain::recall(query("exact query text match"), env.tenant(), &caller, env.pool(), env.gw()).await.unwrap();
    assert!(res.items.iter().all(|h| h.subject_id != bob), "bob's event must be EXCLUDED, not returned");
    let denied: u64 = otel::counter("memory_brain_recall_access_denied_total", &[("reason","subject_scope")]);
    assert!(denied > 0);
}

#[tokio::test]
async fn summaries_first_then_drill() {
    let env = BrainTestEnv::new().await;
    for i in 0..20 { env.append_interaction_event(env.tenant(), env.subject_alice(), "chat.message", &format!("standup note {i}")).await; }
    env.run_ingest_once().await;
    env.run_summarize_once().await;

    let caller = env.caller_entitled_to(&[env.subject_alice()]);
    let summary_only = brain::recall(query("standup"), env.tenant(), &caller, env.pool(), env.gw()).await.unwrap();
    assert!(summary_only.items.iter().any(|h| matches!(h.source, HitSource::Summary)));

    let mut q = query("standup"); q.drill = true;
    let drilled = brain::recall(q, env.tenant(), &caller, env.pool(), env.gw()).await.unwrap();
    assert!(drilled.items.iter().any(|h| matches!(h.source, HitSource::Event)));
}

#[tokio::test]
async fn hot_warm_cold_tiering_is_idempotent() {
    let env = BrainTestEnv::new().await;
    env.seed_events_aged(env.tenant(), /*hot*/ 5, /*warm*/ 5, /*cold*/ 5).await;
    env.run_ingest_once().await;
    env.run_tiering_pass().await;
    let before = env.tier_counts(env.tenant()).await;
    env.run_tiering_pass().await;                       // idempotent
    let after = env.tier_counts(env.tenant()).await;
    assert_eq!(before, after);
    assert!(after.cold > 0 && after.warm > 0 && after.hot > 0);
}

#[tokio::test]
async fn provenance_points_back_to_audit_rows() {
    let env = BrainTestEnv::new().await;
    let ev = env.append_interaction_event(env.tenant(), env.subject_alice(), "chat.message", "decision recorded").await;
    env.run_ingest_once().await;
    let caller = env.caller_entitled_to(&[env.subject_alice()]);
    let res = brain::recall(query("decision recorded"), env.tenant(), &caller, env.pool(), env.gw()).await.unwrap();
    let hit = res.items.first().unwrap();
    assert!(hit.provenance.audit_row_ids.contains(&ev.audit_row_id));
}

#[tokio::test]
async fn chain_anchor_mismatch_drops_hit_sev1() {
    let env = BrainTestEnv::new().await;
    let ev = env.append_interaction_event(env.tenant(), env.subject_alice(), "chat.message", "body").await;
    env.run_ingest_once().await;
    env.corrupt_layer1_row(env.tenant(), ev.source_seq, "TAMPERED").await;
    let caller = env.caller_entitled_to(&[env.subject_alice()]);
    let res = brain::recall(query("body"), env.tenant(), &caller, env.pool(), env.gw()).await.unwrap();
    assert!(res.items.is_empty());
    let m: u64 = otel::counter("memory_brain_chain_anchor_mismatch", &[("tenant_id", &env.tenant().to_string())]);
    assert!(m > 0);
}

#[tokio::test]
async fn spend_cap_exhausted_marks_pending_no_direct_call() {
    let env = BrainTestEnv::new().await;
    env.gw_force_spend_cap_402();                       // gateway returns 402
    let ev = env.append_interaction_event(env.tenant(), env.subject_alice(), "chat.message", "over budget").await;
    env.run_ingest_once().await;
    let state: (String,) = sqlx::query_as("SELECT embed_state FROM brain_event_embedding WHERE tenant_id=$1 AND source_seq=$2")
        .bind(env.tenant()).bind(ev.source_seq).fetch_optional(env.pool()).await.unwrap()
        .unwrap_or((String::from("pending_embed_retry"),));
    assert_eq!(state.0, "pending_embed_retry");
    assert_eq!(env.gw_direct_provider_calls(), 0, "must never bypass the gateway");
}

#[tokio::test]
async fn rebuild_matches_fresh_ingest() {
    let env = BrainTestEnv::new().await;
    env.seed_layer1_chain(env.tenant(), 200).await;
    env.run_ingest_once().await;
    let fresh = env.derived_fingerprint(env.tenant()).await;
    env.truncate_derived(env.tenant()).await;          // drop embeddings + summaries, keep Layer 1
    env.run_rebuild_from_layer1(env.tenant()).await;
    let rebuilt = env.derived_fingerprint(env.tenant()).await;
    assert_eq!(fresh, rebuilt, "derived state must be reproducible from the audit chain");
}

#[tokio::test]
async fn worker_never_writes_audit_chain() {
    let env = BrainTestEnv::new().await;
    let head_before = env.audit_head(env.tenant()).await;
    env.seed_layer1_chain(env.tenant(), 50).await;     // writes via the canonical writer, not the worker
    let head_after_seed = env.audit_head(env.tenant()).await;
    env.run_ingest_once().await;
    env.run_summarize_once().await;
    env.run_tiering_pass().await;
    let head_after_brain = env.audit_head(env.tenant()).await;
    assert_eq!(head_after_seed, head_after_brain, "brain is read-only over Layer 1");
    assert_ne!(head_before, head_after_seed);
}
```

---

## §6 — Implementation skeleton

See §3. The worker is spawned from `services/memory/src/main.rs` next to the Layer-2 loop (per-tenant tokio task), shares the `EmbedClient`, and mounts `POST /v1/memory/recall` in `handlers/mod.rs`. Summarisation and tiering run as periodic passes inside the same process (configurable interval); both are restart-safe via their watermarks.

---

## §7 — Dependencies

- **FR-MEMORY-121** — interaction-event schema; the event shape this worker consumes (`source_seq`, subject, channel, kind, body, `audit_row_id`).
- **FR-MEMORY-122** — capture emitters; produce the events into the stream this worker tails.
- **FR-MEMORY-101** — Layer-2 ingest base + `chain_anchor` compute/verify + canonical Layer-1 bytes (reused for read-time tamper detection).
- **FR-MEMORY-108** — search building blocks reused (RRF, PGroonga full-text fallback, snippet build).
- **FR-MEMORY-113 / FR-MEMORY-114** — recency decay + write-time importance signals (SHOULD-weight recall; soft edge).
- **FR-MEMORY-117** — per-store ACL; governs the subtree the worker writes subject summaries into.
- **FR-AI-019** — BGE-M3 embedding model (invoked only via the gateway).
- **FR-AI-020** — BGE reranker (optional precision polish on recall candidates).
- **FR-AI-022** — ai-gateway tenant policy: residency pin, ZDR, model alias, spend cap (the embeddings path the worker calls).
- **FR-AUTH-003** — RLS pattern + `TENANT_SCOPED_TABLES` registry (add the three brain tables).
- **FR-AUTH-004** — JWT (tenant_id + caller identity on recall).
- **FR-EVAL-001** — per-subject access rules (the recall access predicate; **blocks** nothing here but is consulted at recall time).
- **FR-OBS-001** — OTel metrics.
- Crates: `sqlx@0.7` (postgres + vector), `tokio`, `axum`, `reqwest`, `serde`, `serde_json`, `chrono`, `sha2`, `thiserror`.
- Postgres 16 + pgvector 0.7 (HNSW). Supabase Postgres in prod (DEC: scales to a dedicated DB later without model change).

---

## §8 — Example payloads

### Recall request

```http
POST /v1/memory/recall HTTP/1.1
Authorization: Bearer <jwt>
Content-Type: application/json

{ "q": "what did Daria commit to about the JT1 launch", "drill": false, "limit": 5, "explain": true }
```

### Recall response (summary-first, access-scoped, with provenance)

```json
{
  "items": [
    {
      "audit_row_id": "l1:cyberskill:0001f3a2",
      "subject_id":   "7e57c0de-aaaa-bbbb-cccc-000000000001",
      "channel_id":   "9c3a...",
      "kind":         "chat.message",
      "ts_ns":        1782950400000000000,
      "snippet":      "...committed to ship the JT1 age-verify overlay by 2026-07-01...",
      "score":        0.91,
      "source":       "summary",
      "provenance": {
        "audit_row_ids":     ["l1:cyberskill:0001f3a2", "l1:cyberskill:0001f3b7"],
        "covered_seq_range": [14820, 14975],
        "chain_verified":    true
      }
    }
  ],
  "explain": { "summary_hits": 4, "event_hits": 0, "rrf": false, "query_embed_ms": 22, "summary_search_ms": 31 },
  "degraded_backends": []
}
```

### Tier distribution metric (steady state)

```text
memory_brain_tier_rows_total{tenant_id=cyberskill, tier=hot}  =  4120
memory_brain_tier_rows_total{tenant_id=cyberskill, tier=warm} = 38150
memory_brain_tier_rows_total{tenant_id=cyberskill, tier=cold} = 902340
```

### Access-denied sev signal

```text
memory_brain_recall_access_denied_total{tenant_id=cyberskill, reason=subject_scope} incremented
  caller=stephen@cyberskill.world  excluded_subject=7e57...bob  (closest neighbour, not entitled)
```

---

## §9 — Open questions

All resolved for slice 1. Deferred:
- Streaming ingest (tail the event stream via notify/LISTEN instead of poll) — slice 2.
- Hierarchical summaries (per-subject roll-ups OF per-window summaries) for multi-year horizons — slice 3.
- Per-subject "memory budget" + adaptive summary cadence by activity volume — slice 3+.
- Cross-encoder rerank (FR-AI-020) wired into the recall candidate set by default — slice 2.
- Cold-tier object-storage offload (raw bodies to S3-compatible store, pointer kept) — slice 4+.
- Recall personalisation by reviewer role (manager vs HR vs self-view) beyond the FR-EVAL-001 boundary — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| ai-gateway embeddings down (ingest) | reqwest 5xx/timeout | Row marked `pending_embed_retry`; metric `embed_gateway_down`; worker continues | Self-heals; retry job re-embeds pending rows |
| Tenant spend cap exhausted | gateway 402 | Row marked `pending_embed_retry`; back off; NO direct provider call | Self-heals when cap resets/raised; retry job |
| ai-gateway down at query time | reqwest error in recall | Fall back to full-text over summaries; `query_embed` in degraded | Self-heals when gateway up |
| Postgres deadlock on ingest | sqlx error | tx rollback + retry once | Self-heals |
| Restart mid-batch | UPSERT idempotent on (tenant_id, source_seq) | No duplicate embedding rows; cursor advances correctly | By design |
| Chain_anchor mismatch on recall | read-time verify vs Layer 1 | Drop the hit + sev-1 `memory_brain_chain_anchor_mismatch` | Investigate Layer-1 tamper; re-ingest from chain |
| FR-EVAL-001 subject scope excludes top neighbour | access predicate | Hit excluded (not deranked); `subject_scope` counter | By design |
| Unknown subject (no entitlement record) | access predicate deny-by-default | 0 hits for that subject; `unknown_subject` counter | Operator grants entitlement in FR-EVAL-001 |
| Cross-tenant recall attempt | RLS at DB | 0 rows | By design |
| Summary window churns (high-write channel) | re-summarise on every touch | New version each time; superseded chain grows | Slice-3 adaptive cadence; old versions prunable |
| Summary embedding stale vs current events | covered_seq_range < latest seq | Recall may miss freshest events; drill closes the gap | `drill=true` or re-summarise pass |
| Tiering pass run twice | watermark + idempotent moves | No duplicate/lost rows | By design |
| Cold raw requested but absent from hot index | fetch by audit_row_id from Layer 1 | Raw returned on demand; not in hot index | By design |
| Re-embed mid-flight (model migration) | `embed_model_version` per row | Mixed versions during migration; recall still answers | Migration completes; backfill finishes |
| HNSW index fragmentation | slow vector queries | sev-3; `REINDEX` of the partial hot index | Operator action / scheduled reindex |
| Embedding dim mismatch (gateway returns wrong dim) | sqlx schema check | INSERT fails; sev-2 | Investigate FR-AI-019 / gateway |
| pgvector extension missing | startup `CREATE EXTENSION` fails | Service refuses to start | Operator runs migration |
| RLS violation (cross-tenant write) | postgres 42501 | sev-1 alarm | Investigate worker code |
| Store-ACL denies summary write | FR-MEMORY-117 check | `memory.acl_denied` aux row; summary not written to that subtree | Operator grants `brain-ingest` write on the subtree |
| FR-MEMORY-121 stream lags / stalls | cursor not advancing + lag metric | Recall serves stale (older) evidence; sev-3 on lag SLO breach | Investigate emitters (FR-MEMORY-122) |
| l1_audit_log unreadable | sqlx error | Ingest waits + retries; recall verify fails closed (drops hits) | Restore Layer-1 access |
| Worker attempts to write Layer 1 (bug) | code review + AC #14 invariant | Disallowed by design; caught in test | Author fixes |
| Recall limit > 100 | param check | 400 `limit_too_large` | Caller fixes limit |
| Empty recall result | normal | 200 with `[]` (not 404) | By design |
| Tokio ingest task panic | tracing | Other tenants unaffected; task restarts | Investigate |
| Importance/recency signals absent | feature check | Recall falls back to raw cosine + RRF | By design (soft edge) |
| Spend metric over-counts on retry | charge only on gateway 200 | Pending rows not charged twice | By design |

---

## §11 — Notes

- The worker is the Phase-2 "brain" from `docs/strategy/cyberos-brain-evaluation-plan.md`: it turns the captured interaction log into the persistent, fast-retrieval store described in that note's "Persistent, fast-retrieval memory design" section. It deliberately reuses the Phase-1 substrate (audit chain, pgvector, ai-gateway) rather than re-architecting.
- Layer 1 (`l1_audit_log`) is the system of record; `brain_event_embedding` + `brain_summary` are a derived fast lens. The `--rebuild` path and AC #16 enforce that the lens is reproducible from the chain, so a model swap or index bug is recoverable, never destructive.
- Summaries-first is the cost-and-latency lever: most evaluation recall is answered from a handful of summary rows; raw hot-event search is opt-in (`drill`) or confidence-triggered. The hot HNSW index is partial (`WHERE tier='hot'`) so its size — and every query's cost — is bounded by the hot window, not the lifetime log.
- The access boundary is load-bearing, not cosmetic: tenant RLS at the DB plus the FR-EVAL-001 per-subject predicate at recall, applied as an EXCLUDE (a closest neighbour the caller may not see never appears) with deny-by-default on unknown subjects. This is the technical expression of the strategy note's "access control" and "human in the loop" principles and Vietnam's PDPD purpose-limitation.
- Provenance is what makes the brain defensible: every hit carries a pointer to the exact, hash-chained audit row(s) it came from, so FR-EVAL-003 cites events, not vibes, and a reviewer or the employee can verify each citation against the immutable chain.
- Embeddings and summaries are generated ONLY through the ai-gateway, which pins residency, enforces ZDR, and charges the tenant spend cap (FR-AI-022). Over-cap degrades to `pending_*` with backoff; the worker has no code path that contacts a model provider directly, by design.
- Tiering is age-driven and idempotent. Warm retains the embedding (vector-searchable on drill) but drops out of the hot index; cold keeps only the summary indexed and leaves the raw body in Layer 1 + cold storage, retrievable on demand by `audit_row_id`. Defaults 30d hot / 180d warm are config, not hard-coded policy.
- Summary versioning (supersede, not overwrite) keeps the audit of how a subject/channel summary evolved; recall reads only `superseded_by IS NULL`. Old versions are prunable later without touching the system of record.
- Recall verifies `chain_anchor` per hit exactly as FR-MEMORY-101/108 do; the cost is justified because this evidence may inform pay or an IP dispute, where stale-or-tampered input is unacceptable.
- The `brain-ingest` reserved actor writes subject summaries under an ACL-governed subtree (FR-MEMORY-117); if it lacks write capability there, the write is rejected with a `memory.acl_denied` aux row and the summary is simply not materialised into that subtree — recall still works off the `brain_summary` table.
- Importance (FR-MEMORY-114) and recency (FR-MEMORY-113) are SHOULD-weights so the brain ships before they are wired per tenant; recall degrades gracefully to raw cosine + RRF and improves automatically when the signals are present.
- This FR `blocks` FR-EVAL-003 (the evaluation engine), which consumes `POST /v1/memory/recall` for evidence retrieval and relies on the provenance pointers to cite events. It does NOT touch the rubric (Phase 3) or the human-review workflow (Phase 4) — those are separate FRs.

---

## AI Risk Assessment

- **EU AI Act risk class: limited.** This FR is retrieval infrastructure: it indexes, summarises, and serves an organisation's own interaction records back to authorised internal callers. It does not itself make or automate any decision about a person. It is not a prohibited practice and is not, on its own, a high-risk system under Annex III.
- **Where the risk actually concentrates.** The consequential decisions (performance, progression, pay, employment) live downstream in FR-EVAL-003 + the human-review workflow, which keep a human in the loop (per the strategy note). The brain's contribution to that risk is bounded to two failure modes it must control: (a) leaking one person's record into another's context, and (b) feeding stale or tampered evidence into an assessment. Both are addressed normatively: §1 #8 (access exclude + deny-by-default), §1 #9/#16 (provenance + derivability), §1 #10/#11 (read-time chain verify + Layer-1-wins).
- **Transparency + provenance.** Because every recall hit cites the exact audit row(s) it derived from, an assessment built on this brain is auditable and contestable: a reviewer or the affected employee can trace a claim back to the immutable, hash-chained source. This supports the disclosed-monitoring posture the strategy note requires and Vietnam's PDPD (13/2023/ND-CP) purpose-limitation and data-subject rights; it does not replace counsel's sign-off on the monitoring basis.
- **Data minimisation + residency.** Summaries-first + tiering keep only what recall needs hot; embeddings and summaries are generated in-region under the tenant residency pin and ZDR via the ai-gateway, so employee-interaction text does not leave the residency boundary for embedding.
- **Limits.** The brain must never be treated as the source of truth (it is a derived lens), must never auto-decide anything consequential, and must fail closed on access (unknown subject → deny). These are encoded as `disallowed_tools` and §1 MUST clauses, not left to convention.

---

*End of FR-MEMORY-123. Status: draft (enters draft → ready_to_implement via feature-request-audit).*
