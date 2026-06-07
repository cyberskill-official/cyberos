---
id: FR-TIME-004
title: "TIME auto-detect proposals — Member-confirm suggestions from PROJ activity (status changes + comment patterns + commit/PR activity)"
module: TIME
priority: SHOULD
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CPO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-TIME-001, FR-TIME-002, FR-PROJ-002, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-PROJ-002]
blocks: []

source_pages:
  - website/docs/modules/time.html#auto-detect

source_decisions:
  - DEC-1440 2026-05-17 — Auto-detect engine watches PROJ activity (status transitions, comments, attachments) + (slice 3) Git commits/PRs; proposes TIME entries with confidence score; Member confirms/rejects/edits
  - DEC-1441 2026-05-17 — Closed enum `proposal_source` = {proj_status_change, proj_comment_burst, proj_attachment_added, git_commit, calendar_event}; cardinality 5; slice 1 = proj_* sources only
  - DEC-1442 2026-05-17 — Closed enum `proposal_state` = {pending, accepted, rejected, expired}; cardinality 4
  - DEC-1443 2026-05-17 — Suggestions expire 7 days after creation if Member doesn't act
  - DEC-1444 2026-05-17 — Confidence score 0-100; > 80 = high (auto-fill form); 50-80 = medium (show suggestion); < 50 = filtered out
  - DEC-1445 2026-05-17 — Member always confirms — NEVER auto-creates TIME entry (per feature-request-audit skill §8 — destructive auto-action forbidden)
  - DEC-1446 2026-05-17 — memory audit kinds: time.proposal_generated, time.proposal_accepted, time.proposal_rejected, time.proposal_expired

