---
id: FR-PROJ-001
title: "PROJ Issue + Cycle + Engagement schema — RLS + cross-module linkable + status FSM + audit + assignee validation"
module: PROJ
priority: MUST
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CPO)
created: 2026-05-15
shipped: 2026-05-19
memory_chain_hash: pending
related_frs: [FR-PROJ-002, FR-PROJ-003, FR-AUTH-001, FR-AUTH-003, FR-AI-003]
depends_on: [FR-AUTH-001, FR-AUTH-003]
blocks: [FR-PROJ-002, FR-PROJ-004, FR-PROJ-005, FR-PROJ-008, FR-PROJ-009, FR-EMAIL-007, FR-RES-001]

source_pages:
  - website/docs/modules/proj.html#issue-schema
source_decisions:
  - DEC-210 (5-status FSM: triage/todo/doing/review/done; no custom statuses at slice 1)
  - DEC-211 (4-priority enum: urgent/high/normal/low; numeric mapping for sort)
  - DEC-212 (cross-module links via issue_links table; link_type enum extensible)
  - DEC-213 (engagement-scoped — every issue belongs to an engagement; no orphan issues)

language: rust 1.81
service: cyberos/services/proj/
new_files:
  - services/proj/Cargo.toml
  - services/proj/src/lib.rs
  - services/proj/src/handlers/issues.rs
  - services/proj/src/handlers/cycles.rs
  - services/proj/src/handlers/engagements.rs
  - services/proj/src/types.rs
  - services/proj/src/status_fsm.rs
  - services/proj/src/links.rs
  - services/proj/src/audit.rs
  - services/proj/migrations/0001_engagements.sql
  - services/proj/migrations/0002_cycles.sql
  - services/proj/migrations/0003_issues.sql
  - services/proj/migrations/0004_issue_links.sql
  - services/proj/tests/issues_test.rs
  - services/proj/tests/issues_status_fsm_test.rs
  - services/proj/tests/issues_links_test.rs
  - services/proj/tests/issues_rls_test.rs
modified_files:
  - services/auth/src/rls/templates.rs                    # add engagements/cycles/issues/issue_links to TENANT_SCOPED_TABLES
