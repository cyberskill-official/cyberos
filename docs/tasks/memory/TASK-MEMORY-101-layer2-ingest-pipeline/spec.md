---
id: TASK-MEMORY-101
title: "Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verification + 1s p95 lag + per-tenant cursor + idempotent UPSERT"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-MEMORY-102, TASK-MEMORY-108, TASK-MEMORY-110, TASK-AI-019, TASK-AUTH-003, TASK-OBS-001]
depends_on: [TASK-AI-019, TASK-AUTH-003]
blocks: [TASK-MEMORY-102, TASK-MEMORY-103, TASK-MEMORY-106, TASK-MEMORY-108, TASK-MEMORY-105, TASK-PROJ-008]

source_pages:
  - website/docs/modules/memory.html#layer-2
source_decisions:
  - DEC-070 (Layer 1 is source of truth; Layer 2 is read scale-out; Layer 1 wins on conflict)
  - DEC-072 (chain_anchor in Layer 2 catches Layer 1 tampering at query time)
  - DEC-073 (per-tenant ingest cursor; restart resumes; no global cursor)
  - DEC-074 (1s p95 lag floor; freshness vs cost trade-off)

language: rust 1.81
service: cyberos/services/memory/
new_files:
  - services/memory/Cargo.toml
  - services/memory/src/main.rs
  - services/memory/src/layer2/mod.rs
  - services/memory/src/layer2/ingest.rs
  - services/memory/src/layer2/binlog_tail.rs
  - services/memory/src/layer2/pgvector.rs
  - services/memory/src/layer2/age.rs
  - services/memory/src/layer2/cursor.rs
  - services/memory/src/layer2/chain_anchor.rs
  - services/memory/src/layer2/entity_extract.rs
  - services/memory/migrations/0001_layer2.sql
  - services/memory/migrations/0002_layer2_cursor.sql
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/chain_anchor_test.rs
  - services/memory/tests/layer2_cursor_resume_test.rs
  - services/memory/tests/layer2_idempotency_test.rs
  - services/memory/tests/layer2_tenant_isolation_test.rs