build_envelope:
  language: rust 1.81
  service: cyberos/services/time/
  new_files:
    - services/time/migrations/0008_time_proposals.sql
    - services/time/src/proposals/mod.rs
    - services/time/src/proposals/detector.rs
    - services/time/src/proposals/proj_activity_watcher.rs
    - services/time/src/proposals/confidence_score.rs
    - services/time/src/proposals/expire_job.rs
    - services/time/src/audit/proposal_events.rs
    - services/time/src/handlers/proposal_routes.rs
    - services/time/tests/proposal_from_status_change_test.rs
    - services/time/tests/proposal_from_comment_burst_test.rs
    - services/time/tests/proposal_member_confirm_test.rs
    - services/time/tests/proposal_member_reject_test.rs
    - services/time/tests/proposal_expire_7d_test.rs
    - services/time/tests/proposal_confidence_filter_test.rs
    - services/time/tests/proposal_no_auto_create_test.rs
    - services/time/tests/proposal_enum_cardinality_test.rs
    - services/time/tests/proposal_audit_emission_test.rs

  modified_files:
    - services/time/src/lib.rs

  allowed_tools:
    - file_read: services/{time,proj}/**
    - file_write: services/time/{src,tests,migrations}/**
    - bash: cd services/time && cargo test proposals

  disallowed_tools:
    - auto-create TIME entry without Member confirm (per DEC-1445)
    - keep proposals past 7d expiry (per DEC-1443)
    - emit proposals < 50 confidence (per DEC-1444)

effort_hours: 6
sub_tasks:
  - "0.4h: 0008_time_proposals.sql + 2 closed enums"
  - "0.4h: proposals/mod.rs"
  - "0.7h: detector.rs (orchestrator)"
  - "0.6h: proj_activity_watcher.rs (NATS subscriber for PROJ events)"
  - "0.4h: confidence_score.rs (heuristics)"
  - "0.3h: expire_job.rs"
  - "0.3h: audit/proposal_events.rs"
  - "0.3h: handlers/proposal_routes.rs"
  - "1.6h: tests — 9 test files"
  - "0.5h: integration smoke"

risk_if_skipped: "Without auto-detect, Members manually start timers + remember activities — known UX gap that causes 15-30% billable hours leakage. Without DEC-1445 Member-confirm safeguard, AI-generated wrong entries silently bill clients. Without DEC-1443 expiry, stale proposals accumulate. Without DEC-1444 confidence filtering, noise drowns signal. The 6h effort is SHOULD priority — UX win not regulatory requirement."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship auto-detect proposal engine at `services/time/src/proposals/` watching PROJ activity, proposing TIME entries with confidence scores, requiring Member confirmation (no auto-create), 7-day expiry, and 4 memory audit kinds.

1. **MUST** define closed `proposal_source` enum: `('proj_status_change','proj_comment_burst','proj_attachment_added','git_commit','calendar_event')` per DEC-1441. Cardinality 5. Slice 2 active: proj_* (3 of 5); git_commit + calendar_event slice 3.

2. **MUST** define closed `proposal_state` enum: `('pending','accepted','rejected','expired')` per DEC-1442. Cardinality 4.

3. **MUST** define `time_proposals` table at migration `0008`: `(proposal_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, member_subject_id UUID NOT NULL, engagement_id UUID NOT NULL, project_id UUID, task_id UUID, source proposal_source NOT NULL, source_ref UUID NOT NULL, suggested_start TIMESTAMPTZ NOT NULL, suggested_end TIMESTAMPTZ NOT NULL, suggested_description TEXT, confidence_score INT NOT NULL CHECK (confidence_score BETWEEN 50 AND 100), state proposal_state NOT NULL DEFAULT 'pending', created_at TIMESTAMPTZ NOT NULL DEFAULT now(), expires_at TIMESTAMPTZ NOT NULL, accepted_entry_id UUID, trace_id CHAR(32))`. RLS scoped to Member.

4. **MUST** subscribe to PROJ NATS events via `proj_activity_watcher.rs`:
   - `proj.issue.status_changed`: fires `proj_status_change` proposal evaluator.
   - `proj.comment.created`: per-issue rolling count → `proj_comment_burst` if > 5 comments in 1h.
   - `proj.attachment.added`: fires `proj_attachment_added`.

5. **MUST** compute confidence via `confidence_score.rs::score(source, context)`:
   - Status change from In-Progress → Done: ~85 (likely user did work).
   - Comment burst: ~70 (probably discussion + work).
   - Attachment added: ~65 (artifact suggests work).
   - Multiple signals same hour: aggregate +10 each (cap 100).
   - User's role on engagement matches issue assignee: +5.
   - Issue assignee = Member: required (else skip).

6. **MUST** filter proposals < 50 confidence per DEC-1444 — never persist.

7. **MUST** propose duration from event timestamps. For status_change: `now() - prior_status_change_at` (capped at 4h). For comment_burst: span of comments (capped at 2h). For attachment: 30min default.

8. **MUST NEVER auto-create TIME entry per DEC-1445. Always requires `POST /v1/time/proposals/{id}/accept` from Member with body `{ duration_seconds_override?, description_override? }`. Handler:
   - Validates Member owns proposal.
   - Creates TIME entry via FR-TIME-001 with Member's overrides applied.
   - Transitions state='accepted'.
   - Emits `time.proposal_accepted` sev-2.

9. **MUST** support reject `POST /v1/time/proposals/{id}/reject` body `{ reason? }`. Transitions state='rejected'. Emits `time.proposal_rejected` sev-3.

10. **MUST** expire pending proposals past `expires_at` per DEC-1443 via daily job. Transition state='expired'. Emit `time.proposal_expired` sev-3.

11. **MUST** expose `GET /v1/time/proposals?state=pending` for Member's pending list.

12. **MUST** emit 4 memory audit kinds per DEC-1446. PII-scrub description via FR-MEMORY-111.

13. **MUST** thread trace_id from PROJ event through proposal creation.

14. **MUST NOT** auto-create entry (DEC-1445).

15. **MUST NOT** emit < 50 confidence (DEC-1444).

---

## §2 — Why this design (rationale)

**Why never auto-create (§1 #8, DEC-1445)?** AI-proposed wrong entries bill clients — fraud risk. Member confirmation is the security gate.

**Why confidence filtering (§1 #6, DEC-1444)?** Low-confidence proposals are noise; Members tune out → all proposals ignored → primitive worthless. 50% threshold = quality filter.

**Why 7-day expiry (§1 #10, DEC-1443)?** Older than a week, Member forgot; proposal is stale; clean up.

**Why slice 2 not slice 1 (SHOULD)?** Not required for billing pipeline; quality-of-life UX. Build after core TIME flow works.

---

## §3 — API contract

```sql
-- 0008_time_proposals.sql
CREATE TYPE proposal_source AS ENUM ('proj_status_change','proj_comment_burst','proj_attachment_added','git_commit','calendar_event');
CREATE TYPE proposal_state AS ENUM ('pending','accepted','rejected','expired');

CREATE TABLE time_proposals (
  proposal_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  member_subject_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  project_id UUID,
  task_id UUID,
  source proposal_source NOT NULL,
  source_ref UUID NOT NULL,
  suggested_start TIMESTAMPTZ NOT NULL,
  suggested_end TIMESTAMPTZ NOT NULL,
  suggested_description TEXT,
  confidence_score INT NOT NULL CHECK (confidence_score BETWEEN 50 AND 100),
  state proposal_state NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  accepted_entry_id UUID,
  reject_reason TEXT,
  trace_id CHAR(32)
);
CREATE INDEX idx_proposals_member_pending
  ON time_proposals(member_subject_id, created_at DESC)
  WHERE state = 'pending';
ALTER TABLE time_proposals ENABLE ROW LEVEL SECURITY;
CREATE POLICY time_proposals_rls ON time_proposals
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND member_subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND member_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON time_proposals FROM cyberos_app;
GRANT UPDATE (state, accepted_entry_id, reject_reason) ON time_proposals TO cyberos_app;
```

Endpoints:
```text
GET    /v1/time/proposals?state=pending
POST   /v1/time/proposals/{id}/accept
POST   /v1/time/proposals/{id}/reject
```

---

## §4 — Acceptance criteria

1. **proposal_source cardinality 5**.
2. **proposal_state cardinality 4**.
3. **Status change generates proposal** — issue done → proposal created with > 50 confidence.
4. **Comment burst threshold** — 6 comments in 1h → proposal.
5. **Accept creates TIME entry** — accept → TIME entry persisted + state=accepted.
6. **Reject transitions** — reject → state=rejected.
7. **Expire 7d** — pending past 7d → expired.
8. **No auto-create** — proposal exists but no TIME entry until accept.
9. **< 50 confidence filtered** — heuristic produces 40 → no row persisted.
10. **Accept respects override** — duration_seconds_override applied to TIME entry.
11. **4 memory audit kinds emitted**.
12. **RLS Member-scoped**.
13. **PII scrub description**.
14. **Trace_id from PROJ event preserved**.
15. **Non-assigned Member skipped** — issue assignee != Member → no proposal.
16. **Multi-source same hour aggregates** — status + comment + attachment within 1h → single high-confidence proposal.
17. **Pending list sorted by created_at desc**.
18. **Slice 2 sources only** — git/calendar sources rejected at insert.
19. **Concurrent accept race** — first wins; second sees state≠pending → 409.
20. **Audit on each transition**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn status_change_to_done_creates_proposal() {
    let ctx = TestContext::with_member_assigned_to_issue().await;
    ctx.publish_proj_event("issue.status_changed", json!({
        "issue_id": ctx.issue_id, "from": "in_progress", "to": "done",
        "assignee_id": ctx.member_id
    })).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM time_proposals WHERE member_subject_id=$1 AND state='pending'")
        .bind(ctx.member_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(count >= 1);
}

#[tokio::test]
async fn accept_creates_time_entry() {
    let ctx = TestContext::with_pending_proposal().await;
    let r = ctx.accept_proposal(ctx.proposal_id, json!({})).await;
    assert_eq!(r.status(), 201);
    let entry_id: Uuid = r.json::<serde_json::Value>().await.unwrap()["entry_id"].as_str().unwrap().parse().unwrap();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM time_entries WHERE entry_id=$1)")
        .bind(entry_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn no_auto_create_without_accept() {
    let ctx = TestContext::with_pending_proposal().await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM time_entries WHERE member_subject_id=$1")
        .bind(ctx.member_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn expire_after_7d() {
    let ctx = TestContext::with_old_pending_proposal(8).await;
    ctx.run_expire_job().await;
    let state: String = sqlx::query_scalar("SELECT state::text FROM time_proposals WHERE proposal_id=$1")
        .bind(ctx.proposal_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state, "expired");
}

// 5.5..5.10: confidence filter, assignee match, audit, cardinality
```

---

## §7 — Dependencies

**Upstream:** FR-PROJ-002 (NATS events to subscribe).
**Cross-module:** FR-TIME-001 (entry create), FR-AI-003, FR-MEMORY-111.

---

## §8 — Example payload

`time.proposal_generated`:
```json
{
  "kind": "time.proposal_generated",
  "severity": 3,
  "tenant_id": "8a2f...",
  "actor_id": "system.time.detector",
  "trace_id": "...",
  "payload": {
    "proposal_id": "0190...",
    "member_subject_id_hash16": "f8a1...",
    "engagement_id": "0190...",
    "source": "proj_status_change",
    "confidence_score": 85,
    "expires_at": "2026-05-24T..."
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Git commit detection — slice 3.
- **Deferred:** Calendar event detection — slice 3.
- **Deferred:** ML confidence scoring (vs heuristics) — slice 3.
- **Deferred:** Bulk-accept (Monday morning batch confirm) — slice 3.
- **Deferred:** Decline-pattern learning (Member rejects X repeatedly → stop proposing) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| PROJ NATS event delivery failure | NATS retry | Proposal not created; sev-3 | Inherent NATS retry |
| Confidence score < 50 | filter | Not persisted | Inherent |
| Member not assignee | check | Skipped | Inherent |
| Concurrent accept race | state check | Second 409 | Inherent |
| Accepted proposal entry creation fails | rollback | State remains pending | Member retries |
| Expired proposal accept attempt | state check | 409 + expired | Inherent |
| Proposal for non-existent issue | FK soft | Created; FK check at accept | Inherent |
| Member rejected then status revives | new proposal generated | Treated independently | Inherent |
| Proposal description PII not scrubbed | FR-MEMORY-111 | Audit dropped; sev-3 | Inherent |
| Confidence > 100 from heuristic bug | CHECK constraint | INSERT fails | Bug fix |
| Multi-source proposals overlap timewise | each independent | Member chooses which to accept | Inherent |
| Pending list grows unbounded | expire job | Daily cleanup | Inherent |
| Trace_id from NATS dropped | propagation guard | Sev-3 if missing | Inherent |
| Slice-3 source attempted | CHECK CHECK on source enum | INSERT rejected | Inherent |
| Cross-tenant proposal injection | RLS | 0 rows visible | Inherent |

---

## §11 — Implementation notes

**§11.1** NATS subscriber consumes PROJ events as fire-and-forget; failures don't block PROJ.

**§11.2** Confidence heuristics tunable via tenant config (slice 3 enhancement).

**§11.3** Expire job runs daily 04:00.

**§11.4** Member assignee check prevents proposals for delegated work.

**§11.5** Accept handler creates entry via FR-TIME-001 standard path — same caps + validations.

**§11.6** Proposal description hashed via FR-MEMORY-111 in audit.

**§11.7** Multi-signal aggregation uses time-window join (1h).

**§11.8** Trace_id forwarded from PROJ event to proposal to accepted entry.

**§11.9** SHOULD priority — graceful degradation; if engine down, manual entry still works.

**§11.10** Per-Member pending count exposed via UI badge.

---

*End of FR-TIME-004 spec.*
