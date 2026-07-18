---
id: TASK-HR-004
title: "HR 8 leave types — annual/sick/maternity/paternity/sabbatical/unpaid/bereavement/public_holiday with per-type accrual + approval rules"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: HR
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-HR-002, TASK-HR-006, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: [TASK-HR-006]

source_pages:
  - website/docs/modules/hr.html#leave-types
  # VN Labour Code 45/2019 + Decree 145/2020
  - https://thuvienphapluat.vn/

source_decisions:
  - DEC-1830 2026-05-17 — 8 leave types per VN Labour Code + sabbatical (business-defined); each with distinct accrual + approval rules
  - DEC-1831 2026-05-17 — Closed enum `leave_type` = {annual, sick, maternity, paternity, sabbatical, unpaid, bereavement, public_holiday}; cardinality 8
  - DEC-1832 2026-05-17 — Closed enum `leave_status` = {requested, approved, rejected, taken, cancelled}; cardinality 5
  - DEC-1833 2026-05-17 — Per-type approval gate: manager (default) | CHRO (sabbatical+unpaid >5d) | auto (public_holiday) | none (bereavement up to 3d)
  - DEC-1834 2026-05-17 — Per-type entitlement: annual=12d/yr (Art. 113), sick=30d/yr SI-funded (Decree 144), maternity=180d (Art. 139), paternity=14d (Art. 139), bereavement=3d (Art. 116), public_holiday=11d/yr (Art. 112), sabbatical+unpaid=on-request
  - DEC-1835 2026-05-17 — memory audit kinds: hr.leave_requested, hr.leave_approved, hr.leave_rejected, hr.leave_taken, hr.leave_cancelled