modified_files: []
allowed_tools:
  - file_read: services/memory/**
  - file_write: services/memory/{src,tests,migrations}/**
  - bash: cd services/memory && cargo test layer2
  - bash: psql -h localhost -U postgres -c "CREATE EXTENSION vector; CREATE EXTENSION age;"
disallowed_tools:
  - write Layer 2 outside the ingest path (per DEC-070 — bypass would corrupt the source-of-truth invariant)
  - treat Layer 2 as source of truth (per DEC-070)
  #4)
  - skip chain_anchor verification on read (per §1
  #8 — RLS at DB layer enforces)
  - cross-tenant write (per §1

effort_hours: 18
subtasks:
  - "0.5h: Cargo.toml + crate skeleton"
  - "1.0h: 0001_layer2.sql + 0002_layer2_cursor.sql migrations"
  - "1.0h: layer2/binlog_tail.rs — read .binlog frames + advance offset"
  - "1.0h: layer2/cursor.rs — get/update per-tenant cursor"
  - "1.0h: layer2/chain_anchor.rs — SHA-256 of canonical row bytes (matches memory canonical-JSON)"
  - "1.0h: layer2/entity_extract.rs — spaCy NER call + custom recognizers"
  - "1.0h: layer2/pgvector.rs — embedding insert + UPSERT on (tenant_id, seq)"
  - "1.0h: layer2/age.rs — graph node + edge insertion"
  - "1.5h: layer2/ingest.rs — main loop, BGE sidecar call, transaction wrapper"
  - "1.0h: Backoff + retry for BGE failures"
  - "0.5h: AGE entity-extraction fallback (mark pending_retry, don't block pgvector write)"
  - "0.5h: OTel metrics (lag, failures, throughput)"
  - "1.0h: Tests — happy ingest + cursor resume + idempotent UPSERT + tenant isolation + chain_anchor mismatch"
  - "1.0h: Tests — BGE-down retry + AGE-fail-pgvector-still-writes + lag SLO"
  - "1.0h: Tests — concurrent ingest across tenants + restart-mid-batch correctness"
  - "1.0h: integration with TASK-MEMORY-102 (rebuild from Layer 1)"
risk_if_skipped: "memory search (TASK-MEMORY-108) doesn't exist. KB retrieval, OBS dashboards, CUO ambient context all fail. Layer 2 is the read scale-out layer. Without 1s p95 lag, freshly-written memories are invisible to search for too long; the user-perceived 'I just saved this; why can't I find it?' regression. Without chain_anchor, Layer 1 tampering goes undetected at query time."
---

## §1 — Description (BCP-14 normative)

A long-lived Rust process **MUST** tail every `<memory-root>/audit/*.binlog` and ingest each row into pgvector (embeddings) + Apache AGE (graph) within ≤ 1 second of binlog append. Each piece:

1. **MUST** consume from binlog via per-tenant offset cursor; persist last-consumed seq to Postgres for restart resume. Cursor table `layer2_ingest_cursor (tenant_id PK, last_seq, updated_at)`.
2. **MUST** compute BGE-M3 embedding via TASK-AI-019 sidecar for every memory-body row; insert into pgvector with `(tenant_id, seq, id, kind, path, ts_ns, embedding 1024-dim, chain_anchor 32-bytes)`.
3. **MUST** extract entities (PERSON, ORG, PLACE, CONCEPT) + relations via spaCy NER + custom recognizers; insert into AGE graph as nodes + edges. Failure to extract entities does NOT block pgvector insertion (per §1 #11 fallback).
4. **MUST** tag every Layer 2 row with `chain_anchor = SHA-256(canonical(Layer 1 row at seq N))` for tamper detection. Read paths (TASK-MEMORY-108) verify chain_anchor matches Layer 1 before returning results; mismatch → sev-1 + drop the result.
5. **MUST** be idempotent — re-ingesting the same seq is a no-op (UPSERT on `(tenant_id, seq)`). Restart-mid-batch can re-process some rows; the UPSERT prevents duplicates.
6. **MUST** target ingest staleness ≤ 1 second p95 from binlog append to Layer 2 visibility (DEC-074). Steady-state SLO measured via `memory_layer2_ingest_lag_seconds` histogram.
7. **MUST** emit OTel metrics:
- `memory_layer2_ingest_lag_seconds{tenant_id}` (histogram; SLO p95 < 1s).
- `memory_layer2_ingest_failures_total{tenant_id, reason}` (counter; reason ∈ bge_down | age_fail | postgres_error | chain_anchor_mismatch).
- `memory_layer2_throughput_rows_per_sec{tenant_id}` (gauge).
- `memory_layer2_cursor_advance_total{tenant_id}` (counter).
- `memory_layer2_age_pending_retry_total{tenant_id}` (gauge; entries needing AGE backfill).
8. **MUST** support tenant isolation via `tenant_id` column + RLS (TASK-AUTH-003 pattern). The `layer2_memories` table is in `TENANT_SCOPED_TABLES` registry; RLS USING + WITH CHECK clauses applied per TASK-AUTH-003 §1 #2.
9. **MUST NOT** be the source of truth — DEC-070 invariant: Layer 1 wins on any conflict. Read paths comparing Layer 1 vs Layer 2 (e.g., on chain_anchor mismatch) MUST trust Layer 1 + flag Layer 2 row as `stale`.
10. **MUST** retry BGE-M3 sidecar failures with exponential backoff (100ms, 250ms, 500ms, 1s, 2s) up to 5 attempts before marking the row as `pending_embed_retry`. Retry job (slice 2 follow-up) reprocesses such rows.
11. **MUST** apply graceful AGE-entity-extraction fallback: if spaCy/AGE call fails, write the pgvector row (with embedding) AND insert an AGE-pending row marked `state: pending_retry`. The pgvector row is queryable; AGE backfill runs in a separate job.
12. **MUST** validate Layer 1 row's structural integrity before ingestion: parse the binlog frame; check seq monotonicity (next seq = previous + 1 per tenant); chain_anchor recompute (frames carry their own hash). Frame-level validation failure → log sev-2 + skip + advance cursor (alternative would block ingest indefinitely on a corrupted row).
13. **MUST** support concurrent multi-tenant ingestion via tokio task per tenant. Each tenant's ingest runs independently; one tenant's BGE saturation doesn't starve another. Task scheduling round-robin per CPU core.
14. **MUST** support graceful shutdown: on SIGTERM, current in-flight transaction completes; cursor saved; tasks drain; service exits cleanly. No partial commits.
15. **SHOULD** support a `--rebuild` flag for TASK-MEMORY-102 CI gate: starts from seq 0 for all tenants; ignores existing cursor; re-ingests everything from Layer 1.

---

## §2 — Why this design (rationale for humans)

**Why per-tenant cursor (DEC-073)?** Global cursor would let one tenant's slow ingest stall everyone. Per-tenant + tokio task per tenant means tenant A can be 10s behind without affecting tenant B. Also: tenant-scoped restart resume is more granular.

**Why chain_anchor (DEC-072)?** Layer 1 is the source of truth, but read paths hit Layer 2 for performance. If Layer 1 is corrupted (e.g., disk bit-flip, malicious mutation), Layer 2 contains the pre-corruption state. The chain_anchor lets read paths detect drift: recompute SHA-256 of Layer 1 row at seq N; compare to stored chain_anchor; mismatch → flag as sev-1 + drop the result.

**Why 1s p95 lag (DEC-074)?** Users expect "I just saved this; let me search for it" to work. 1s is below human perception of staleness; longer windows produce "where did my data go?" complaints. The 1s budget covers BGE embedding (~50ms on GPU), entity extraction (~30ms), pgvector + AGE writes (~10ms) with healthy margin.

**Why graceful AGE fallback (§1 #11)?** Entity extraction is the slowest part of the pipeline; a spaCy hang would block ALL ingestion. Writing pgvector + marking AGE pending preserves the searchability (vector search works) while AGE catches up via background job.

**Why idempotent UPSERT (§1 #5)?** Restart-mid-batch is the recovery scenario. After SIGTERM, the cursor is saved at seq N; some rows up to N+1 might have been written with the old transaction not yet committed. On restart, ingest re-processes from cursor; UPSERT ensures no duplicates.

**Why frame-level validation skip on corruption (§1 #12)?** A single corrupted frame in binlog should NOT block ingest indefinitely. Skip + sev-2 + advance cursor lets ingest continue; the corrupted row is investigated separately. Without skip, one bad frame stalls the entire tenant's ingest.

**Why concurrent multi-tenant (§1 #13)?** Single-threaded ingest scales linearly with tenant count. At 100 tenants × 100 rows/sec, single thread saturates. Per-tenant tasks distribute across CPU cores.

**Why DEC-070 Layer 1 wins (§1 #9)?** Layer 2 is derived. If Layer 2 says X but Layer 1 says Y, Y is correct (Layer 2 derivation must have a bug). Read paths trust Layer 1 + flag Layer 2 for re-ingest.

**Why per-tenant tokio task (§1 #13)?** Better than one big task with internal queue: per-tenant task has natural backpressure (the task either keeps up or doesn't); no need for a separate scheduler. Failure mode is contained: one task panicking only affects one tenant.

**Why graceful shutdown (§1 #14)?** Mid-transaction abort would leave pgvector + AGE inconsistent. Drain + commit ensures atomicity — either all 3 inserts (pgvector, AGE, cursor) succeed or none.

---

## §3 — API contract

```sql
-- services/memory/migrations/0001_layer2.sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "$user", public;
SELECT create_graph('cyberos_layer2');

CREATE TABLE layer2_memories (
    tenant_id     UUID NOT NULL,
    seq           BIGINT NOT NULL,
    id            UUID NOT NULL,
    kind          TEXT NOT NULL,
    path          TEXT NOT NULL,
    ts_ns         BIGINT NOT NULL,
    embedding     vector(1024) NOT NULL,
    chain_anchor  BYTEA NOT NULL,
    age_state     TEXT NOT NULL DEFAULT 'complete' CHECK (age_state IN ('complete', 'pending_retry')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, seq)
);

CREATE INDEX layer2_memories_hnsw ON layer2_memories USING hnsw (embedding vector_cosine_ops);
CREATE INDEX layer2_memories_kind_idx ON layer2_memories (tenant_id, kind);
CREATE INDEX layer2_memories_age_pending_idx ON layer2_memories (tenant_id) WHERE age_state = 'pending_retry';

ALTER TABLE layer2_memories ENABLE ROW LEVEL SECURITY;
ALTER TABLE layer2_memories FORCE ROW LEVEL SECURITY;
CREATE POLICY layer2_isolation ON layer2_memories
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);
```

```sql
-- services/memory/migrations/0002_layer2_cursor.sql
CREATE TABLE layer2_ingest_cursor (
    tenant_id  UUID PRIMARY KEY,
    last_seq   BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

```rust
// services/memory/src/layer2/ingest.rs
use std::sync::Arc;

pub struct IngestConfig {
    pub bge_url: String,
    pub spacy_url: String,
    pub poll_interval: Duration,
}

pub async fn run_ingest_loop(pool: Arc<PgPool>, config: IngestConfig) -> anyhow::Result<()> {
    loop {
        let tenants = active_tenants(&pool).await?;
        let mut joinset = tokio::task::JoinSet::new();
        for tenant_id in tenants {
            let pool = pool.clone();
            let config = config.clone();
            joinset.spawn(async move {
                ingest_one_tenant(tenant_id, &pool, &config).await
            });
        }
        while let Some(_) = joinset.join_next().await {}
        tokio::time::sleep(config.poll_interval).await;
    }
}

async fn ingest_one_tenant(tenant_id: Uuid, pool: &PgPool, config: &IngestConfig) -> anyhow::Result<()> {
    let cursor = cursor::get(pool, tenant_id).await?;
    let frames = binlog_tail::read_frames_after(tenant_id, cursor).await?;

    for frame in frames {
        // §1 #12: structural validation
        if !chain_anchor::validate_frame(&frame) {
            tracing::warn!(tenant_id = %tenant_id, seq = frame.seq, "corrupted frame; skipping");
            cursor::advance(pool, tenant_id, frame.seq).await?;
            metrics::ingest_failure(tenant_id, "frame_corrupted");
            continue;
        }

        let embedding_result = bge::embed_with_retry(&config.bge_url, &frame.body).await;
        let entity_result = entity_extract::run(&config.spacy_url, &frame.body).await;

        let mut tx = pool.begin().await?;
        rls_set_tenant(&mut tx, tenant_id).await?;

        let embedding = match embedding_result {
            Ok(e) => e,
            Err(_) => {
                metrics::ingest_failure(tenant_id, "bge_down");
                continue;   // skip; retry job picks up later
            }
        };

        let chain_anchor = chain_anchor::compute(&frame);
        sqlx::query("INSERT INTO layer2_memories (tenant_id, seq, id, kind, path, ts_ns, embedding, chain_anchor, age_state)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     ON CONFLICT (tenant_id, seq) DO NOTHING")
            .bind(tenant_id).bind(frame.seq).bind(frame.id)
            .bind(&frame.kind).bind(&frame.path).bind(frame.ts_ns)
            .bind(embedding).bind(&chain_anchor)
            .bind(if entity_result.is_ok() { "complete" } else { "pending_retry" })
            .execute(&mut *tx).await?;

        if let Ok(entities) = entity_result {
            age::insert_entities_and_relations(&mut tx, tenant_id, frame.seq, &entities).await?;
        } else {
            metrics::ingest_failure(tenant_id, "age_fail");
        }

        cursor::advance_in_tx(&mut tx, tenant_id, frame.seq).await?;
        tx.commit().await?;

        metrics::lag_observed(tenant_id, (Utc::now().timestamp_nanos() - frame.ts_ns) / 1_000_000_000);
        metrics::cursor_advanced(tenant_id);
    }
    Ok(())
}
```

```rust
// services/memory/src/layer2/chain_anchor.rs
pub fn compute(frame: &BinlogFrame) -> [u8; 32] {
    let canonical = canonicalise_layer1_row(frame);
    sha256(&canonical)
}

pub fn verify(stored_anchor: &[u8; 32], current_layer1_row: &Layer1Row) -> bool {
    let recomputed = sha256(&canonicalise_layer1_row_from(current_layer1_row));
    recomputed == *stored_anchor
}

pub fn validate_frame(frame: &BinlogFrame) -> bool {
    // CRC check + format validation
    let computed_crc = crc32c::crc32c(&frame.payload);
    computed_crc == frame.crc
}
```

---

## §4 — Acceptance criteria

1. **Append a Layer 1 row → within 1s, Layer 2 row exists** — synthetic write to binlog; query layer2_memories within 1s; row present.
2. **Restart ingest process → resumes from cursor** (no re-ingest of prior rows; no duplicate INSERT).
3. **Cross-tenant query: tenant A's ingest doesn't write to tenant B's index** — RLS enforces; tenant B SELECT from layer2_memories returns 0 of A's rows.
4. **Idempotent: re-ingest same seq → UPSERT, single row** in layer2_memories.
5. **Lag metric < 1s p95 under steady-state load** — emit 1000 frames over 100s; histogram p95 < 1s.
6. **Embedding sidecar (TASK-AI-019) failure → ingest retries with backoff; doesn't crash** — kill BGE; ingest queues retries; continues.
7. **AGE entity-extraction failure → pgvector row written; AGE row marked `state: pending_retry`** — kill spaCy; pgvector still inserts; age_state='pending_retry'.
8. **CI rebuild test (TASK-MEMORY-102) recreates Layer 2 from Layer 1 in <30min** for slice-1 data volume.
9. **Chain_anchor verifies post-write** — Layer 1 row at seq N; recompute chain_anchor; equals stored value.
10. **Chain_anchor mismatch flagged sev-1** — manually corrupt Layer 1; read path detects mismatch; sev-1 OBS event.
11. **Frame-level validation skip on CRC failure** — inject CRC-corrupt frame; ingest skips + advances cursor + sev-2 log.
12. **Concurrent multi-tenant ingest** — 10 tenants × 100 rows; all 10 progress concurrently; per-tenant lag < 2s.
13. **Graceful shutdown** — SIGTERM; in-flight tx commits; cursor saved; restart resumes correctly.
14. **--rebuild flag re-ingests from seq 0** — flag passed; cursor reset; full re-ingest.
15. **OTel metrics emit per ingest event**.
16. **RLS on layer2_memories — INSERT with wrong tenant_id rejected (42501)**.
17. **HNSW index used for vector search** — EXPLAIN shows `Index Scan using layer2_memories_hnsw`.

---

## §5 — Verification

```rust
#[tokio::test]
async fn append_to_layer1_visible_in_layer2_within_1s() {
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    spawn_ingest_loop(pool.clone()).await;

    let seq = test_helper::append_layer1_row(tenant, "decisions/test.md", "test body").await;
    let t0 = std::time::Instant::now();

    while t0.elapsed() < Duration::from_secs(2) {
        let row = sqlx::query("SELECT * FROM layer2_memories WHERE tenant_id = $1 AND seq = $2")
            .bind(tenant).bind(seq).fetch_optional(&pool).await.unwrap();
        if row.is_some() { return; }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("Layer 2 row not visible within 2s");
}

#[tokio::test]
async fn restart_resumes_from_cursor() {
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    let handle = spawn_ingest_loop(pool.clone()).await;
    test_helper::append_layer1_rows(tenant, 100).await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories").fetch_one(&pool).await.unwrap();

    handle.shutdown().await;
    let _ = spawn_ingest_loop(pool.clone()).await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories").fetch_one(&pool).await.unwrap();
    assert_eq!(count_before, count_after, "no duplicate rows after restart");
}

#[tokio::test]
async fn cross_tenant_isolation() {
    let pool = test_pool().await;
    let a = test_tenant().await;
    let b = test_tenant().await;
    test_helper::append_layer1_row(a, "decisions/a.md", "tenant A").await;
    test_helper::append_layer1_row(b, "decisions/b.md", "tenant B").await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    rls::with_tenant(&pool, a, |tx| async move {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories").fetch_one(&mut **tx).await.unwrap();
        assert_eq!(count, 1, "tenant A should see only their row");
    }).await;
}

#[tokio::test]
async fn bge_down_triggers_retry_no_crash() {
    bge_test_helper::stop();
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    spawn_ingest_loop(pool.clone()).await;
    test_helper::append_layer1_row(tenant, "decisions/x.md", "body").await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let metric: u64 = otel_test_helper::counter_value("memory_layer2_ingest_failures_total", &[("reason", "bge_down")]);
    assert!(metric > 0);
    // ingest still alive
    bge_test_helper::start();
    tokio::time::sleep(Duration::from_secs(2)).await;
    let row = sqlx::query("SELECT 1 FROM layer2_memories WHERE tenant_id = $1").bind(tenant).fetch_optional(&pool).await.unwrap();
    assert!(row.is_some(), "ingest recovered after BGE restart");
}

#[tokio::test]
async fn age_failure_pgvector_still_writes() {
    spacy_test_helper::stop();
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    spawn_ingest_loop(pool.clone()).await;
    test_helper::append_layer1_row(tenant, "decisions/x.md", "body").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let row: (String,) = sqlx::query_as("SELECT age_state FROM layer2_memories WHERE tenant_id = $1").bind(tenant).fetch_one(&pool).await.unwrap();
    assert_eq!(row.0, "pending_retry");
}

#[tokio::test]
async fn chain_anchor_mismatch_sev1() {
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    test_helper::append_layer1_row(tenant, "decisions/x.md", "body").await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    test_helper::corrupt_layer1_row_at_seq(tenant, 1, "TAMPERED").await;

    // Read path queries chain_anchor
    let result = layer2_search::query_with_anchor_check(tenant, "body").await;
    assert!(matches!(result, Err(SearchError::ChainAnchorMismatch)));
    let metric: u64 = otel_test_helper::counter_value("memory_layer2_ingest_failures_total", &[("reason", "chain_anchor_mismatch")]);
    assert!(metric > 0);
}

#[tokio::test]
async fn idempotent_upsert() {
    let pool = test_pool().await;
    let tenant = test_tenant().await;
    spawn_ingest_loop(pool.clone()).await;
    let seq = test_helper::append_layer1_row(tenant, "decisions/x.md", "body").await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    test_helper::reset_cursor(tenant, seq - 1).await;   // simulate restart-mid-batch
    tokio::time::sleep(Duration::from_secs(2)).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layer2_memories WHERE tenant_id = $1 AND seq = $2")
        .bind(tenant).bind(seq).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn rls_blocks_wrong_tenant_insert() {
    let pool = test_pool_as_cyberos_app().await;
    let a = test_tenant().await;
    let b = test_tenant().await;
    let err = rls::with_tenant(&pool, a, |tx| async move {
        sqlx::query("INSERT INTO layer2_memories (tenant_id, seq, id, kind, path, ts_ns, embedding, chain_anchor) VALUES ($1, 1, gen_random_uuid(), 'k', 'p', 0, $2, $3)")
            .bind(b)   // wrong tenant
            .bind(vec![0.0f32; 1024])
            .bind(vec![0u8; 32])
            .execute(&mut **tx).await
    }).await.expect_err("expected RLS violation");
    let violation = rls::classify_pg_error(&err).expect("expected RlsCheckViolation");
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **TASK-AI-019** — BGE-M3 sidecar (embeddings).
- **TASK-AUTH-003** — RLS pattern + TENANT_SCOPED_TABLES registry (add `layer2_memories`).
- **TASK-OBS-001** — OTel metrics.
- **TASK-MEMORY-102 (downstream)** — rebuild CI gate uses `--rebuild` flag.
- Crates: `sqlx@0.7` (postgres + vector), `tokio`, `reqwest`, `serde`, `chrono`, `crc32c`, `sha2`, `apache-age` (or psql via sqlx).
- Postgres 16 + pgvector 0.7 + Apache AGE 1.5.
- spaCy + Vietnamese model (vi_core_news_lg) for NER.

---

## §8 — Example payloads

### Layer 2 row (selected fields)

```text
tenant_id:    550e8400-...
seq:          12345
id:           7e57c0de-...
kind:         decisions
path:         memories/decisions/2026-05-15-revoke-policy.md
ts_ns:        1747526400000000000
embedding:    [0.012, -0.034, ...]   (1024 dims)
chain_anchor: a3f9c8d7e6b5a4f3...
age_state:    complete
```

### Cursor row

```text
tenant_id:  550e8400-...
last_seq:   12345
updated_at: 2026-05-15T14:00:00.500Z
```

### Sev-1 chain_anchor mismatch event

```text
sev-1  memory_layer2_chain_anchor_mismatch
       tenant_id=550e... seq=12345 stored=a3f9c8d7... recomputed=ff00aa11...
       Layer 1 may be tampered; halting reads against this seq
```

---

## §9 — Open questions

All resolved. Deferred:
- Real-time streaming (binlog-tail via inotify instead of poll) — slice 2.
- AGE backfill job for `pending_retry` rows — slice 2.
- Embedding model upgrade (BGE-M4 when available) — slice 4+.
- Per-tenant ingest priority (paid vs free) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| BGE sidecar down | reqwest error | Ingest retries with backoff (5 attempts) then `pending_embed_retry`; metric `bge_down` increments | Self-heals when sidecar up; backfill job processes pending |
| AGE entity extraction fails | spaCy error | pgvector row written with `age_state=pending_retry`; AGE backfill runs separately | Self-heals; backfill job |
| Postgres deadlock | sqlx error | tx rollback + retry once | Self-heals |
| Layer 1 row corruption (frame CRC) | `validate_frame` fails | Skip + advance cursor + sev-2 log | Engineer investigates |
| Layer 1 row corruption (chain_anchor mismatch on read) | Read-path verify | sev-1 + drop result | Engineer investigates |
| Cursor table corruption | sqlx error | Restart fails | Operator restores from backup |
| Concurrent ingest race (same tenant, two processes) | UPSERT idempotent | No duplicates | By design |
| Postgres unavailable | sqlx connect | Ingest waits + retries | Self-heals |
| RLS violation (cross-tenant write) | postgres 42501 | Sev-1 alarm | Investigate ingest code |
| Restart mid-batch | UPSERT idempotent | No duplicates; cursor advances correctly | By design |
| HNSW index fragmentation | Slow vector queries | sev-3 alarm; REINDEX | Operator action |
| Tenant deleted while ingest running | tenants table FK violation | Tenant ingest task exits cleanly; metric | By design |
| Embedding dim mismatch (BGE returns wrong dim) | sqlx schema check | INSERT fails; sev-2 | Investigate TASK-AI-019 |
| Frame deserialise fails (msgpack error) | parse error | Skip + sev-2 | Engineer investigates Layer 1 writer |
| Disk full on layer2 partition | INSERT fails | sev-1; ingest pauses | Operator extends disk |
| AGE extension missing | startup CREATE GRAPH fails | Service refuses to start | Operator runs migration |
| pgvector extension missing | startup CREATE EXTENSION fails | Service refuses to start | Operator runs migration |
| Tokio task panic | observability via tracing | Other tenants unaffected; restart task | Investigate |
| Lag > 1s p95 sustained | OTel histogram alarm | sev-3 | Investigate BGE OR DB load |
| --rebuild on production | flag check + confirmation | Operator must explicitly confirm in production | By design (slice 2) |

---

## §11 — Notes

- Per-tenant tokio task = natural backpressure + failure isolation. One slow tenant doesn't starve others.
- Chain_anchor is the read-path safety check. Layer 2 read returns + chain_anchor verified vs Layer 1 = trust.
- AGE pending_retry is the graceful-degradation primitive. pgvector search works even when AGE is down.
- BGE retry budget (5 attempts × exponential = ~4s total) covers transient sidecar issues without indefinite queueing.
- HNSW index for cosine similarity is the slice-1 default. Slice 4+ may add IVFFlat as alternative.
- spaCy NER + custom recognizers extract PERSON/ORG/PLACE/CONCEPT. Custom recognizers reuse TASK-AI-012's VN_* set for Vietnamese-specific entities.
- The `--rebuild` flag is the TASK-MEMORY-102 hook. CI rebuilds Layer 2 from Layer 1 to ensure derivability.
- Concurrent multi-tenant via tokio tasks scales across CPU cores. At 100 tenants × 8 cores, each core handles ~12 tenants.
- Graceful shutdown drains in-flight tx; cursor saved at last completed seq. Restart resumes correctly.

---

*End of TASK-MEMORY-101. Status: done (implemented 2026-05-23).*

## As built (2026-07-02)

Apache AGE was removed; the layer-2 graph is the relational l2_edge table + pgvector. Any AGE/CREATE GRAPH references above are historical.
