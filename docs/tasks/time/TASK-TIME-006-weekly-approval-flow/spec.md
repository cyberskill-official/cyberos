---
id: TASK-TIME-006
title: "TIME weekly approval flow — Member submit → AM (engagement_admin) review → CFO visibility with auto-lock + bulk-approve + diff view"
module: TIME
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-TIME-002, TASK-TIME-003, TASK-TIME-005, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111, TASK-EMAIL-001]
depends_on: [TASK-TIME-001]
blocks: []

source_pages:
  - website/docs/modules/time.html#approval

source_decisions:
  - DEC-1420 2026-05-17 — Weekly approval cadence — Members submit timesheet by EOD Monday for prior week; AM reviews + approves/rejects by Wednesday EOD; CFO has visibility (no veto unless escalated)
  - DEC-1421 2026-05-17 — Closed enum `timesheet_status` = {open, submitted, approved, rejected, locked}; CI cardinality 5
  - DEC-1422 2026-05-17 — Locked once approved — no further mutation (corrections via TASK-TIME-001 correction_to pattern only)
  - DEC-1423 2026-05-17 — Bulk-approve for AM: approve N timesheets in one action (per-engagement)
  - DEC-1424 2026-05-17 — Diff view on resubmission: AM sees what Member changed since prior submission
  - DEC-1425 2026-05-17 — Auto-lock 14 days post-week-end if no submission (with sev-2 audit + email warning)
  - DEC-1426 2026-05-17 — Email notifications: Submit reminder Monday AM; AM review reminder Wednesday AM; rejection notifies Member
  - DEC-1427 2026-05-17 — memory audit kinds: time.timesheet_submitted, time.timesheet_approved, time.timesheet_rejected, time.timesheet_auto_locked, time.timesheet_bulk_approved