build_envelope:
  language: rust 1.81
  service: cyberos/services/hr/
  new_files:
    - services/hr/migrations/0004_leave_requests.sql
    - services/hr/src/leave/mod.rs
    - services/hr/src/leave/entitlement_calc.rs
    - services/hr/src/leave/approval_router.rs
    - services/hr/src/handlers/leave_routes.rs
    - services/hr/src/audit/leave_events.rs
    - services/hr/tests/leave_type_enum_cardinality_test.rs
    - services/hr/tests/leave_status_enum_cardinality_test.rs
    - services/hr/tests/leave_entitlement_per_type_test.rs
    - services/hr/tests/leave_approval_routing_test.rs
    - services/hr/tests/leave_balance_deduction_test.rs
    - services/hr/tests/leave_audit_emission_test.rs

  modified_files:
    - services/hr/src/members.rs

  allowed_tools:
    - file_read: services/hr/**
    - file_write: services/hr/{src,tests,migrations}/**
    - bash: cd services/hr && cargo test leave

  disallowed_tools:
    - bypass approval gate (per DEC-1833)
    - exceed type entitlement (per DEC-1834)

effort_hours: 5
subtasks:
  - "0.3h: 0004_leave_requests.sql"
  - "0.4h: leave/mod.rs"
  - "0.5h: entitlement_calc.rs"
  - "0.5h: approval_router.rs"
  - "0.4h: handlers/leave_routes.rs"
  - "0.3h: audit/leave_events.rs"
  - "2.0h: tests — 6 test files"
  - "0.6h: Member UI for leave request + balance display"

risk_if_skipped: "Without leave type enforcement, members can request leave types beyond entitlement → over-counting. Without DEC-1833 approval routing, sabbatical/unpaid auto-approved without CHRO oversight. Without DEC-1834 statutory entitlements, VN Labour Code non-compliance."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship leave types at `services/hr/src/leave/` with 8 types + per-type entitlement + per-type approval routing + balance tracking, 5 memory audit kinds.

1. **MUST** validate `leave_type` against closed enum per DEC-1831.

2. **MUST** validate `leave_status` against closed enum per DEC-1832.

3. **MUST** compute entitlement per DEC-1834 at `entitlement_calc.rs::compute(member, leave_type, year)`:
   - annual: 12d/yr (TASK-HR-002 type override: contractor=0, part_time=pro-rated)
   - sick: 30d/yr (SI-funded; doesn't deduct from annual)
   - maternity: 180d (one-time per pregnancy)
   - paternity: 14d (one-time per pregnancy of partner)
   - bereavement: 3d/event (close family)
   - public_holiday: 11d/yr (auto-applied, no request)
   - sabbatical: on-request, no auto-entitlement
   - unpaid: on-request, no auto-entitlement

4. **MUST** route approval per DEC-1833 at `approval_router.rs::route(request)`:
   - annual/sick/paternity: manager (member.manager_id)
   - sabbatical / unpaid >5d: CHRO
   - public_holiday: auto-approve
   - bereavement ≤3d: auto-approve
   - bereavement >3d: manager + CHRO

5. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE hr_leave_requests (
     request_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     leave_type TEXT NOT NULL
       CHECK (leave_type IN ('annual','sick','maternity','paternity','sabbatical','unpaid','bereavement','public_holiday')),
     start_date DATE NOT NULL,
     end_date DATE NOT NULL CHECK (end_date >= start_date),
     days_count NUMERIC(5,2) NOT NULL CHECK (days_count > 0),
     reason TEXT,
     status TEXT NOT NULL DEFAULT 'requested'
       CHECK (status IN ('requested','approved','rejected','taken','cancelled')),
     approver_id UUID,
     approved_at TIMESTAMPTZ,
     rejection_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX leave_member_year_idx ON hr_leave_requests(tenant_id, member_id, EXTRACT(YEAR FROM start_date));
   ALTER TABLE hr_leave_requests ENABLE ROW LEVEL SECURITY;
   CREATE POLICY leave_rls ON hr_leave_requests
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_leave_requests FROM cyberos_app;
   GRANT UPDATE (status, approver_id, approved_at, rejection_reason) ON hr_leave_requests TO cyberos_app;
   ```

6. **MUST** prevent over-entitlement per DEC-1834 — at approval, check member's running balance for the type+year; reject if exceeds.

7. **MUST** emit 5 memory audit kinds per DEC-1835. PII per TASK-MEMORY-111: reason text SHA-256 hashed; member_id (uuid) + dates ok.

8. **MUST** thread trace_id from request → router → approver → status update → audit.

9. **MUST NOT** bypass approval gate per DEC-1833 (manager+CHRO required for >5d unpaid).

10. **MUST NOT** exceed type entitlement per DEC-1834.

---

## §2 — Why this design

**Why 8 types (DEC-1830)?** Covers VN Labour Code + business need (sabbatical for senior retention). Closed enum prevents type sprawl.

**Why per-type approval (DEC-1833)?** Sabbatical/unpaid have business impact (replacement cost); CHRO oversight. Bereavement should be friction-free.

**Why statutory entitlements (DEC-1834)?** Decree 145/2020 mandates specific days; auto-enforce to avoid VN Labour inspector findings.

**Why per-type balance (DEC-1834)?** Sick days don't deduct from annual; separate accrual prevents under-utilization (members feeling they "spend" days when sick).

---

## §3 — API contract

```text
POST   /v1/hr/leave-requests                  body: {leave_type, start_date, end_date, reason?}
POST   /v1/hr/leave-requests/{id}/approve     (manager/CHRO)
POST   /v1/hr/leave-requests/{id}/reject      body: {reason}
POST   /v1/hr/leave-requests/{id}/cancel      (member-self before approval)
GET    /v1/hr/members/{id}/leave-balance      (per-type balance for current year)
```

Sample balance:
```json
{
  "year": 2026,
  "balances": [
    {"leave_type": "annual", "entitled": 12, "taken": 5, "pending": 2, "remaining": 5},
    {"leave_type": "sick", "entitled": 30, "taken": 3, "pending": 0, "remaining": 27}
  ]
}
```

---

## §4 — Acceptance criteria
1. **leave_type enum cardinality 8**. 2. **leave_status enum cardinality 5**. 3. **Per-type entitlement computed**. 4. **Per-type approval routing**. 5. **Balance deduction on status=taken**. 6. **Over-entitlement rejected at approve**. 7. **Manager+CHRO co-sign for unpaid >5d**. 8. **Auto-approve bereavement ≤3d**. 9. **Auto-apply public_holiday**. 10. **5 memory audit kinds emitted**. 11. **PII scrubbed (reason SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Cancel before approve allowed**. 15. **Cancel after approve requires CHRO**. 16. **Append-only via REVOKE except 4 status cols**. 17. **Contractor → 0 annual entitlement (TASK-HR-002 override)**. 18. **Part_time → pro-rated (TASK-HR-002 override)**. 19. **Sick SI-funded note in memory audit**. 20. **Year boundary handled (Dec 31 → Jan 1 transition)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn annual_entitlement_12d() {
    let ctx = TestContext::with_indefinite_member().await;
    let bal = ctx.fetch_balance(ctx.member_id, "annual", 2026).await;
    assert_eq!(bal.entitled, 12);
}

#[tokio::test]
async fn unpaid_over_5d_requires_chro() {
    let ctx = TestContext::with_member_and_manager().await;
    let req = ctx.request_leave("unpaid", 10).await;
    let r = ctx.try_approve_as_manager(req.id).await;
    assert!(r.is_err());  // Manager alone insufficient
    let r2 = ctx.approve_as_chro(req.id).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn over_entitlement_rejected() {
    let ctx = TestContext::with_member_used_12_annual().await;
    let req = ctx.request_leave("annual", 5).await;
    let r = ctx.try_approve(req.id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn contractor_zero_annual() {
    let ctx = TestContext::with_contractor_member().await;
    let bal = ctx.fetch_balance(ctx.member_id, "annual", 2026).await;
    assert_eq!(bal.entitled, 0);
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001.
**Cross-module:** TASK-HR-002 (contract type override), TASK-HR-006 (accrual cron), TASK-AUTH-101 (CHRO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Over-entitlement | balance check | 400 | reduce request |
| Type not in enum | CHECK constraint | 400 | use valid |
| Date range invalid | validate | 400 | fix |
| Manager not set | route fallback to CHRO | inherent | data fix |
| Approve already approved | status check | 409 | inherent |
| Cancel after taken | reject | 400 | inherent |
| Year boundary leave | split into 2 requests | manual | inherent |
| Sabbatical at probation | reject per HR-002 | inherent | post-probation |
| Maternity claim without record | reject | 400 | data setup |
| Cross-tenant request | RLS | 404 | inherent |

## §11 — Implementation notes
- §11.1 Entitlement calc consults TASK-HR-002 contract type overrides.
- §11.2 Approval router: state-machine; default manager, escalate per type/duration rules.
- §11.3 Sick leave SI-funded: audit notes for TASK-REW-004 statutory deduction logic.
- §11.4 memory audit body: member_id, leave_type, days_count, status; reason SHA256.
- §11.5 Public_holiday auto-applied: cron at start-of-year, creates approved requests for VN public holiday calendar.

---

*End of TASK-HR-004 spec.*
