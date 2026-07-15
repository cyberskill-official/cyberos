---
id: TASK-PROJ-008
title: "memory audit row per issue mutation — chained to PROJ history_event table with field-level diff and chain_anchor verification"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-001, TASK-PROJ-003, TASK-PROJ-004, TASK-MEMORY-101]
depends_on: [TASK-PROJ-001, TASK-MEMORY-101, TASK-PROJ-004]
blocks: []

source_pages:
  - website/docs/modules/proj.html#audit-trail
source_decisions:
  - DEC-290 (every issue mutation emits a memory audit row + a PROJ history_event row; the two MUST be linked via chain_anchor)
  - DEC-291 (history_event captures field-level diff; memory row carries the memory-chain hash for tamper detection)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0008_history_events.sql
  - services/proj-sync/src/history/mod.rs
  - services/proj-sync/src/history/diff.rs
  - services/proj-sync/tests/history_event_test.rs
modified_files:
  - services/proj-sync/src/scalar_handlers.rs        # call history::emit on every mutation
  - services/proj-sync/src/lifecycle/transitions.rs  # already emits status_changed; this task adds history_event link
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test history
disallowed_tools:
  - emit memory row without corresponding history_event row (per DEC-290 — they're a pair)
  - skip chain_anchor verification on history-event query (per DEC-291)

effort_hours: 5
subtasks:
  - "0.5h: 0008_history_events.sql migration (history_event table + chain_anchor field + indexed by issue_id)"
  - "0.5h: history/mod.rs — public API: emit_mutation(issue_id, field, before, after, by_subject) -> HistoryEvent"
  - "1.0h: diff.rs — field-level diff for JSON values (Y.Text snapshot, scalars, arrays)"
  - "0.5h: link memory row's payload.history_event_id; history_event.chain_anchor = memory row's chain hash"
  - "0.5h: integration into scalar_handlers + status_transitions (every mutating handler calls history::emit)"
  - "1.0h: query API: GET /api/proj/issues/:id/history → list of HistoryEvent with chain_anchor verified"
  - "1.0h: history_event_test.rs — happy + diff correctness + tamper detection"
risk_if_skipped: "Without history events, 'when did this issue's assignee change' requires manual audit-log archaeology. Without chain_anchor, the history table can be tampered without trace (operator deletes a row; no detection). Without field-level diff, the audit row says 'issue updated' (useless) instead of 'estimate: 5 → 8'. Compliance reviewers building project trails need this primitive."
---

## §1 — Description (BCP-14 normative)

The history-event layer **MUST** record every mutation to issue rows AND link to a corresponding memory audit row via chain_anchor. The contract:

1. **MUST** define `history_event` table with columns: `id UUID PK`, `issue_id UUID FK`, `seq BIGSERIAL`, `field TEXT`, `before JSONB`, `after JSONB`, `mutation_kind TEXT` (`scalar_lww | status_transition | crdt_snapshot | comment_added | comment_edited | comment_deleted`), `by_subject_id UUID`, `occurred_at_ns BIGINT`, `chain_anchor TEXT` (SHA-256 hex from memory row), `memory_row_id TEXT`, `tenant_id UUID`.
2. **MUST** emit one history_event per mutation. EVERY mutating handler (status transition, scalar LWW, comment CRUD, snapshot persist) MUST call `history::emit_mutation()` AS PART OF THE SAME TRANSACTION.
3. **MUST** link the history_event ↔ memory row bidirectionally:
    - history_event row carries `memory_row_id` and `chain_anchor`.
    - memory audit row's payload carries `history_event_id` and `history_event_seq`.
    - Operators can pivot from either side.
4. **MUST** compute a per-field diff:
    - Scalars: `before` = prior value JSON; `after` = new value JSON.
    - Y.Text fields (`description`, `comment.body`): `before` = SHA-256 of prior content; `after` = SHA-256 of new content; raw content NOT stored (privacy + size). The Y.Doc snapshots (TASK-PROJ-003) are the canonical text history.
    - Arrays (`labels`, `comments`): `before` and `after` are JSON arrays; diff applied client-side for visualization.
5. **MUST** make history table APPEND-ONLY:
    - DELETE forbidden by RLS policy (only `cyberos_admin` role; never `cyberos_app`).
    - UPDATE forbidden on all fields except none (no field is mutable).
    - INSERT only via `history::emit_mutation()`.
6. **MUST** verify `chain_anchor` matches actual memory-chain hash on query:
    - GET history endpoint walks `chain_anchor` per row → re-fetches memory row → asserts SHA-256(canonical(memory_row_minus_chain) || prev_chain) == chain_anchor.
    - Mismatch → 500 with `{"error":"chain_tampered","issue_id":..,"event_seq":..}`; sev-1 alarm.
7. **MUST** expose `GET /api/proj/issues/:id/history` returning the full timeline ordered by `seq ASC`. Response includes both raw history_event data AND verified chain_anchor flag per row.
8. **MUST** support `?since=<seq>` parameter for incremental loading.
9. **MUST** emit `proj.issue_mutated` memory audit row per mutation with payload `{issue_id, field, mutation_kind, before_hash, after_hash, by_subject_id, history_event_id, history_event_seq, trace_id}`.
10. **MUST** emit OTel metrics:
    - `proj_history_events_total{mutation_kind}` (counter).
    - `proj_chain_anchor_verifications_total{result}` (counter; result ∈ ok | mismatch).
11. **MUST** RLS-enforce (TASK-AUTH-003).
12. **MUST** cache `chain_verified` flag per history_event row for 60s in memory (per request worker) to amortise memory round-trips on hot endpoints (issue page load fetches 50+ history events).
13. **MUST** support a background `chain-anchor-sweep` job (cron, hourly) that walks recent history rows + verifies chain anchors WITHOUT a user-driven query. Mismatches emit SEV-1 audit row `proj.chain_tampered`. This is proactive detection.
14. **MUST** support pagination on history query: `?limit=N&before_seq=M` (descending; reverse-chronological) and `?limit=N&after_seq=M` (ascending; for delta loads). Default limit 50; max 500.
15. **MUST** include `mutator_session_id` field correlating with TASK-PROJ-002 decision_session_id — one PATCH that mutates 3 fields produces 3 history events sharing a session_id.
16. **MUST** support history "summary" endpoint that aggregates per-day per-field counts: `GET /api/proj/issues/:id/history/summary?from=&to=` → `[{date, field, mutation_count}]`. For visual sparklines.
17. **MUST** redact PII in `before`/`after` JSON for scalar fields whose name is in the per-tenant `pii_field_allowlist` (e.g. `assignee_email`, `client_phone`). Redacted form stored; raw form available only to admin via separate audit-fetch endpoint.
18. **MUST** include `mutation_source` field: `web | mobile | api | cli | bridge_import | bulk_admin`. Operators investigating "who/what made this change" need the channel context.
19. **MUST** validate `before` value matches the issue's current state at mutation time (optimistic-concurrency check): if `current_value != provided_before`, emit `before_mismatch` audit + return 409. Prevents lost-update bugs.
20. **MUST** support `?include_session_summary=true` query parameter that adds a per-session aggregation to the response (`sessions: [{session_id, mutations: [...]}]`) so UI can render "Alice made 3 changes in this PATCH."

---

## §2 — Why this design (rationale for humans)

**Why dual-write (DEC-290)?** memory audit rows live in the memory chain (Layer 1 immutable, tenant-cross-cutting); history_event lives in Postgres (queryable, indexed by issue_id, tenant-scoped). The pair gives queryability + immutability — querying history is fast; the memory row is the cryptographic proof.

**Why chain_anchor (DEC-291)?** Without it, a sufficiently-privileged operator could delete a history_event row and the memory audit row separately, leaving no detection. The anchor binds them: deleting the history row leaves a memory row pointing to it (detectable); tampering the memory row breaks the chain hash (detectable per TASK-MEMORY-101).

**Why hash Y.Text content (§1 #4)?** Issue descriptions can contain sensitive content (employee feedback, customer data). Storing raw before/after in history bloats the database AND duplicates the canonical Y.Doc snapshots (TASK-PROJ-003). Hash is sufficient for "did the content change?" — clients fetch the actual text from Y.Doc snapshots.

**Why append-only (§1 #5)?** History tables that allow UPDATE/DELETE become "history-of-history" rabbit holes. Append-only is the universal pattern for audit logs; aligns with memory chain semantics.

**Why query-time chain_anchor verify (§1 #6)?** Catches tampering at query time, not just at write time. Operators investigating "did anyone alter the audit trail" see verification status per row. Mismatch is loud (sev-1).

**Why same transaction (§1 #2)?** If history_event INSERT fails after memory row already emitted, the two diverge silently. Transactional pair = atomicity: both or neither.

**Why 60s cache (§1 #12)?** Issue page load fetches 50+ history rows; without cache, that's 50+ memory round-trips × ~10ms = 500ms page-load delay. 60s TTL is short enough that recent tampering surfaces quickly.

**Why proactive sweep (§1 #13)?** Tampering of an old row may not be detected until a query hits it. Hourly sweep ensures detection latency ≤ 1 hour. Critical for compliance audit posture.

**Why pagination (§1 #14)?** Long-lived issues accumulate thousands of mutations; unbounded fetch is slow + breaks clients. Default 50 + max 500 covers typical UI; before/after_seq enables infinite-scroll patterns.

**Why session_id correlation (§1 #15)?** Operator changes 3 fields in one PATCH; the 3 history events should be visually grouped ("Alice changed status + assignee + estimate"). Session_id is the correlation key.

**Why summary endpoint (§1 #16)?** Sparkline UI ("activity on this issue over last 30 days") needs aggregated counts, not full event detail. Pre-aggregation at server saves N round-trips.

**Why PII redaction (§1 #17)?** Audit trails persist for years; sensitive scalars (employee emails, client phone) shouldn't proliferate. Per-tenant allowlist respects compliance scope.

**Why mutation_source (§1 #18)?** Investigating "this issue's estimate keeps changing"; knowing it came from bulk_admin vs web tells operators where to look (script bug vs operator typo).

**Why optimistic-concurrency check (§1 #19)?** Two operators editing the same field concurrently produce lost updates. Provided `before` value as guard rejects the second.

**Why session summary in query (§1 #20)?** Without server-side aggregation, clients re-aggregate from raw events; inefficient + risk of inconsistency. One pre-aggregated response.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0008_history_events.sql

CREATE TABLE history_event (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id        UUID NOT NULL,
    seq             BIGSERIAL NOT NULL,
    field           TEXT NOT NULL,
    before          JSONB,
    after           JSONB,
    mutation_kind   TEXT NOT NULL CHECK (mutation_kind IN
                    ('scalar_lww','status_transition','crdt_snapshot','comment_added','comment_edited','comment_deleted')),
    by_subject_id   UUID NOT NULL,
    occurred_at_ns  BIGINT NOT NULL,
    chain_anchor    TEXT NOT NULL,
    memory_row_id    TEXT NOT NULL,
    tenant_id       UUID NOT NULL,
    UNIQUE (issue_id, seq)
);
CREATE INDEX idx_history_event_issue ON history_event (issue_id, seq DESC);
CREATE INDEX idx_history_event_by_subject ON history_event (by_subject_id, occurred_at_ns DESC);

ALTER TABLE history_event ENABLE ROW LEVEL SECURITY;
CREATE POLICY history_event_tenant_isolation ON history_event
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- INSERT-only: revoke UPDATE/DELETE from cyberos_app role
REVOKE UPDATE, DELETE ON history_event FROM cyberos_app;
```

### Rust API

```rust
// services/proj-sync/src/history/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum MutationKind {
    ScalarLww, StatusTransition, CrdtSnapshot,
    CommentAdded, CommentEdited, CommentDeleted,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct HistoryEvent {
    pub id:              uuid::Uuid,
    pub issue_id:        uuid::Uuid,
    pub seq:             i64,
    pub field:           String,
    pub before:          Option<serde_json::Value>,
    pub after:           Option<serde_json::Value>,
    pub mutation_kind:   MutationKind,
    pub by_subject_id:   uuid::Uuid,
    pub occurred_at_ns:  i64,
    pub chain_anchor:    String,
    pub memory_row_id:    String,
}

pub async fn emit_mutation(
    tx: &mut sqlx::PgTransaction<'_>,
    issue_id: uuid::Uuid,
    field: &str,
    before: serde_json::Value,
    after: serde_json::Value,
    mutation_kind: MutationKind,
    by_subject_id: uuid::Uuid,
) -> Result<HistoryEvent, HistoryError> {
    let occurred_at_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let before_hash = hash_json(&before);
    let after_hash  = hash_json(&after);

    // 1. Insert history_event with PLACEHOLDER chain_anchor (filled after memory emit)
    let event_id = uuid::Uuid::new_v4();
    let seq: i64 = sqlx::query_scalar(
        "INSERT INTO history_event (id, issue_id, field, before, after, mutation_kind,
                                     by_subject_id, occurred_at_ns, chain_anchor, memory_row_id, tenant_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '', '', current_setting('app.tenant_id')::uuid)
         RETURNING seq"
    ).bind(event_id).bind(issue_id).bind(field).bind(&before).bind(&after)
     .bind(mutation_kind).bind(by_subject_id).bind(occurred_at_ns)
     .fetch_one(&mut **tx).await?;

    // 2. Emit memory row (returns chain_anchor)
    let payload = serde_json::json!({
        "issue_id": issue_id, "field": field, "mutation_kind": mutation_kind,
        "before_hash": before_hash, "after_hash": after_hash,
        "by_subject_id": by_subject_id, "history_event_id": event_id, "history_event_seq": seq,
        "trace_id": current_trace_id(),
    });
    let memory_emit = memory_writer::emit_tx("proj.issue_mutated", payload).await?;

    // 3. Update history_event with the actual chain_anchor + memory_row_id
    sqlx::query(
        "UPDATE history_event SET chain_anchor = $1, memory_row_id = $2 WHERE id = $3"
    ).bind(&memory_emit.chain_anchor).bind(&memory_emit.row_id).bind(event_id)
     .execute(&mut **tx).await?;

    Ok(HistoryEvent {
        id: event_id, issue_id, seq, field: field.into(),
        before: Some(before), after: Some(after),
        mutation_kind, by_subject_id, occurred_at_ns,
        chain_anchor: memory_emit.chain_anchor, memory_row_id: memory_emit.row_id,
    })
}

fn hash_json(v: &serde_json::Value) -> String {
    let canon = serde_json::to_vec(v).unwrap();
    hex::encode(sha2::Sha256::digest(&canon))
}
```

### Query handler

```rust
// services/proj-sync/src/history/query.rs
pub async fn list_history(
    pool: &sqlx::PgPool,
    issue_id: uuid::Uuid,
    since: Option<i64>,
) -> anyhow::Result<Vec<HistoryEventWithVerification>> {
    let rows: Vec<HistoryEvent> = sqlx::query_as(
        "SELECT * FROM history_event
         WHERE issue_id = $1 AND seq > COALESCE($2, 0)
         ORDER BY seq ASC"
    ).bind(issue_id).bind(since).fetch_all(pool).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let verified = verify_chain_anchor(&row).await;
        if !verified {
            metrics::counter!("proj_chain_anchor_verifications_total", "result" => "mismatch").increment(1);
            tracing::error!(issue_id = %issue_id, seq = row.seq, "chain_anchor mismatch");
            // Sev-1 alarm via OBS
        } else {
            metrics::counter!("proj_chain_anchor_verifications_total", "result" => "ok").increment(1);
        }
        out.push(HistoryEventWithVerification { event: row, chain_verified: verified });
    }
    Ok(out)
}

async fn verify_chain_anchor(row: &HistoryEvent) -> bool {
    // Fetch memory row by memory_row_id; recompute chain hash
    match memory_reader::fetch_row(&row.memory_row_id).await {
        Ok(br) => br.chain_anchor() == row.chain_anchor,
        Err(_) => false,  // Row missing = tampered
    }
}
```

---

## §4 — Acceptance criteria

1. **Status transition emits history_event** — issue status change → 1 history_event row with mutation_kind=status_transition.
2. **Scalar LWW emits history_event** — title PATCH → 1 row with mutation_kind=scalar_lww, before/after scalars.
3. **CRDT snapshot emits history_event** — description Y.Doc snapshot → 1 row with mutation_kind=crdt_snapshot; before/after = content hashes (not raw text).
4. **Comment add emits history_event** — POST comment → mutation_kind=comment_added; before=null; after={id, author, hash}.
5. **history_event linked to memory row** — row's `memory_row_id` matches a memory row with `kind="proj.issue_mutated"`.
6. **memory row links back** — memory row's `payload.history_event_id` matches the history_event UUID.
7. **chain_anchor verified** — happy query → all rows `chain_verified: true`.
8. **chain_anchor mismatch detected** — manually tamper a memory row → query returns `chain_verified: false`; metric increments.
9. **Append-only enforced** — DELETE/UPDATE on history_event by cyberos_app role → permission denied.
10. **seq monotonic per issue** — 10 mutations → seq 1..10 consecutive.
11. **Same-transaction pair** — if memory emit fails, history_event also rolls back; observed via DB inspection after forced failure.
12. **`?since=<seq>` returns delta** — query with since=5 → returns rows 6..N.
13. **Y.Text hash, not raw** — description history rows have hash strings (not raw markdown) in before/after.
14. **RLS tenant isolation** — tenant A's history invisible to tenant B.
15. **OTel metric per mutation_kind** — counter `proj_history_events_total{mutation_kind="scalar_lww"}` increments on scalar PATCH.
16. **chain_verified cached for 60s** — second query within 60s avoids re-fetching memory row; cache hit metric increments (AC for §1 #12).
17. **Background sweep detects tamper** — tamper a row at T0; sweep at T+1h emits `proj.chain_tampered` SEV-1 (AC for §1 #13).
18. **Pagination returns ≤ limit** — GET ?limit=50 with 200 events → 50 returned; before_seq cursor included (AC for §1 #14).
19. **Session_id correlates multi-field PATCH** — PATCH changing status+assignee+estimate → 3 history events sharing session_id (AC for §1 #15).
20. **Summary endpoint aggregates per day** — GET /history/summary?from=&to= → [{date, field, count}] (AC for §1 #16).
21. **PII redaction applied** — tenant `pii_field_allowlist: [assignee_email]`; PATCH email → history row shows `<EMAIL>` in before/after; admin endpoint shows raw (AC for §1 #17).
22. **mutation_source captured** — PATCH via API → mutation_source=api; via CLI → mutation_source=cli (AC for §1 #18).
23. **Optimistic-concurrency check enforces** — PATCH with stale `before` value → 409 `before_mismatch`; audit row emitted (AC for §1 #19).
24. **Session summary in response** — GET ?include_session_summary=true → response has sessions array grouping events (AC for §1 #20).

---

## §5 — Verification

```rust
#[tokio::test]
async fn status_transition_emits_history_and_memory_pair() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_in_status(IssueStatus::Todo).await;
    let _ = apply_transition(&env.pool, issue, IssueStatus::InProgress, env.alice(), None).await.unwrap();
    let events: Vec<HistoryEvent> = sqlx::query_as("SELECT * FROM history_event WHERE issue_id = $1")
        .bind(issue).fetch_all(&env.pool).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].mutation_kind, MutationKind::StatusTransition);
    assert!(!events[0].chain_anchor.is_empty());

    let memory_row = env.memory.find_by_id(&events[0].memory_row_id).await.unwrap();
    assert_eq!(memory_row["payload"]["history_event_id"], events[0].id.to_string());
}

#[tokio::test]
async fn description_history_stores_hash_not_raw() {
    let env = TestEnv::new().await;
    let issue = env.create_issue().await;
    env.update_description(issue, "secret content").await;
    let event: HistoryEvent = sqlx::query_as("SELECT * FROM history_event WHERE issue_id = $1 AND field = 'description'")
        .bind(issue).fetch_one(&env.pool).await.unwrap();
    let after = event.after.unwrap();
    let after_str = serde_json::to_string(&after).unwrap();
    assert!(!after_str.contains("secret content"));
    assert!(after_str.contains("\""));   // hash string present
}

#[tokio::test]
async fn chain_anchor_tamper_detected() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_with_2_mutations().await;
    env.memory.tamper_row_for_test(/* the second mutation's memory row */).await;

    let history = list_history(&env.pool, issue, None).await.unwrap();
    let tampered_count = history.iter().filter(|h| !h.chain_verified).count();
    assert!(tampered_count >= 1);
}

#[tokio::test]
async fn append_only_enforced_for_cyberos_app() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_with_mutation().await;
    let res: Result<sqlx::postgres::PgQueryResult, _> = sqlx::query("DELETE FROM history_event WHERE issue_id = $1")
        .bind(issue).execute(env.pool_as("cyberos_app")).await;
    assert!(res.is_err());   // permission denied
}

#[tokio::test]
async fn since_parameter_returns_delta() {
    let env = TestEnv::new().await;
    let issue = env.create_issue_with_n_mutations(10).await;
    let delta = list_history(&env.pool, issue, Some(5)).await.unwrap();
    assert_eq!(delta.len(), 5);
    assert!(delta.iter().all(|h| h.event.seq > 5));
}
```

---

## §6 — Implementation skeleton

(API + DB above.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — issues table FK.
- **TASK-PROJ-003** — CRDT snapshot trigger.
- **TASK-PROJ-004** — status transition handler.
- **TASK-MEMORY-101** — MemoryWriter::emit_tx provides chain_anchor + row_id.

---

## §8 — Example payloads

```json
{
  "kind": "proj.issue_mutated",
  "payload": {
    "issue_id":           "iss-...",
    "field":              "status",
    "mutation_kind":      "status_transition",
    "before_hash":        "9b0e8c5...",
    "after_hash":         "ab12cd...",
    "by_subject_id":      "7e57c0de-...",
    "history_event_id":   "0e3b1a2c-...",
    "history_event_seq":  42,
    "trace_id":           "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Background chain_anchor verification scan (proactive vs query-time) — slice 3+; current is reactive.
- History compaction (after 5 years, keep only summary rows) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| memory emit fails mid-tx | sqlx tx rollback | history_event row not committed | None automatic; caller retries |
| chain_anchor recompute differs | verify_chain_anchor returns false | sev-1 alarm; row marked unverified | Investigate tamper; engage incident response |
| memory row deleted (impossible per AGENTS.md §3.5) | fetch_row Err | sev-1; marked unverified | None automatic |
| history_event DELETE attempted | RLS / role permission | permission denied; logged | None — by design |
| Concurrent mutations same field | LWW handles | each gets distinct seq; both recorded | None |
| Y.Doc snapshot diff-hash collision | impractical (SHA-256) | None | None |
| Query without since returns 10K+ rows | unbounded | Slow query; add pagination | Slice 3+ paginate |
| Tampered chain anchor by privileged role | verify catches at query | Detected; sev-1 | Investigate |
| Multiple history rows for one mutation | shouldn't happen if handlers correct | Audit catches; integrity check | Code review |
| RLS bypass | RLS policy | 0 rows | None |
| OTel exporter unavailable | buffered then dropped | Logged | Restore TASK-OBS-001 |
| Chain reader unreachable | verify returns false (treat as missing) | Marked unverified | Investigate TASK-MEMORY-101 |
| Encrypted body field (future) | hash applies to ciphertext | Hash is over what's stored | By design |
| Cache hit returns stale verified=true for tampered row | 60s window | Detected within 1h via sweep | None — by design |
| Sweep job fails | logged + retry on next cron | None visible until next sweep | Operator investigates |
| Pagination cursor invalid (seq doesn't exist) | empty result | 400 | Caller adjusts |
| session_id missing on PATCH | server generates one | each event has unique session | None |
| Summary endpoint scan-heavy on huge issue | aggregate-as-you-go via cursor | bounded time | None |
| PII field allowlist update | next mutation honours new list | None | None |
| Operator request raw PII via admin endpoint without auth | RBAC enforces | 403 | None |
| mutation_source missing (legacy clients) | default "api" | None | None |
| Optimistic-concurrency provided `before` is stale | 409 | Caller re-reads + retries | None |
| Same session_id reused across PATCHes (bug) | dedup at handler | one prevails; SEV-3 warn | Author fixes |
| Tamper of cache (in-process memory) | cache is process-local | replicated risk | None |
| Sweep finds tamper but memory row missing | treat as tamper | SEV-1 | Operator investigates |
| pii_field_allowlist with regex | not supported (literal names only) | 422 if regex provided | Operator uses literal field names |
| Summary aggregation over > 1y range | timeout | 400 with suggest narrower | Caller narrows |
| include_session_summary with 1000 events | response size > 1MB | truncated; 413 if > 5MB | Caller paginates |

---

## §11 — Implementation notes

- The `emit_mutation` API takes a `&mut sqlx::PgTransaction` to ensure same-tx semantics; callers' handlers are responsible for starting + committing the tx.
- memory emit returns `(chain_anchor, row_id)` synchronously per TASK-MEMORY-101's writer protocol; the history_event UPDATE after emit fills these in.
- `hash_json` uses serde_json's default serialization (no sort_keys); for canonical hashing across runs, callers should pre-canonicalise. The hash is for "did it change," not for cross-system reproducibility.
- Y.Text content hashes use SHA-256 over the encoded Y.Doc update bytes (deterministic via Yjs binary format).
- `cyberos_app` is the runtime SQL role (TASK-AUTH-003); `cyberos_ops` (admin) bypasses the REVOKE.
- Chain verification on every query is O(N) memory reads; for hot endpoints (issue page load), cache verified flags for 60s.
- `proj_chain_anchor_verifications_total{result="mismatch"}` > 0 triggers TASK-OBS-007 sev-1 alert.
- Future: per-field "annotations" (operator's explanation of why a mutation happened) — slice 3+.
- The 60s cache is per-worker (not shared across pods); if pods are sticky-routed per tenant, cache hit rate is high. Cross-pod cache would add Redis dependency without justified benefit.
- Background sweep is intentionally simple: walks `WHERE seq > <last_swept_seq>` per tenant, verifies, advances watermark. Sub-minute completion for typical tenants.
- Pagination cursor (before_seq / after_seq) was chosen over offset because: (a) offset is unstable when concurrent inserts happen; (b) cursor is keyset-based, O(log n) index lookup.
- session_id correlation with TASK-PROJ-002 decision_session_id: both come from the same source (request-scoped header or client-generated); the proj handlers propagate.
- Summary endpoint uses Postgres `generate_series` + JOIN for efficient per-day aggregation; bounded by date range.
- PII field allowlist is per-tenant (different tenants have different sensitivity); literal field names only (no regex) to keep the allowlist auditable.
- mutation_source defaults to "api" for backward compat; explicit values come from request headers (e.g. `X-Cyberos-Source: web`).
- Optimistic-concurrency `before` value is the bug-prevention mechanism; without it, "last-write-wins" loses data silently. With it, the concurrent loser gets a 409 and can re-read.
- The session summary aggregation is computed in Postgres via window functions; bounded by event count.
- We considered storing the full memory row alongside history_event to avoid the memory round-trip on verify, but rejected: duplication = drift risk; the round-trip is fast (cached after first).
- Chain verification result is per-row; partial mismatch in a result set still returns the OK rows (per-row verified flag). Clients decide what to do with mixed results.
- The `proj.chain_tampered` SEV-1 audit row from the sweep includes the offending issue_id, seq, expected_chain, actual_chain — operators have full context.
- We rejected hashing the raw `before`/`after` JSON for scalars too (only Y.Text gets hashed) because scalar diffs are operator-meaningful ("estimate: 5 → 8") and the bytes are small.
- The summary endpoint's date range is bounded at 1 year to keep response time bounded; longer ranges should use TASK-MEMORY-108 search directly.
- The mutator_source label is bounded (6 enum values) for metric cardinality safety.

---

*End of TASK-PROJ-008.*