build_envelope:
  language: rust 1.81
  service: cyberos/services/time/
  new_files:
    - services/time/migrations/0005_timesheets.sql
    - services/time/migrations/0006_timesheet_reviews.sql
    - services/time/src/timesheet/mod.rs
    - services/time/src/timesheet/submit.rs
    - services/time/src/timesheet/review.rs
    - services/time/src/timesheet/bulk_approve.rs
    - services/time/src/timesheet/auto_lock_job.rs
    - services/time/src/timesheet/diff.rs
    - services/time/src/audit/timesheet_events.rs
    - services/time/src/handlers/timesheet_routes.rs
    - services/time/tests/timesheet_submit_test.rs
    - services/time/tests/timesheet_am_approve_test.rs
    - services/time/tests/timesheet_am_reject_test.rs
    - services/time/tests/timesheet_resubmit_diff_test.rs
    - services/time/tests/timesheet_bulk_approve_test.rs
    - services/time/tests/timesheet_auto_lock_14d_test.rs
    - services/time/tests/timesheet_status_enum_test.rs
    - services/time/tests/timesheet_locked_immutable_test.rs
    - services/time/tests/timesheet_email_notification_test.rs
    - services/time/tests/timesheet_audit_emission_test.rs

  modified_files:
    - services/time/src/lib.rs
    - services/time/src/entry/create.rs                              # block entries for locked weeks

  allowed_tools:
    - file_read: services/time/**
    - file_write: services/time/{src,tests,migrations}/**
    - bash: cd services/time && cargo test timesheet

  disallowed_tools:
    - mutate locked timesheets (per DEC-1422)
    - skip 14d auto-lock (per DEC-1425)
    - allow non-AM approval (per DEC-1420)

effort_hours: 6
subtasks:
  - "0.4h: 0005_timesheets.sql + 0006_timesheet_reviews.sql + closed enum"
  - "0.4h: timesheet/mod.rs"
  - "0.5h: submit.rs"
  - "0.5h: review.rs (AM approve/reject)"
  - "0.4h: bulk_approve.rs"
  - "0.4h: auto_lock_job.rs"
  - "0.4h: diff.rs"
  - "0.3h: audit/timesheet_events.rs"
  - "0.4h: handlers/timesheet_routes.rs"
  - "1.5h: tests — 10 test files"
  - "0.4h: entry/create.rs block on locked"

risk_if_skipped: "Without approval flow, no governance layer between Member timesheet entries + invoice generation → fraudulent or erroneous hours bill clients directly. Without DEC-1422 lock, approved timesheets get mutated post-hoc → re-billing complexity. Without DEC-1425 auto-lock, abandoned timesheets persist indefinitely. Without DEC-1424 diff view, AM can't tell what Member changed at resubmit → rubber-stamp approval risk. The 6h effort lands the approval-governance primitive."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship weekly approval flow at `services/time/src/timesheet/` with 5-state status FSM, AM review + reject, bulk approval, 14-day auto-lock, diff view on resubmission, email notifications, and 5 memory audit kinds.

1. **MUST** define closed `timesheet_status` enum: `('open','submitted','approved','rejected','locked')` per DEC-1421. Cardinality 5.

2. **MUST** define `timesheets` table at migration `0005`: `(timesheet_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, member_subject_id UUID NOT NULL, engagement_id UUID NOT NULL, week_start_date DATE NOT NULL, week_end_date DATE NOT NULL, status timesheet_status NOT NULL DEFAULT 'open', total_seconds INT NOT NULL DEFAULT 0, billable_seconds INT NOT NULL DEFAULT 0, submitted_at TIMESTAMPTZ, reviewed_at TIMESTAMPTZ, reviewed_by_subject_id UUID, locked_at TIMESTAMPTZ, lock_reason TEXT, trace_id CHAR(32))`. Partial unique `(member_subject_id, engagement_id, week_start_date)`. Append-only at status field via REVOKE per task-audit skill rule 12.

3. **MUST** define `timesheet_reviews` table at migration `0006` for review history: `(id BIGSERIAL PRIMARY KEY, timesheet_id UUID NOT NULL REFERENCES timesheets(timesheet_id), reviewer_subject_id UUID NOT NULL, action TEXT NOT NULL CHECK (action IN ('approved','rejected')), reason TEXT, reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now(), trace_id CHAR(32))`. Append-only.

4. **MUST** enforce RLS scoped to tenant_id; Members see own timesheets, AMs see engagement-scoped, CFO sees all.

5. **MUST** expose Member submit `POST /v1/time/timesheets/{id}/submit`. Handler:
   - Validates Member owns timesheet.
   - Validates status='open' or 'rejected'.
   - Aggregates entries for week → total_seconds, billable_seconds.
   - Transitions status='submitted' + submitted_at.
   - Triggers email to AM via TASK-EMAIL-001.
   - Emits `time.timesheet_submitted` sev-2.

6. **MUST** expose AM approve `POST /v1/time/timesheets/{id}/approve`. Caller has `engagement_admin` role. Handler:
   - Validates status='submitted'.
   - Transitions status='approved' + reviewed_at + reviewed_by.
   - INSERTs review row.
   - Auto-transitions to 'locked' immediately per DEC-1422.
   - Emits `time.timesheet_approved` sev-1.

7. **MUST** expose AM reject `POST /v1/time/timesheets/{id}/reject` body `{ reason }`. Handler:
   - Transitions status='rejected'.
   - INSERTs review row with reason.
   - Triggers email to Member with reason.
   - Emits `time.timesheet_rejected` sev-1.

8. **MUST** expose bulk approve `POST /v1/time/timesheets/bulk-approve` body `{ engagement_id, week_start_date }` per DEC-1423. AM scope. Approves all submitted timesheets matching filter; one memory row + per-timesheet audit. Emits `time.timesheet_bulk_approved` sev-2 + N individual `time.timesheet_approved`.

9. **MUST** expose diff view `GET /v1/time/timesheets/{id}/diff?since=<submission_n>` per DEC-1424. Returns added/removed/modified entries since prior submission for resubmission review.

10. **MUST** auto-lock 14d post-week-end per DEC-1425 via `auto_lock_job.rs`:
    - Daily job: SELECT timesheets WHERE status='open' AND week_end_date < now() - 14d.
    - Transition status='locked' + lock_reason='auto_lock_14d_no_submission'.
    - Triggers email to Member + AM.
    - Emits `time.timesheet_auto_locked` sev-2.

11. **MUST** block entry writes for locked weeks per DEC-1422. TASK-TIME-001 entry/create.rs modified to check `timesheets.status` for the week; locked → 412 + `week_locked`.

12. **MUST** trigger emails per DEC-1426 via TASK-EMAIL-001:
    - Monday 09:00: reminder to Members with unsubmitted previous week.
    - Wednesday 09:00: reminder to AMs with pending reviews.
    - On rejection: Member notified with reason.
    - On auto-lock: Member + AM notified.

13. **MUST** emit 5 memory audit kinds per DEC-1427. PII-scrub reason via TASK-MEMORY-111.

14. **MUST** thread trace_id end-to-end.

15. **MUST NOT** allow mutation of approved/locked timesheets per DEC-1422.

16. **MUST NOT** allow non-AM approval per DEC-1420 (cfo has visibility only; no approval power).

---

## §2 — Why this design (rationale)

**Why weekly cadence (§1 #1, DEC-1420)?** Industry standard; matches client billing cadence; recent enough that Members remember what they did; not so frequent it's overhead.

**Why immediate auto-lock on approve (§1 #6, DEC-1422)?** Approved = ready-for-invoice. Lock prevents drift. Corrections via correction_to pattern (TASK-TIME-001 derivative).

**Why 14d auto-lock (§1 #10, DEC-1425)?** Forgotten timesheets pollute reporting. 14 days = enough for Member to catch up; not so long that quarter-close blows up.

**Why bulk approve (§1 #8, DEC-1423)?** AM reviewing 20 Members weekly = 20 individual clicks. Bulk per-engagement reduces to 1 click + spot-check view.

**Why diff view (§1 #9, DEC-1424)?** Resubmission after rejection — AM needs to see what changed, not re-review from scratch. Standard pattern (Git PR diff).

---

## §3 — API contract

```sql
-- 0005_timesheets.sql
CREATE TYPE timesheet_status AS ENUM ('open','submitted','approved','rejected','locked');

CREATE TABLE timesheets (
  timesheet_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  member_subject_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  week_start_date DATE NOT NULL,
  week_end_date DATE NOT NULL,
  status timesheet_status NOT NULL DEFAULT 'open',
  total_seconds INT NOT NULL DEFAULT 0,
  billable_seconds INT NOT NULL DEFAULT 0,
  submitted_at TIMESTAMPTZ,
  reviewed_at TIMESTAMPTZ,
  reviewed_by_subject_id UUID,
  locked_at TIMESTAMPTZ,
  lock_reason TEXT,
  trace_id CHAR(32)
);
CREATE UNIQUE INDEX uniq_timesheet_member_week
  ON timesheets(member_subject_id, engagement_id, week_start_date);
ALTER TABLE timesheets ENABLE ROW LEVEL SECURITY;
CREATE POLICY timesheets_rls ON timesheets
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON timesheets FROM cyberos_app;
GRANT UPDATE (status, total_seconds, billable_seconds, submitted_at, reviewed_at,
              reviewed_by_subject_id, locked_at, lock_reason) ON timesheets TO cyberos_app;

-- 0006_timesheet_reviews.sql
CREATE TABLE timesheet_reviews (
  id BIGSERIAL PRIMARY KEY,
  timesheet_id UUID NOT NULL REFERENCES timesheets(timesheet_id),
  reviewer_subject_id UUID NOT NULL,
  action TEXT NOT NULL CHECK (action IN ('approved','rejected')),
  reason TEXT,
  reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
ALTER TABLE timesheet_reviews ENABLE ROW LEVEL SECURITY;
CREATE POLICY timesheet_reviews_rls ON timesheet_reviews
  USING (timesheet_id IN (SELECT timesheet_id FROM timesheets WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (timesheet_id IN (SELECT timesheet_id FROM timesheets WHERE tenant_id = current_setting('auth.tenant_id')::uuid));
REVOKE UPDATE, DELETE ON timesheet_reviews FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/time/timesheets/{id}/submit
POST   /v1/time/timesheets/{id}/approve              (engagement_admin)
POST   /v1/time/timesheets/{id}/reject               (engagement_admin)
POST   /v1/time/timesheets/bulk-approve              (engagement_admin)
GET    /v1/time/timesheets/{id}/diff?since=N
GET    /v1/time/timesheets/pending                    (engagement_admin)
GET    /v1/time/timesheets/mine                       (member)
```

---

## §4 — Acceptance criteria

1. **timesheet_status cardinality 5**.
2. **Submit transitions** — open → submitted; email sent.
3. **Approve locks** — submitted → approved → locked immediate.
4. **Reject + reason** — submitted → rejected; Member emailed.
5. **Resubmit** — rejected → submitted; diff view shows changes.
6. **Bulk approve** — 5 submitted in engagement → all → approved.
7. **Auto-lock 14d** — week_end+14d + status=open → locked.
8. **Locked entries blocked** — entry write for locked week → 412.
9. **AM-only approval** — Member tries to approve own → 403.
10. **5 memory audit kinds emitted**.
11. **Email cadence Monday/Wednesday** — scheduled jobs verified.
12. **CFO read-only** — CFO can view but POST approve → 403.
13. **Trace_id end-to-end**.
14. **RLS cross-tenant denied**.
15. **PII scrub reason**.
16. **Concurrent submit race** — first wins; second 409.
17. **Approved correction via correction_to** — works (TASK-TIME-001 path).
18. **Diff includes added + removed + modified** — entry diffs accurate.
19. **Pending list ordered by week_end** — oldest first.
20. **Bulk approve idempotent on already-approved** — skips, no error.

---

## §5 — Verification

```rust
#[tokio::test]
async fn submit_then_approve_locks() {
    let ctx = TestContext::with_timesheet().await;
    ctx.submit_timesheet(ctx.ts_id).await;
    ctx.as_am().approve_timesheet(ctx.ts_id).await;
    let status: String = sqlx::query_scalar("SELECT status::text FROM timesheets WHERE timesheet_id=$1")
        .bind(ctx.ts_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "locked");
}

#[tokio::test]
async fn reject_with_reason_emails_member() {
    let ctx = TestContext::with_submitted_timesheet().await;
    ctx.as_am().reject_timesheet(ctx.ts_id, "missing project codes").await;
    let emails = ctx.sent_emails_to(ctx.member_id).await;
    assert!(emails.iter().any(|e| e.body.contains("missing project codes")));
}

#[tokio::test]
async fn auto_lock_after_14d() {
    let ctx = TestContext::with_open_timesheet_week_ago(15).await;
    ctx.run_auto_lock_job().await;
    let status: String = sqlx::query_scalar("SELECT status::text FROM timesheets WHERE timesheet_id=$1")
        .bind(ctx.ts_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "locked");
}

#[tokio::test]
async fn locked_week_blocks_entry_write() {
    let ctx = TestContext::with_locked_timesheet().await;
    let r = ctx.create_entry_for_week(ctx.member_id, ctx.locked_week_date).await;
    assert_eq!(r.status(), 412);
}

// 5.5..5.10: bulk, diff, AM-only, audit, race, CFO read-only
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-001 (entries to aggregate).
**Cross-module:** TASK-AUTH-101 (engagement_admin role), TASK-EMAIL-001 (notifications), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.timesheet_approved`:
```json
{
  "kind": "time.timesheet_approved",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "user.engagement_admin.789",
  "trace_id": "...",
  "payload": {
    "timesheet_id": "0190...",
    "member_subject_id_hash16": "f8a1...",
    "engagement_id": "0190...",
    "week_start_date": "2026-05-10",
    "total_seconds": 144000,
    "billable_seconds": 130000
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-week submission (catch-up) — slice 2.
- **Deferred:** Auto-approve for trusted Members (zero-rejection track record) — slice 3.
- **Deferred:** Variance flags (week deviates > 20% from history) — slice 3.
- **Deferred:** Mobile push notifications — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Submit before week-end | date check | 400 + early_submission | Wait for week end |
| Approve without AM role | role check | 403 | Inherent |
| Reject without reason | validation | 400 | Inherent |
| Email send fails | TASK-EMAIL-001 retry | Audit logged; user notified | Inherent retry |
| Concurrent submit + auto-lock race | tx isolation; partial unique catches | Submit wins if before lock | Inherent |
| Bulk approve includes non-submitted | filter on status='submitted' | Skipped silently | Inherent |
| Bulk approve cross-engagement | scope check | 403 if non-AM | Inherent |
| Auto-lock job crashes | watchdog | Sev-2; manual lock CLI | Operator runs job manually |
| Locked week with pending corrections | correction_to path | Allowed; preserves audit | TASK-TIME-001 derivative |
| Member resubmits after lock | status check | 412 + week_locked | New entries via correction |
| AM tries to approve own timesheet | role + self check | 403 | Inherent |
| Submitted timesheet with 0 entries | allowed (zero-hours week) | Approved if AM ok | Inherent |
| Diff view across multiple resubmissions | versioned reviews | Last-submission diff | Inherent |
| Tenant timezone affects week boundary | per-tenant config | Monday 00:00 tenant TZ | Inherent |
| Email rate-limit hit | TASK-EMAIL-001 queue | Delivery delayed | Inherent retry |
| Reviewer subject_id deleted post-approval | FK soft | Review row retained | Inherent forensic |
| Bulk approve > 100 timesheets | size limit | 413 | Caller filters narrower |
| Auto-lock during Member typing | tx isolation | Member's submit may race with lock | First-wins |

---

## §11 — Implementation notes

**§11.1** Week boundary: Monday 00:00 tenant timezone.

**§11.2** Aggregation at submit reads entries for week × engagement × member.

**§11.3** Bulk approve uses single tx + N audit emits.

**§11.4** Auto-lock job runs daily 03:00; sweeps past-due.

**§11.5** Diff view compares two submission snapshots; each submission persists entry IDs.

**§11.6** Email templates use TASK-PORTAL-002 brand pack overrides for tenant theming.

**§11.7** Locked-week entry block at TASK-TIME-001 create.rs entry-point.

**§11.8** Concurrent submit race resolved by partial unique on (member, engagement, week).

**§11.9** Audit row carries member + AM subject IDs as hashes.

**§11.10** CFO visibility via separate read endpoint with `cfo` role; no write.

---

*End of TASK-TIME-006 spec.*
