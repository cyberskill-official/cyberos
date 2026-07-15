---
id: TASK-ESOP-006
title: "ESOP M&A acceleration trigger — Board declares M&A event + 5-business-day Member notice + full vesting acceleration for all active grants"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: ESOP
priority: p1
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-ESOP-001, TASK-ESOP-002, TASK-EMAIL-009, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-ESOP-001]
blocks: []

source_pages:
  - website/docs/modules/esop.html#ma-acceleration

source_decisions:
  - DEC-2300 2026-05-17 — Board declares M&A event → all active grants accelerated to fully_vested; Member notice via TASK-EMAIL-009 within 5 business days
  - DEC-2301 2026-05-17 — Closed enum `ma_event_status` = {declared, accelerating, members_notified, completed, dismissed}; cardinality 5
  - DEC-2302 2026-05-17 — Board threshold for declaration: ≥3 board signs (per TASK-ESOP-003 board threshold)
  - DEC-2303 2026-05-17 — Acceleration cron processes all active grants → marks status=accelerated + sets shares_vested = total_shares
  - DEC-2304 2026-05-17 — memory audit kinds: esop.ma_event_declared, esop.ma_event_signed, esop.ma_event_accelerating, esop.ma_event_member_notified, esop.ma_event_completed

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0006_ma_events.sql
    - services/esop/src/ma/mod.rs
    - services/esop/src/ma/acceleration_runner.rs
    - services/esop/src/ma/member_notifier.rs
    - services/esop/src/handlers/ma_routes.rs
    - services/esop/src/audit/ma_events.rs
    - services/esop/tests/ma_event_status_enum_cardinality_test.rs
    - services/esop/tests/ma_board_threshold_test.rs
    - services/esop/tests/ma_acceleration_full_vest_test.rs
    - services/esop/tests/ma_member_notice_5bd_test.rs
    - services/esop/tests/ma_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/{esop,email}/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test ma

  disallowed_tools:
    - accelerate without board threshold (per DEC-2302)
    - skip member notice (per DEC-2300)

effort_hours: 5
subtasks:
  - "0.3h: 0006_ma_events.sql"
  - "0.3h: ma/mod.rs"
  - "0.5h: acceleration_runner.rs"
  - "0.4h: member_notifier.rs"
  - "0.4h: handlers/ma_routes.rs"
  - "0.3h: audit/ma_events.rs"
  - "2.0h: tests — 5 test files"
  - "0.8h: CEO+Board UI"

risk_if_skipped: "Without M&A acceleration, change-of-control breaks member expectations. Without DEC-2300 5-day notice, members surprised → trust damaged."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship M&A acceleration at `services/esop/src/ma/` with Board declaration + acceleration cron + 5-business-day member notice, 5 memory audit kinds.

1. **MUST** validate `ma_event_status` against closed enum per DEC-2301.

2. **MUST** require board threshold per DEC-2302 (reuse TASK-ESOP-003 board threshold logic).

3. **MUST** run acceleration at `acceleration_runner.rs::accelerate(ma_event)` per DEC-2303:
   - For all active grants: set vested = total_shares
   - Insert special accrual row (kind='ma_acceleration')
   - Update grant.status = 'accelerated'

4. **MUST** notify members within 5 business days at `member_notifier.rs::notify(ma_event)` per DEC-2300:
   - For each affected member: send email via TASK-EMAIL-009
   - Track per-member notification_status

