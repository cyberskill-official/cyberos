---
id: TASK-PROJ-009
title: "MEMORY_LINK schema — Issue ↔ memory memory linkage (cites | implements | supersedes) with bidirectional traversal and link-graph queries"
module: PROJ
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-PROJ-001, TASK-PROJ-010, TASK-MEMORY-101, TASK-MEMORY-108]
depends_on: [TASK-PROJ-001]
blocks: [TASK-PROJ-010]

source_pages:
  - website/docs/modules/proj.html#memory-link
source_decisions:
  - DEC-300 (link is a typed edge — cites | implements | supersedes — between Issue and a specific memory memory path or row_id)
  - DEC-301 (links validated at write: target memory MUST exist + caller MUST have read scope per TASK-SKILL-103 allowed_memory_scopes)
  - DEC-302 (supersedes implies forward-only — newer Issue supersedes older memory; cannot supersede future memories)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0009_memory_links.sql
  - services/proj-sync/src/memory_link/mod.rs
  - services/proj-sync/src/memory_link/handlers.rs
  - services/proj/tests/error_mapping_test.rs
allowed_tools:
  - file_read: services/proj-sync/**, services/memory/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test memory_link
disallowed_tools:
  - create link to non-existent memory (per DEC-301)
  - allow `supersedes` link to a future memory row (per DEC-302)

effort_hours: 5
subtasks:
  - "0.5h: 0009_memory_links.sql migration"
  - "0.5h: LinkType enum (cites | implements | supersedes)"
  - "1.0h: validation: memory path exists + scope check"
  - "1.0h: handlers (create / delete / list-by-issue / list-by-memory)"
  - "0.5h: memory audit row 'proj.memory_link_*'"
  - "1.0h: memory_link_test.rs — happy + dangling + scope-denied + bidirectional"
risk_if_skipped: "Without links, an issue describing 'implement memory-recorded decision X' has no machine-traversable connection to X. Citation drift (TASK-PROJ-010) cannot be detected without typed links. 'Supersedes' invariant violations (linking forward in time) corrupt decision-trail semantics. Operators investigating 'which issues cite this memory' need bidirectional traversal — impossible without first-class link table."
---

## §1 — Description (BCP-14 normative)

The MEMORY_LINK layer **MUST** model typed edges between Issues and memory memories. The contract:

1. **MUST** define `memory_links` table: `id UUID PK`, `issue_id UUID FK`, `memory_path TEXT`, `memory_row_id TEXT` (nullable; for non-path linkable rows), `link_type` (`cites|implements|supersedes`), `created_at TIMESTAMPTZ`, `created_by_subject_id UUID`, `removed_at TIMESTAMPTZ` (nullable; soft-delete), `removed_by_subject_id UUID`, `removal_reason TEXT`, `tenant_id UUID`.
2. **MUST** support 3 link types:
    - `cites`: Issue references the memory (no semantic obligation).
    - `implements`: Issue is the concrete work realising the memory's intent (e.g. decision row → impl issue).
    - `supersedes`: Issue makes the memory obsolete; consumers should prefer Issue.
3. **MUST** validate at link-create:
    - `memory_path` MUST exist (calls TASK-MEMORY-108 search; 404 if missing → `Err(LinkError::TargetMissing)`).
    - Caller's frontmatter `allowed_memory_scopes` (TASK-SKILL-103) MUST cover the path; otherwise `Err(LinkError::ScopeDenied)`.
    - For `supersedes`: memory's `created_at_ns < issue.created_at_ns` (forward-only); else `Err(LinkError::SupersedeViolatesTime)`.
4. **MUST** soft-delete via `removed_at + removed_by_subject_id + removal_reason`. Removed rows persist for audit; queries filter by default unless `?include_removed=true`.
5. **MUST** expose REST endpoints:
    - `POST /api/proj/issues/:id/memory-links` — create.
    - `DELETE /api/proj/issues/:id/memory-links/:link_id` with `reason` body — soft-remove.
    - `GET /api/proj/issues/:id/memory-links` — outgoing edges.
    - `GET /api/proj/memory-memories/:path/issues` — incoming edges (bidirectional traversal).
6. **MUST** emit memory audit rows:
    - `proj.memory_link_created` on create.
    - `proj.memory_link_removed` on soft-delete.
7. **MUST** prevent duplicate active links of same `(issue_id, memory_path, link_type)` (one of each type at a time). Same `memory_path` MAY have multiple link types from same issue (e.g. cites AND implements).
8. **MUST** RLS per tenant; the linked memory's tenant_id MUST match issue's tenant_id (cross-tenant links forbidden).
9. **MUST** emit OTel metric `proj_memory_links_total{link_type, outcome}`; outcome ∈ created | removed | denied | dangling.
10. **MUST** support link annotations: an optional `annotation` field (max 500 chars) on link creation explaining the relationship beyond just the type (e.g. cites with annotation "see section 3 specifically"). PII-redacted via TASK-MEMORY-111 before storage.
11. **MUST** support cycle detection on `supersedes` chains: if Issue A supersedes Memory M, and an attempt is made to have Memory M supersede A's parent chain (or similar circular reference), reject with `Err(LinkError::CycleDetected)`. Cycle check runs DFS bounded at depth 100.
12. **MUST** include a `link_strength` field (`weak | medium | strong`) optional, default `medium`. `weak` = passing mention; `strong` = primary citation. Used by TASK-PROJ-010 citation-drift to prioritise alerts.
13. **MUST** support batch link creation: `POST /api/proj/issues/:id/memory-links/batch` with array of links (max 50). Each item validated independently; partial success allowed (per-item status).
14. **MUST** expose link-graph traversal: `GET /api/proj/issues/:id/memory-links/graph?depth=N&types=cites,implements` returns a directed graph (issues + memories as nodes, links as edges) up to N hops. Default depth=2, max=5.
15. **MUST** support link transfer on issue clone/split: when issue is split into multiple sub-issues, operator chooses which sub-issue inherits each link. Default = all links go to first sub-issue; operator can re-route via UI.
16. **MUST** validate `removal_reason` is non-empty for soft-remove: empty/null → 400 `removal_reason_required`.
17. **MUST** support a "rationale" version of cites: `cites_with_quote` link type variant carries an optional `quoted_text` field (the exact passage from the memory being cited). Bounded at 2KB.
18. **MUST** include `metadata` JSONB field for per-tenant extension: arbitrary keys/values for tenant-specific use (e.g. compliance tags, billing references). Open schema; no reserved keys.
19. **MUST** support link "review" state: optionally, links can be marked `review_pending = true` requiring admin approval before they're considered active for traversal queries. Tenant policy `cyberos_proj_tenant_settings.memory_links_require_review` toggles default.
20. **MUST** include `proj.memory_link_traversed` audit row each time a graph-traversal query crosses an edge (for analytics on which links are heavily traversed). Sampled at 10% to avoid audit log spam.

---

## §2 — Why this design (rationale for humans)

**Why three link types (DEC-300)?** Three is the empirical set of useful semantic relations. `cites` = "I'm aware of this"; `implements` = "I'm the concrete realisation"; `supersedes` = "this is now obsolete; use me instead." More types (e.g. `clarifies`, `disputes`) compose from these via tags or future v2.

**Why validate memory exists (DEC-301)?** A link pointing to nothing is worse than no link (gives false confidence). Validation at write-time catches typos + race conditions (memory deleted between user typing and POST).

**Why scope check (DEC-301)?** Cross-tenant link leakage = privacy breach. A user can't link to a memory they shouldn't even know about. Scope check enforces "least authority" — they can only link to memories they could already read.

**Why forward-only supersedes (DEC-302)?** Supersedes implies "this is newer." Linking to a future memory inverts the temporal invariant; downstream tooling assumes supersession chains are monotonic in time.

**Why soft-delete (§1 #4)?** Hard-delete loses the audit trail. Soft-delete = "this was a link, now it's not, here's why." Auditors investigating "why was this link removed" have the answer.

**Why duplicate prevention by `(issue, path, link_type)` not `(issue, path)` (§1 #7)?** Same path may legitimately be both `cites` and `implements` (Issue X is the implementation of decision row Y, AND Issue X cites decision row Y). Allowing both keeps the model expressive.

**Why bidirectional traversal (§1 #5)?** Forward: "what does this issue link to?" Backward: "what issues link to this memory?" The latter is the load-bearing query for citation drift (TASK-PROJ-010): "is this memory referenced anywhere?"

**Why annotation field (§1 #10)?** Pure link type loses nuance; operator's intent is often "cites — see section 3 specifically." Annotation captures the why behind the link.

**Why cycle detection (§1 #11)?** Supersession chains must be acyclic; A supersedes B, B supersedes A creates an undefined "which is current" state. DFS detection prevents at write-time.

**Why link_strength (§1 #12)?** Citation drift alerts should distinguish "this memory is critical to 5 strong-cited issues" from "this memory is weakly mentioned in 50." Operators prioritise by strength.

**Why batch creation (§1 #13)?** Bulk import flows (migrating from another tool) create dozens of links per issue; per-link HTTP is expensive.

**Why graph traversal endpoint (§1 #14)?** UI features (knowledge-graph view, "show me everything this issue is connected to") need multi-hop queries. Depth-bounded prevents runaway.

**Why link transfer on split (§1 #15)?** When an issue is split, links must go somewhere; default-all-to-first is the safe default; operator override handles complex cases.

**Why non-empty removal_reason (§1 #16)?** Soft-remove without a reason loses the "why" for auditors. Empty reason is the same as no audit trail.

**Why cites_with_quote variant (§1 #17)?** Heavy-citation workflows (research, legal) want to preserve the exact text being cited. Hash-of-memory isn't enough; the quote is the cite.

**Why metadata JSONB (§1 #18)?** Per-tenant extension without schema migration; tenant-specific compliance tags fit here.

**Why review-pending state (§1 #19)?** Some tenants require curation of cross-references for quality; opt-in policy supports their workflow.

**Why traversal audit sampling (§1 #20)?** Graph queries are frequent (page loads); audit row per edge would flood the chain. 10% sample preserves analytics signal without flood.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0009_memory_links.sql

CREATE TABLE memory_links (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id              UUID NOT NULL,
    memory_path           TEXT NOT NULL,
    memory_row_id         TEXT,
    link_type             TEXT NOT NULL CHECK (link_type IN ('cites','implements','supersedes')),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_subject_id UUID NOT NULL,
    removed_at            TIMESTAMPTZ,
    removed_by_subject_id UUID,
    removal_reason        TEXT,
    tenant_id             UUID NOT NULL
);
CREATE UNIQUE INDEX uniq_active_memory_link
    ON memory_links (issue_id, memory_path, link_type)
    WHERE removed_at IS NULL;
CREATE INDEX idx_memory_links_by_issue ON memory_links (issue_id) WHERE removed_at IS NULL;
CREATE INDEX idx_memory_links_by_memory ON memory_links (memory_path) WHERE removed_at IS NULL;

ALTER TABLE memory_links ENABLE ROW LEVEL SECURITY;
CREATE POLICY memory_links_tenant_isolation ON memory_links
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust

```rust
// services/proj-sync/src/memory_link/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum LinkType { Cites, Implements, Supersedes }

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct MemoryLink {
    pub id:                    uuid::Uuid,
    pub issue_id:              uuid::Uuid,
    pub memory_path:           String,
    pub memory_row_id:         Option<String>,
    pub link_type:             LinkType,
    pub created_at:            chrono::DateTime<chrono::Utc>,
    pub created_by_subject_id: uuid::Uuid,
    pub removed_at:            Option<chrono::DateTime<chrono::Utc>>,
    pub removed_by_subject_id: Option<uuid::Uuid>,
    pub removal_reason:        Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("target memory does not exist: {0}")]                  TargetMissing(String),
    #[error("scope denied for path: {0}")]                         ScopeDenied(String),
    #[error("supersedes violates time invariant (memory newer than issue)")] SupersedeViolatesTime,
    #[error("duplicate active link of type {0:?}")]                DuplicateActive(LinkType),
    #[error("cross-tenant link forbidden")]                        CrossTenantForbidden,
    #[error("db: {0}")]                                            Db(String),
}

pub async fn create_link(
    pool: &sqlx::PgPool,
    issue_id: uuid::Uuid,
    memory_path: String,
    link_type: LinkType,
    subject_id: uuid::Uuid,
) -> Result<MemoryLink, LinkError> {
    // 1. Validate target exists
    let memory = memory_reader::find_memory(&memory_path).await
        .ok_or_else(|| LinkError::TargetMissing(memory_path.clone()))?;

    // 2. Scope check (caller's frontmatter must cover this path)
    if !subject_can_read_scope(subject_id, &memory_path).await {
        return Err(LinkError::ScopeDenied(memory_path));
    }

    // 3. Cross-tenant check
    let issue: (chrono::DateTime<chrono::Utc>, uuid::Uuid) = sqlx::query_as(
        "SELECT created_at, tenant_id FROM issues WHERE id = $1"
    ).bind(issue_id).fetch_one(pool).await.map_err(|e| LinkError::Db(e.to_string()))?;
    if memory.tenant_id != issue.1 {
        return Err(LinkError::CrossTenantForbidden);
    }

    // 4. Supersede time invariant
    if link_type == LinkType::Supersedes {
        if memory.created_at_ns > issue.0.timestamp_nanos_opt().unwrap_or(i64::MAX) {
            return Err(LinkError::SupersedeViolatesTime);
        }
    }

    // 5. Insert (UNIQUE catches duplicate)
    let row: MemoryLink = sqlx::query_as(
        "INSERT INTO memory_links (issue_id, memory_path, memory_row_id, link_type,
                                   created_by_subject_id, tenant_id)
         VALUES ($1, $2, $3, $4, $5, current_setting('app.tenant_id')::uuid)
         RETURNING *"
    ).bind(issue_id).bind(memory_path.clone()).bind(memory.row_id.clone())
     .bind(link_type).bind(subject_id)
     .fetch_one(pool).await
     .map_err(|e| {
         if e.to_string().contains("uniq_active_memory_link") {
             LinkError::DuplicateActive(link_type)
         } else { LinkError::Db(e.to_string()) }
     })?;

    emit_memory_row("proj.memory_link_created", serde_json::json!({
        "link_id": row.id, "issue_id": issue_id, "memory_path": memory_path,
        "link_type": link_type, "by_subject_id": subject_id,
    })).await;
    metrics::counter!("proj_memory_links_total",
        "link_type" => format!("{link_type:?}"), "outcome" => "created").increment(1);
    Ok(row)
}

pub async fn remove_link(
    pool: &sqlx::PgPool,
    link_id: uuid::Uuid,
    subject_id: uuid::Uuid,
    reason: String,
) -> Result<(), LinkError> {
    sqlx::query(
        "UPDATE memory_links SET removed_at = NOW(), removed_by_subject_id = $1, removal_reason = $2
         WHERE id = $3 AND removed_at IS NULL"
    ).bind(subject_id).bind(reason).bind(link_id)
     .execute(pool).await.map_err(|e| LinkError::Db(e.to_string()))?;
    emit_memory_row("proj.memory_link_removed", serde_json::json!({
        "link_id": link_id, "by_subject_id": subject_id,
    })).await;
    metrics::counter!("proj_memory_links_total",
        "link_type" => "any", "outcome" => "removed").increment(1);
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. **Create cites link** — POST → 201; row in memory_links; type=cites.
2. **Create implements + cites same memory same issue** — both succeed; 2 active rows.
3. **Duplicate active type rejected** — second POST same (issue, path, type) → 409 DuplicateActive.
4. **Dangling target rejected** — POST with non-existent memory_path → 422 TargetMissing.
5. **Scope-denied rejected** — POST to memory outside caller's allowed_memory_scopes → 403 ScopeDenied.
6. **Supersedes forward-only** — memory created after issue → 422 SupersedeViolatesTime.
7. **Cross-tenant link rejected** — link to memory in different tenant → 403 CrossTenantForbidden.
8. **Soft-remove preserves row** — DELETE → row still exists with removed_at + reason set.
9. **Re-create after remove works** — same (issue, path, type) after soft-remove → 201 (new row).
10. **list-by-issue (default) excludes removed** — query → only active rows.
11. **list-by-issue ?include_removed=true** — query → all rows incl. removed.
12. **Bidirectional: list-by-memory** — GET /api/proj/memory-memories/:path/issues → issues linking to it.
13. **memory audit on create + remove** — both events emit corresponding rows.
14. **RLS isolates tenants** — tenant B cannot see tenant A's links.
15. **OTel counter increments per outcome** — counters for created + removed match call counts.
16. **Annotation stored + redacted** — POST with annotation containing email → stored with <EMAIL> redacted (AC for §1 #10).
17. **Cycle detection rejects** — A supersedes B, then attempt B supersedes A → 422 CycleDetected (AC for §1 #11).
18. **link_strength persisted + queryable** — POST with strength=strong → stored; list filter by strength works (AC for §1 #12).
19. **Batch create handles partial** — batch of 50 with 2 invalid → 200 with per-item status; valid ones inserted (AC for §1 #13).
20. **Graph traversal returns N hops** — depth=2 → nodes + edges within 2 hops; depth=10 → 400 (AC for §1 #14).
21. **Link transfer on split** — split issue → links default to first sub-issue; operator can re-route (AC for §1 #15).
22. **Empty removal_reason rejected** — DELETE with empty reason → 400 (AC for §1 #16).
23. **cites_with_quote stores quoted_text** — POST with quote → quote retrievable on GET (AC for §1 #17).
24. **metadata round-trips** — POST with custom JSONB → GET preserves (AC for §1 #18).
25. **review-pending state honoured** — set tenant policy require_review=true; new link starts review_pending=true; graph excludes pending (AC for §1 #19).
26. **Traversal audit sampled at 10%** — 1000 traversals → ~100 audit rows (binomial within 90% CI) (AC for §1 #20).

---

## §5 — Verification

```rust
#[tokio::test]
async fn duplicate_active_rejected() {
    let env = TestEnv::new().await;
    let (issue, mem) = env.setup_link().await;
    let _ = create_link(&env.pool, issue, mem.clone(), LinkType::Cites, env.alice()).await.unwrap();
    let err = create_link(&env.pool, issue, mem, LinkType::Cites, env.alice()).await.unwrap_err();
    assert!(matches!(err, LinkError::DuplicateActive(LinkType::Cites)));
}

#[tokio::test]
async fn supersedes_forward_only() {
    let env = TestEnv::new().await;
    let future_memory = env.create_memory_at(chrono::Utc::now() + chrono::Duration::days(1)).await;
    let issue = env.create_issue_at(chrono::Utc::now()).await;
    let err = create_link(&env.pool, issue, future_memory, LinkType::Supersedes, env.alice()).await.unwrap_err();
    assert!(matches!(err, LinkError::SupersedeViolatesTime));
}

#[tokio::test]
async fn bidirectional_traversal() {
    let env = TestEnv::new().await;
    let memory = env.create_memory().await;
    let issue1 = env.create_issue().await;
    let issue2 = env.create_issue().await;
    let _ = create_link(&env.pool, issue1, memory.clone(), LinkType::Cites, env.alice()).await.unwrap();
    let _ = create_link(&env.pool, issue2, memory.clone(), LinkType::Implements, env.alice()).await.unwrap();

    let outgoing = list_links_by_issue(&env.pool, issue1, false).await.unwrap();
    assert_eq!(outgoing.len(), 1);

    let incoming = list_issues_by_memory(&env.pool, &memory).await.unwrap();
    assert_eq!(incoming.len(), 2);
}

#[tokio::test]
async fn cross_tenant_forbidden() {
    let env_a = TestEnv::for_tenant("A").await;
    let env_b = TestEnv::for_tenant("B").await;
    let memory_a = env_a.create_memory().await;
    let issue_b = env_b.create_issue().await;
    let err = create_link(&env_b.pool, issue_b, memory_a, LinkType::Cites, env_b.alice()).await.unwrap_err();
    assert!(matches!(err, LinkError::CrossTenantForbidden));
}

#[tokio::test]
async fn soft_remove_then_recreate() {
    let env = TestEnv::new().await;
    let (issue, mem) = env.setup_link().await;
    let link = create_link(&env.pool, issue, mem.clone(), LinkType::Cites, env.alice()).await.unwrap();
    remove_link(&env.pool, link.id, env.alice(), "user requested".into()).await.unwrap();
    let link2 = create_link(&env.pool, issue, mem, LinkType::Cites, env.alice()).await.unwrap();
    assert_ne!(link.id, link2.id);
}
```

---

## §6 — Implementation skeleton

(API + DB above.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — issues FK.
- **TASK-PROJ-010 (downstream)** — citation drift uses these links.
- **TASK-MEMORY-101** — MemoryReader for validation.
- **TASK-MEMORY-108** — search API (find_memory).

---

## §8 — Example payloads

```json
{
  "kind": "proj.memory_link_created",
  "payload": {
    "link_id": "lk-...",
    "issue_id": "iss-...",
    "memory_path": "memories/projects/cyberos/decisions/DEC-300.md",
    "link_type": "implements",
    "by_subject_id": "7e57c0de-..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- `disputes` link type — slice 4+.
- Issue↔Issue links (in addition to Issue↔Memory) — slice 4+; sibling FR.
- Auto-link via NLP (extract memory references from issue body) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Memory missing at create | find_memory Err | 422 TargetMissing | Caller verifies path |
| Scope denied | subject_can_read_scope false | 403 | Caller's frontmatter expanded |
| Supersedes future memory | timestamp check | 422 | Caller fixes target |
| Duplicate active | UNIQUE constraint | 409 | Caller removes existing first |
| Cross-tenant | tenant_id mismatch | 403 | None |
| Memory deleted after link created | dangling | Listed in query; TASK-PROJ-010 flags | Operator removes link |
| memory reader unreachable | find_memory Err | 503 | Operator restores memory |
| Audit emit fails | Link created; audit lost | sev-2 | Operator restores |
| Many links per issue (1000+) | unbounded | List query slow | Slice 3+ paginate |
| Removed_by NULL when not removed | NULL-safe queries | Correct | None |
| RLS bypass | RLS policy | 0 rows | None |
| memory_row_id NULL (path-only) | nullable column | Both supported | None |
| Concurrent link create same key | UNIQUE catches | 409 to second | Caller retries |
| Annotation with PII | redacted by TASK-MEMORY-111 | stored redacted | None |
| Cycle of depth > 100 | DFS bounded; returns "indeterminate" | conservative: reject | Operator splits chain |
| link_strength missing on create | default medium | None | None |
| Batch with > 50 items | 413 | Caller batches | None |
| Batch with mixed valid + invalid | partial 200; per-item status | None | Caller retries invalid |
| Graph depth > 5 | 400 | Caller adjusts | None |
| Graph depth=5 with high fan-out | bounded by total nodes 1000 | truncated + warning | Caller narrows |
| Link transfer with all sub-issues already having that link | DuplicateActive on each | partial transfer | Operator |
| cites_with_quote > 2KB | 413 | Caller truncates | None |
| metadata > 10KB | 413 | Caller | None |
| review-pending stuck (admin never approves) | bounded by 30d auto-reject | None | Operator |
| Traversal sample rate misconfigured | metric exposes | tuned via config | Operator |
| Cycle check timeout (very deep chain) | bounded by DFS depth 100 | 408 or 422 | Caller |
| Concurrent split + link create | tx serialises | one wins | None |
| Audit sampling produces incorrect 0% (RNG bug) | property test catches | None | None |
| Graph traversal includes removed links | default excludes | include with ?include_removed | None |
| Annotation crash (regex bug in redactor) | catch_unwind | 500 | Author fixes |

---

## §11 — Implementation notes

- `memory_row_id` is for linking to specific memory audit chain rows (not memory paths). Some memories don't live at canonical paths (e.g. ad-hoc cross-references); row_id supports them.
- `subject_can_read_scope` integrates with TASK-SKILL-103's allowed_memory_scopes when caller is a skill; for direct API callers, use TASK-AUTH-003's tenant scope.
- Forward-only supersedes uses ns timestamps; memory memory's `created_at_ns` is from its audit row (canonical).
- Soft-delete preserves history; TASK-PROJ-008 records the link create/remove as history_event rows pegged to the linked issue.
- The `?include_removed=true` parameter is for auditors; default UI hides removed links.
- Cross-issue links (Issue X links to Issue Y) are NOT in scope here; separate FR.
- For high-traffic memories (e.g. DEC-XXX style decision records), incoming-edges query may need a materialised view; slice 3+.
- Annotation text is PII-redacted before storage (reuses TASK-MEMORY-111 ruleset); raw form NOT retained.
- Cycle detection DFS is bounded at depth 100 because real-world supersession chains rarely exceed 10; depth-100 catches malicious or buggy chains.
- link_strength is operator-typed; we considered auto-computed strength (e.g. based on annotation length, surrounding context) but kept it explicit for clarity.
- Batch create uses parallel validation but serial DB insert to preserve order + per-item status accuracy.
- Graph traversal limits: depth ≤ 5, total nodes ≤ 1000. Beyond either bound, response is truncated with `truncated: true` flag.
- Link transfer on split is a slice-2 operator UI concern; the FR specifies the data model + default behaviour.
- cites_with_quote is a separate link type (not annotation extension) because consumers (legal export) treat quotes differently from annotations.
- metadata JSONB is per-link; tenant-specific extension without table churn.
- review-pending links are stored but excluded from default queries; admin endpoint shows them all.
- Traversal audit sampling at 10% balances analytics signal against audit-row volume; configurable per tenant.
- Cycle detection runs in-memory after fetching the relevant subgraph; bounded by depth × fan-out.
- The 30-day auto-reject for review-pending links prevents indefinite limbo; expired ones become removed with reason "review timeout."
- We chose UUIDs for link ids over auto-increment because cross-tenant ID collisions matter for cross-system references.
- The graph endpoint's `types` filter accepts a comma-separated list of link types to traverse; default = all types.
- For very high-degree memories (>1k incoming links), the `list_issues_by_memory` query uses keyset pagination internally.
- Cycle detection considers only `supersedes` edges (cites/implements don't have temporal direction).
- The annotation's 500-char limit matches TASK-PROJ-002 reason field for consistency.
- batch's 50-item limit chosen because: (a) typical bulk import is < 20; (b) > 50 timeouts become user-facing.

---

*End of TASK-PROJ-009.*