allowed_tools:
  - file_read: services/proj/**, services/auth/src/rls/**
  - file_write: services/proj/**
  - bash: cd services/proj && cargo test
disallowed_tools:
  - allow custom statuses outside the 5-status FSM (per DEC-210)
  - allow orphan issues (no engagement) (per DEC-213)
  - skip memory audit on issue mutations (per §1 #6)
  - bypass RLS (per §1 #8)

effort_hours: 12
sub_tasks:
  - "0.5h: 0001_engagements.sql + 0002_cycles.sql + 0003_issues.sql + 0004_issue_links.sql migrations"
  - "0.5h: types.rs (IssueStatus, IssuePriority, LinkType enums)"
  - "0.5h: status_fsm.rs (legal transitions per §1 #3)"
  - "1.0h: handlers/issues.rs (POST + GET + PATCH + DELETE)"
  - "0.5h: handlers/cycles.rs (POST + GET; ends_at > starts_at validation)"
  - "0.5h: handlers/engagements.rs (POST + GET)"
  - "0.5h: links.rs (bidirectional link insertion)"
  - "0.5h: audit.rs (proj.issue_created, proj.issue_status_changed, proj.issue_assigned, proj.issue_linked)"
  - "0.5h: assignee validation (subject must exist + same tenant)"
  - "0.5h: cycle membership validation (cycle must belong to issue's engagement)"
  - "0.5h: TENANT_SCOPED_TABLES registry update"
  - "1.5h: Tests — happy + RLS + FSM transitions + invalid status + invalid cycle + cross-tenant assignee + bidirectional links"
  - "0.5h: OTel metrics emission"
  - "1.5h: Tests — link types + concurrent mutations + audit row payloads"
risk_if_skipped: "PROJ has no model; downstream FRs (FR-PROJ-002 memory anchoring, FR-PROJ-003 status mutations) have nothing to operate on. Without RLS, cross-tenant data exposure. Without status FSM, illegal transitions corrupt state. Without cross-module links, issues can't be derived from emails/chats — productivity loss."
---

## §1 — Description (BCP-14 normative)

The PROJ service **MUST** model project work as Issues + Cycles + Engagements with cross-module linkability. The schema and contract:

1. **MUST** define 4 tables:
    - `engagements (id, tenant_id, client_id, name, status, started_at, ended_at)`
    - `cycles (id, tenant_id, engagement_id, name, starts_at, ends_at, state)`
    - `issues (id, tenant_id, engagement_id, cycle_id, title, body, status, priority, assignee_subject_id, estimate_hours, created_at, updated_at)`
    - `issue_links (issue_id, linked_to_id, link_type, created_at)`
2. **MUST** support 5 status values per DEC-210: `triage | todo | doing | review | done`. No custom statuses at slice 1.
3. **MUST** enforce status FSM transitions:
    - `triage` → {`todo`, `done` (rejected as not-applicable)}
    - `todo` → {`doing`, `triage` (deferral), `done`}
    - `doing` → {`review`, `todo` (paused), `done`}
    - `review` → {`done`, `doing` (rejected back), `todo` (significant rework)}
    - `done` → {} (terminal; reopen requires explicit `reopen` API)
   Illegal transitions return `400 BAD_REQUEST` with `{"error":"illegal_status_transition","from":"<s>","to":"<t>","allowed":[...]}`.
4. **MUST** support 4 priority values: `urgent | high | normal | low` (numeric mapping for sort: urgent=4, high=3, normal=2, low=1).
5. **MUST** expose REST:
    - `POST /v1/proj/issues` (create)
    - `GET /v1/proj/issues?engagement_id=&cycle_id=&assignee=&status=&limit=` (list with filters)
    - `GET /v1/proj/issues/{id}` (single)
    - `PATCH /v1/proj/issues/{id}` (update; status mutation goes through FSM)
    - `DELETE /v1/proj/issues/{id}` (soft-delete via `status: deleted` flag — reserved status outside the 5; only root-admin)
    - `POST /v1/proj/issues/{id}/links` (add link)
6. **MUST** emit memory audit rows:
    - `proj.issue_created` per POST.
    - `proj.issue_status_changed` per status mutation (payload: old_status, new_status, by_subject_id).
    - `proj.issue_assigned` per assignee change (payload: from_subject_id, to_subject_id).
    - `proj.issue_linked` per link added (payload: source_id, linked_to_id, link_type).
7. **MUST** enforce RLS via FR-AUTH-003 pattern. All 4 tables added to `TENANT_SCOPED_TABLES`. Cross-tenant queries return 0 rows.
8. **MUST** support cross-module links via `link_type` enum:
    - `duplicates`, `blocks`, `blocked_by`, `related`, `derived_from_email_thread`, `derived_from_chat_thread`, `derived_from_meeting_decision`.
9. **MUST** insert `issue_links` rows bidirectionally for symmetric link types: `blocks`/`blocked_by`, `duplicates`/`duplicated_by` (auto-inserted as inverse).
10. **MUST** validate `assignee_subject_id` exists AND belongs to same tenant. Cross-tenant assignee → 400 with `assignee_cross_tenant`.
11. **MUST** validate cycle membership: `issue.cycle_id` MUST belong to `issue.engagement_id`. Cross-engagement cycle → 400 with `cycle_engagement_mismatch`.
12. **MUST** validate `cycle.ends_at > cycle.starts_at`. Reverse → 400.
13. **MUST** support optimistic locking via `If-Match: <updated_at>` header on PATCH. Mismatch → 412 PRECONDITION_FAILED with `concurrent_update`.
14. **SHOULD** emit OTel metrics:
    - `proj_issues_created_total{tenant_id, engagement_id}` (counter).
    - `proj_issues_status_changed_total{from, to}` (counter).
    - `proj_issues_assigned_total` (counter).
    - `proj_issue_links_total{link_type}` (counter).

---

## §2 — Why this design (rationale for humans)

**Why 5-status FSM (DEC-210)?** Custom statuses are a productivity-tool footgun — every team adds 10 statuses, search/filter degrades, reporting becomes inconsistent. 5 statuses cover 90% of real workflows; constraint forces teams to use them consistently. P3 introduces custom labels (separate from status) for additional dimensions.

**Why 4 priorities (DEC-211)?** Same reasoning. urgent/high/normal/low maps to urgent action / next-week / planned / nice-to-have. Numeric mapping for sort.

**Why engagement-scoped (DEC-213)?** An issue without engagement context is a TODO list item, not project work. Forcing engagement attribution makes "what work is happening for which client" answerable directly.

**Why FSM transitions enforced (§1 #3)?** Without enforcement, "doing → triage" is silent backward progress without audit signal. FSM enforcement makes deferrals explicit (audit shows `doing → todo (paused)`).

**Why bidirectional symmetric links (§1 #9)?** A "blocks" relation implicitly creates "blocked by" — without bidirectional storage, queries from one side miss the relation. Auto-insertion preserves the invariant.

**Why optimistic locking (§1 #13)?** Two operators editing the same issue concurrently → last-write-wins silently. With If-Match, second operator gets 412 + diff to review.

**Why audit on every mutation (§1 #6)?** Project work IS the value chain. Audit answers "who decided to mark this done? when? from what state?" — essential for retros + compliance.

**Why cross-tenant assignee blocked (§1 #10)?** Cross-tenant assignment leaks the assignee-tenant's subject into the assigner-tenant's data. RLS catches reads but not writes; explicit validation at API.

**Why cycle-membership validation (§1 #11)?** A cycle from engagement A can't host an issue in engagement B. Without validation, the relationship is corrupt and reports break.

---

## §3 — API contract

### Migrations

```sql
-- services/proj/migrations/0001_engagements.sql
CREATE TABLE engagements (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL,
    client_id   UUID NULL,        -- FK to CRM clients (P1.5)
    name        TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    status      TEXT NOT NULL CHECK (status IN ('active','completed','paused','cancelled')),
    started_at  DATE NOT NULL,
    ended_at    DATE NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE engagements ENABLE ROW LEVEL SECURITY;
ALTER TABLE engagements FORCE ROW LEVEL SECURITY;
CREATE POLICY engagements_isolation ON engagements
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- services/proj/migrations/0002_cycles.sql
CREATE TABLE cycles (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL,
    engagement_id UUID NOT NULL REFERENCES engagements(id) ON DELETE CASCADE,
    name          TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    starts_at     DATE NOT NULL,
    ends_at       DATE NOT NULL CHECK (ends_at > starts_at),
    state         TEXT NOT NULL CHECK (state IN ('upcoming','active','closed')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE cycles ENABLE ROW LEVEL SECURITY;
ALTER TABLE cycles FORCE ROW LEVEL SECURITY;
CREATE POLICY cycles_isolation ON cycles
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- services/proj/migrations/0003_issues.sql
CREATE TABLE issues (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    engagement_id       UUID NOT NULL REFERENCES engagements(id),
    cycle_id            UUID NULL REFERENCES cycles(id),
    title               TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    body                TEXT NULL CHECK (length(coalesce(body, '')) <= 50000),
    status              TEXT NOT NULL CHECK (status IN ('triage','todo','doing','review','done','deleted')),
    priority            TEXT NOT NULL CHECK (priority IN ('urgent','high','normal','low')),
    assignee_subject_id UUID NULL,
    estimate_hours      NUMERIC(6,2) NULL CHECK (estimate_hours IS NULL OR estimate_hours > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX issues_engagement_idx ON issues(tenant_id, engagement_id);
CREATE INDEX issues_assignee_idx ON issues(tenant_id, assignee_subject_id) WHERE assignee_subject_id IS NOT NULL;
CREATE INDEX issues_status_idx ON issues(tenant_id, status);

ALTER TABLE issues ENABLE ROW LEVEL SECURITY;
ALTER TABLE issues FORCE ROW LEVEL SECURITY;
CREATE POLICY issues_isolation ON issues
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- services/proj/migrations/0004_issue_links.sql
CREATE TABLE issue_links (
    issue_id     UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    linked_to_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    link_type    TEXT NOT NULL CHECK (link_type IN ('duplicates','duplicated_by','blocks','blocked_by','related','derived_from_email_thread','derived_from_chat_thread','derived_from_meeting_decision')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (issue_id, linked_to_id, link_type)
);
ALTER TABLE issue_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE issue_links FORCE ROW LEVEL SECURITY;
CREATE POLICY issue_links_isolation ON issue_links
    USING (
        EXISTS(SELECT 1 FROM issues WHERE id = issue_id
               AND tenant_id = current_setting('app.tenant_id', true)::uuid)
    )
    WITH CHECK (
        EXISTS(SELECT 1 FROM issues WHERE id = issue_id
               AND tenant_id = current_setting('app.tenant_id', true)::uuid)
    );
```

### Types

```rust
// services/proj/src/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum IssueStatus { Triage, Todo, Doing, Review, Done, Deleted }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum IssuePriority { Urgent, High, Normal, Low }

impl IssuePriority {
    pub fn numeric(&self) -> u8 {
        match self { Self::Urgent => 4, Self::High => 3, Self::Normal => 2, Self::Low => 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum LinkType {
    Duplicates, DuplicatedBy, Blocks, BlockedBy, Related,
    DerivedFromEmailThread, DerivedFromChatThread, DerivedFromMeetingDecision,
}

impl LinkType {
    pub fn inverse(&self) -> Option<LinkType> {
        match self {
            Self::Blocks => Some(Self::BlockedBy),
            Self::BlockedBy => Some(Self::Blocks),
            Self::Duplicates => Some(Self::DuplicatedBy),
            Self::DuplicatedBy => Some(Self::Duplicates),
            _ => None,
        }
    }
}
```

### Status FSM

```rust
// services/proj/src/status_fsm.rs
pub fn allowed_transitions(from: IssueStatus) -> &'static [IssueStatus] {
    match from {
        IssueStatus::Triage => &[IssueStatus::Todo],
        IssueStatus::Todo   => &[IssueStatus::Doing, IssueStatus::Triage, IssueStatus::Done],
        IssueStatus::Doing  => &[IssueStatus::Review, IssueStatus::Todo, IssueStatus::Done],
        IssueStatus::Review => &[IssueStatus::Done, IssueStatus::Doing, IssueStatus::Todo],
        IssueStatus::Done   => &[],   // terminal
        IssueStatus::Deleted => &[],
    }
}

pub fn validate_transition(from: IssueStatus, to: IssueStatus) -> Result<(), IssueError> {
    if allowed_transitions(from).contains(&to) {
        Ok(())
    } else {
        Err(IssueError::IllegalStatusTransition {
            from, to, allowed: allowed_transitions(from).to_vec(),
        })
    }
}
```

### Handler skeleton

```rust
// services/proj/src/handlers/issues.rs
pub async fn create_issue(
    req: CreateIssueRequest, claims: &Claims, pool: &PgPool, request_id: &str,
) -> Result<Issue, IssueError> {
    // §1 #11: cycle membership
    if let Some(cycle_id) = req.cycle_id {
        validate_cycle_in_engagement(pool, claims.tenant_id, cycle_id, req.engagement_id).await?;
    }
    // §1 #10: assignee validation
    if let Some(assignee) = req.assignee_subject_id {
        validate_assignee_in_tenant(pool, claims.tenant_id, assignee).await?;
    }

    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.tenant_id = $1::text").bind(claims.tenant_id.to_string()).execute(&mut *tx).await?;

    let issue: Issue = sqlx::query_as(
        "INSERT INTO issues (tenant_id, engagement_id, cycle_id, title, body, status, priority, assignee_subject_id, estimate_hours)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
    ).bind(claims.tenant_id).bind(req.engagement_id).bind(req.cycle_id)
     .bind(&req.title).bind(req.body.as_deref())
     .bind(req.status.unwrap_or(IssueStatus::Triage)).bind(req.priority.unwrap_or(IssuePriority::Normal))
     .bind(req.assignee_subject_id).bind(req.estimate_hours)
     .fetch_one(&mut *tx).await?;

    audit::emit_issue_created(&mut tx, &issue, claims.subject_id, request_id).await?;
    tx.commit().await?;
    metrics::issue_created(claims.tenant_id, issue.engagement_id);
    Ok(issue)
}

pub async fn patch_issue(
    issue_id: Uuid, req: PatchIssueRequest, if_match: Option<DateTime<Utc>>,
    claims: &Claims, pool: &PgPool, request_id: &str,
) -> Result<Issue, IssueError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.tenant_id = $1::text").bind(claims.tenant_id.to_string()).execute(&mut *tx).await?;

    let current: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1").bind(issue_id).fetch_optional(&mut *tx).await?.ok_or(IssueError::NotFound)?;

    // §1 #13: optimistic locking
    if let Some(expected_updated_at) = if_match {
        if current.updated_at != expected_updated_at {
            return Err(IssueError::ConcurrentUpdate {
                expected: expected_updated_at, actual: current.updated_at,
            });
        }
    }

    // §1 #3: status FSM
    if let Some(new_status) = req.status {
        if new_status != current.status {
            status_fsm::validate_transition(current.status, new_status)?;
            audit::emit_status_changed(&mut tx, &current, current.status, new_status, claims.subject_id, request_id).await?;
        }
    }

    // §1 #6: assignee change audit
    if let Some(new_assignee) = req.assignee_subject_id {
        if Some(new_assignee) != current.assignee_subject_id {
            validate_assignee_in_tenant(pool, claims.tenant_id, new_assignee).await?;
            audit::emit_assigned(&mut tx, &current, current.assignee_subject_id, Some(new_assignee), claims.subject_id, request_id).await?;
        }
    }

    let updated: Issue = sqlx::query_as(
        "UPDATE issues SET title = COALESCE($1, title), body = $2, status = COALESCE($3, status),
         priority = COALESCE($4, priority), assignee_subject_id = $5,
         estimate_hours = $6, updated_at = NOW() WHERE id = $7 RETURNING *"
    ).bind(req.title).bind(req.body).bind(req.status).bind(req.priority)
     .bind(req.assignee_subject_id).bind(req.estimate_hours).bind(issue_id)
     .fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn create_link(
    issue_id: Uuid, linked_to_id: Uuid, link_type: LinkType,
    claims: &Claims, pool: &PgPool, request_id: &str,
) -> Result<(), IssueError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.tenant_id = $1::text").bind(claims.tenant_id.to_string()).execute(&mut *tx).await?;

    sqlx::query("INSERT INTO issue_links (issue_id, linked_to_id, link_type) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(issue_id).bind(linked_to_id).bind(link_type).execute(&mut *tx).await?;

    // §1 #9: bidirectional for symmetric types
    if let Some(inverse) = link_type.inverse() {
        sqlx::query("INSERT INTO issue_links (issue_id, linked_to_id, link_type) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(linked_to_id).bind(issue_id).bind(inverse).execute(&mut *tx).await?;
    }

    audit::emit_linked(&mut tx, issue_id, linked_to_id, link_type, claims.subject_id, request_id).await?;
    tx.commit().await?;
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. POST issue → 201 + UUID + audit row `proj.issue_created`.
2. PATCH valid status (`triage → todo`) → 200 + audit row `proj.issue_status_changed`.
3. PATCH invalid status (`triage → done`) → 400 with allowed transitions.
4. PATCH `done → todo` (terminal) → 400 (use reopen API instead).
5. PATCH assignee → 200 + audit row `proj.issue_assigned`.
6. Cross-tenant assignee → 400 with `assignee_cross_tenant`.
7. Cycle from different engagement → 400 with `cycle_engagement_mismatch`.
8. Cycle ends_at < starts_at → 400 (DB CHECK).
9. RLS: cross-tenant SELECT returns 0 rows.
10. Bidirectional link: POST `blocks` link from A→B → row exists for A→B (blocks) AND B→A (blocked_by).
11. Cross-module link `derived_from_email_thread` creates issue_links row.
12. Title > 200 chars → 400 (DB CHECK).
13. Body > 50000 chars → 400.
14. Optimistic lock: PATCH with stale `If-Match` → 412 `concurrent_update`.
15. PATCH same status (no change) → 200, no audit row.
16. List filter by engagement_id, cycle_id, assignee, status all work.
17. Soft-delete (`status: deleted`) only by root-admin.

---

## §5 — Verification

```rust
#[tokio::test]
async fn create_issue_emits_audit() {
    let (pool, claims) = test_setup_with_engagement().await;
    let req = CreateIssueRequest {
        engagement_id: claims.engagement_id, title: "Test".into(),
        priority: Some(IssuePriority::Normal), ..Default::default()
    };
    let issue = create_issue(req, &claims.claims, &pool, "req").await.unwrap();
    assert!(memory_test_helper::has_row("proj.issue_created", &issue.id.to_string()).is_some());
}

#[tokio::test]
async fn invalid_status_transition_returns_400() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    let err = patch_issue(issue.id, PatchIssueRequest { status: Some(IssueStatus::Done), ..Default::default() }, None, &claims.claims, &pool, "req").await.expect_err("expected IllegalStatusTransition");
    assert!(matches!(err, IssueError::IllegalStatusTransition { .. }));
}

#[tokio::test]
async fn cross_tenant_assignee_blocked() {
    let (pool, claims_a) = test_setup_with_engagement().await;
    let claims_b = test_setup_with_engagement().await.1;
    let req = CreateIssueRequest {
        engagement_id: claims_a.engagement_id, title: "x".into(),
        assignee_subject_id: Some(claims_b.claims.subject_id), ..Default::default()
    };
    let err = create_issue(req, &claims_a.claims, &pool, "req").await.expect_err("expected AssigneeCrossTenant");
    assert!(matches!(err, IssueError::AssigneeCrossTenant { .. }));
}

#[tokio::test]
async fn cycle_engagement_mismatch_blocked() {
    let (pool, claims) = test_setup().await;
    let eng_a = test_helper::create_engagement(&pool, claims.tenant_id).await;
    let eng_b = test_helper::create_engagement(&pool, claims.tenant_id).await;
    let cycle_b = test_helper::create_cycle(&pool, eng_b).await;
    let req = CreateIssueRequest { engagement_id: eng_a, cycle_id: Some(cycle_b), title: "x".into(), ..Default::default() };
    let err = create_issue(req, &claims, &pool, "req").await.expect_err("expected CycleEngagementMismatch");
    assert!(matches!(err, IssueError::CycleEngagementMismatch { .. }));
}

#[tokio::test]
async fn rls_blocks_cross_tenant_select() {
    let (pool, claims_a) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims_a, IssueStatus::Triage).await;
    let claims_b = test_setup_with_engagement().await.1;

    rls::with_tenant(&pool, claims_b.tenant_id, |tx| async move {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issues").fetch_one(&mut **tx).await.unwrap();
        assert_eq!(count, 0);
    }).await;
}

#[tokio::test]
async fn bidirectional_blocks_link() {
    let (pool, claims) = test_setup_with_engagement().await;
    let a = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    let b = create_test_issue(&pool, &claims, IssueStatus::Triage).await;
    create_link(a.id, b.id, LinkType::Blocks, &claims.claims, &pool, "req").await.unwrap();

    let count_blocks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issue_links WHERE issue_id = $1 AND linked_to_id = $2 AND link_type = 'blocks'")
        .bind(a.id).bind(b.id).fetch_one(&pool).await.unwrap();
    assert_eq!(count_blocks, 1);

    let count_blocked_by: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issue_links WHERE issue_id = $1 AND linked_to_id = $2 AND link_type = 'blocked_by'")
        .bind(b.id).bind(a.id).fetch_one(&pool).await.unwrap();
    assert_eq!(count_blocked_by, 1);
}

#[tokio::test]
async fn optimistic_lock_412_on_stale_if_match() {
    let (pool, claims) = test_setup_with_engagement().await;
    let issue = create_test_issue(&pool, &claims, IssueStatus::Triage).await;

    // First PATCH succeeds
    let _ = patch_issue(issue.id, PatchIssueRequest { title: Some("v2".into()), ..Default::default() }, Some(issue.updated_at), &claims.claims, &pool, "req").await.unwrap();

    // Second PATCH with stale If-Match
    let err = patch_issue(issue.id, PatchIssueRequest { title: Some("v3".into()), ..Default::default() }, Some(issue.updated_at), &claims.claims, &pool, "req").await.expect_err("expected ConcurrentUpdate");
    assert!(matches!(err, IssueError::ConcurrentUpdate { .. }));
}

#[tokio::test]
async fn cycle_ends_at_lt_starts_at_rejected() {
    let (pool, claims) = test_setup().await;
    let eng = test_helper::create_engagement(&pool, claims.tenant_id).await;
    let req = CreateCycleRequest {
        engagement_id: eng, name: "bad".into(),
        starts_at: today(), ends_at: today() - chrono::Duration::days(1), state: "upcoming".into(),
    };
    let err = create_cycle(req, &claims, &pool, "req").await.expect_err("expected DB CHECK");
    // SQL CHECK constraint violation
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-AUTH-001** — tenants exist before engagements created.
- **FR-AUTH-003** — RLS pattern; add 4 PROJ tables to TENANT_SCOPED_TABLES.
- **FR-AI-003** — memory_writer for audit row emission.
- **FR-PROJ-002 (downstream)** — memory decision anchoring uses these structures.
- Crates: `axum`, `sqlx`, `tokio`, `serde`, `chrono`, `uuid`.

---

## §8 — Example payloads

### Create issue

```http
POST /v1/proj/issues HTTP/1.1
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "engagement_id": "eng-...",
  "title": "Implement FR-PROJ-001",
  "body": "Full schema per §3",
  "priority": "high",
  "assignee_subject_id": "subject-stephen-...",
  "estimate_hours": 12
}

→ 201 Created
{ "id": "issue-...", "status": "triage", "priority": "high", "created_at": "...", ... }
```

### Patch status

```http
PATCH /v1/proj/issues/issue-... HTTP/1.1
If-Match: 2026-05-15T14:00:00Z

{ "status": "doing" }

→ 200 OK
```

### Audit row `proj.issue_status_changed`

```json
{
  "kind": "proj.issue_status_changed",
  "payload": {
    "issue_id": "issue-...",
    "old_status": "todo",
    "new_status": "doing",
    "by_subject_id": "subject-stephen-...",
    "request_id": "req_..."
  }
}
```

### Illegal transition response

```http
HTTP/1.1 400 Bad Request
{
  "error": "illegal_status_transition",
  "from": "triage",
  "to": "done",
  "allowed": ["todo"]
}
```

### Cross-module link

```http
POST /v1/proj/issues/issue-1/links HTTP/1.1
{ "linked_to_id": "issue-2", "link_type": "derived_from_email_thread" }

→ 201 Created
```

---

## §9 — Open questions

All resolved. Deferred:
- Custom labels (separate from status) — slice 4+.
- Sub-issues / parent-child — slice 4+.
- Time tracking integration (FR-TIME-001) — slice 3+.
- Estimate vs actual variance reporting — slice 3+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| FK violation (cycle missing) | sqlx error | 400 cycle_not_found | Caller fixes cycle_id |
| Concurrent PATCH | optimistic lock | 412 concurrent_update | Caller refetches + retries |
| Cross-tenant attempt | RLS blocks | 0 rows | By design |
| Illegal status transition | FSM check | 400 with allowed list | Caller picks valid transition |
| Cycle ends_at < starts_at | DB CHECK | 400 | Caller fixes dates |
| Cross-tenant assignee | claims check | 400 assignee_cross_tenant | Caller uses same-tenant subject |
| Cycle from different engagement | validate_cycle_in_engagement | 400 cycle_engagement_mismatch | Caller fixes |
| Title > 200 chars | DB CHECK | 400 | Caller shortens |
| Body > 50000 chars | DB CHECK | 400 | Caller shortens or attaches file |
| Negative estimate_hours | DB CHECK | 400 | Caller fixes |
| Soft-delete by non-root-admin | role check | 403 | Operator escalates |
| Audit row emit fails | memory_writer error | tx rollback; 500 | Operator investigates memory |
| Self-link (issue_id == linked_to_id) | sanity check | 400 self_link | Caller fixes |
| Reopen done issue | terminal status | 400; use reopen API | API to introduce in slice 3 |
| Concurrent link insert (same triple) | UNIQUE | First wins; ON CONFLICT DO NOTHING | By design |
| Concurrent status change | optimistic lock | One wins; other 412 | By design |
| Engagement deleted with cycles | FK ON DELETE CASCADE | Cycles + issues cascade | By design |
| Assignee deleted | FK NOT enforced (subject deletion is rare) | NULL assignee_subject_id allowed | By design |
| RLS bypass via `cyberos_ops` | Sev-2 audit | Auditable | By design |
| Cycle state inconsistent with dates | manual operator action | Sev-3 | Manual fix via API |

---

## §11 — Notes

- 5-status FSM is intentionally constrained — custom statuses degrade tooling consistency.
- Bidirectional link auto-insertion preserves invariant ("if A blocks B, then B is blocked_by A").
- Optimistic locking via If-Match catches concurrent PATCH races.
- Cross-tenant assignee blocked at API + RLS catches at DB if API bypassed.
- Cycle membership validation prevents schema corruption.
- DB CHECKs on title length / body length / estimate sign / cycle dates = defense in depth.
- Audit on every mutation enables full project work history reconstruction.
- Soft-delete (`status: deleted`) preserves history; hard-delete reserved for compliance erasure.
- Engagement-scoped issues (no orphans) keeps "what work for which client" answerable.

---

*End of FR-PROJ-001. Status: draft (10/10 target).*