5. **MUST** define tables at migration `0006`:
   ```sql
   CREATE TABLE esop_ma_events (
     ma_event_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     event_description TEXT NOT NULL,
     declared_by UUID NOT NULL,
     declared_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     status TEXT NOT NULL DEFAULT 'declared'
       CHECK (status IN ('declared','accelerating','members_notified','completed','dismissed')),
     acceleration_started_at TIMESTAMPTZ,
     all_notified_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32)
   );
   ALTER TABLE esop_ma_events ENABLE ROW LEVEL SECURITY;
   CREATE POLICY ma_rls ON esop_ma_events
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_ma_events FROM cyberos_app;
   GRANT UPDATE (status, acceleration_started_at, all_notified_at, completed_at) ON esop_ma_events TO cyberos_app;

   CREATE TABLE esop_ma_board_signs (
     sign_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     ma_event_id UUID NOT NULL REFERENCES esop_ma_events(ma_event_id),
     board_member_id UUID NOT NULL,
     signed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (ma_event_id, board_member_id)
   );
   ALTER TABLE esop_ma_board_signs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY ma_signs_rls ON esop_ma_board_signs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_ma_board_signs FROM cyberos_app;

   CREATE TABLE esop_ma_member_notices (
     notice_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     ma_event_id UUID NOT NULL REFERENCES esop_ma_events(ma_event_id),
     member_id UUID NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','sent','failed')),
     sent_at TIMESTAMPTZ,
     email_message_id UUID,
     UNIQUE (ma_event_id, member_id)
   );
   ALTER TABLE esop_ma_member_notices ENABLE ROW LEVEL SECURITY;
   CREATE POLICY notices_rls ON esop_ma_member_notices
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_ma_member_notices FROM cyberos_app;
   GRANT UPDATE (status, sent_at, email_message_id) ON esop_ma_member_notices TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/esop/ma-events                       (CEO declares)
   POST /v1/esop/ma-events/{id}/board-sign       (board member)
   POST /v1/esop/ma-events/{id}/accelerate       (auto on threshold)
   GET  /v1/esop/ma-events/{id}                  (status)
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2304. PII per TASK-MEMORY-111: descriptions SHA256.

8. **MUST** thread trace_id from declare → sign → accelerate → notify → audit.

9. **MUST NOT** accelerate without board threshold per DEC-2302.

10. **MUST NOT** skip member notice per DEC-2300 (5-business-day deadline tracked).

---

## §2 — Why this design

**Why board threshold (DEC-2302)?** M&A acceleration = major equity event; requires governance.

**Why 5-business-day notice (DEC-2300)?** Industry standard for change-of-control disclosure; allows members to plan.

**Why per-member notice tracking (DEC-2300)?** Audit trail proves notification delivered.

---

## §3 — API contract

Sample M&A event:
```json
POST /v1/esop/ma-events
{
  "event_description": "Acquisition by AcmeCorp, closing 2026-Q3"
}
```

Sample status:
```json
{
  "ma_event_id": "uuid",
  "status": "completed",
  "board_signs_count": 3,
  "members_notified_count": 30,
  "completed_at": "2026-06-01T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **ma_event_status enum cardinality 5**. 2. **Board threshold (≥3 default)**. 3. **All active grants accelerated**. 4. **vested = total_shares set**. 5. **5-business-day notice tracked**. 6. **Per-member notice via TASK-EMAIL-009**. 7. **5 memory audit kinds emitted**. 8. **PII scrubbed (desc SHA256)**. 9. **RLS denies cross-tenant**. 10. **CEO-only declare**. 11. **Board member sign-only**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except status cols**. 14. **UNIQUE on (ma_event_id, board_member_id)**. 15. **UNIQUE on (ma_event_id, member_id) for notices**. 16. **status workflow enforced**. 17. **Per-member notice failure isolated**. 18. **Cancelled grants excluded**. 19. **bigint shares**. 20. **Dismiss allowed pre-acceleration**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn board_threshold_required() {
    let ctx = TestContext::with_declared_ma().await;
    ctx.board_sign(ctx.b1).await;
    ctx.board_sign(ctx.b2).await;
    let m = ctx.fetch_ma(ctx.ma_id).await;
    assert_eq!(m.status, "declared");  // still
    ctx.board_sign(ctx.b3).await;
    let m2 = ctx.fetch_ma(ctx.ma_id).await;
    assert_eq!(m2.status, "accelerating");
}

#[tokio::test]
async fn all_grants_accelerated() {
    let ctx = TestContext::with_5_active_grants_and_ma_signed().await;
    ctx.run_acceleration(ctx.ma_id).await;
    for g_id in ctx.grant_ids() {
        let g = ctx.fetch_grant(g_id).await;
        assert_eq!(g.status, "accelerated");
        let a = ctx.fetch_latest_accrual(g_id).await;
        assert_eq!(a.vested_cumulative, g.total_shares);
    }
}

#[tokio::test]
async fn members_notified_in_5_bd() {
    let ctx = TestContext::with_accelerated_ma().await;
    ctx.run_notifier(ctx.ma_id).await;
    let notices = ctx.fetch_notices(ctx.ma_id).await;
    let sent = notices.iter().filter(|n| n.status == "sent").count();
    assert_eq!(sent, ctx.member_count());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-ESOP-001.
**Cross-module:** TASK-ESOP-002 (accrual rows), TASK-EMAIL-009 (notice), TASK-MCP-007 (deadline cron), TASK-AUTH-101 (CEO + board), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Board threshold not reached | inherent | stays declared | get signs |
| Acceleration mid-run crash | resume | partial | retry |
| Notice send fail per-member | per-row | isolate; retry | inherent |
| 5-day deadline missed | cron alert | sev-1 + escalate | manual notify |
| Cancelled grant in active list | filter | skip | inherent |
| Cross-tenant declare | RLS | 403 | inherent |
| Duplicate sign | UNIQUE | second skip | inherent |
| Member has no email | sev-2 | skip + manual notify | data fix |
| Concurrent acceleration | UPDATE WHERE | first wins | inherent |
| Dismiss post-acceleration | reject | 409 | inherent |

## §11 — Implementation notes
- §11.1 Cron via TASK-MCP-007 for 5-business-day countdown; sev-1 alert if deadline approaches with pending notices.
- §11.2 Acceleration creates TASK-ESOP-002 accrual row with kind='ma_acceleration'.
- §11.3 memory audit body: ma_event_id, status, counts; description SHA256.
- §11.4 Per-member notice via TASK-EMAIL-009; email template includes vested shares + next steps.
- §11.5 Business-day calc: VN public holidays from TASK-HR-005 policy.

---

*End of TASK-ESOP-006 spec.*
