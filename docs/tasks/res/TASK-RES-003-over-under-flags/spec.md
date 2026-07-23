---
id: TASK-RES-003
title: "RES over/under-allocation flags — 110% warning / 60% under-utilization threshold with weekly digest to CHRO"
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
module: res
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 8
slice: 8
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-RES-001, TASK-RES-002, TASK-CHAT-005, TASK-EMAIL-009, TASK-MEMORY-111]
depends_on: [TASK-RES-001]
blocks: []

source_pages:
  - website/docs/modules/res.html#thresholds

source_decisions:
  - DEC-2050 2026-05-17 — Two thresholds: 110% = over-allocated (burnout risk); 60% = under-utilized (bench risk)
  - DEC-2051 2026-05-17 — Closed enum `allocation_flag` = {over_allocated, healthy, under_utilized}; cardinality 3
  - DEC-2052 2026-05-17 — Computed at TASK-RES-001 batch + on TASK-RES-002 commit; cached on matrix row
  - DEC-2053 2026-05-17 — Weekly digest Friday 16:00 CHRO summarizes flag counts + flagged members
  - DEC-2054 2026-05-17 — memory audit kinds: res.flag_changed, res.weekly_digest_sent

language: rust 1.81
service: cyberos/services/res/
new_files:
  - services/res/migrations/0003_allocation_flags.sql
  - services/res/src/flags/mod.rs
  - services/res/src/flags/computer.rs
  - services/res/src/flags/weekly_digest.rs
  - services/res/src/audit/flags_events.rs
  - services/res/tests/flags_thresholds_test.rs
  - services/res/tests/flag_enum_cardinality_test.rs
  - services/res/tests/flags_weekly_digest_test.rs
  - services/res/tests/flags_audit_emission_test.rs

modified_files:
  - services/res/src/matrix/computer.rs

allowed_tools:
  - file_read: services/{res,chat,email}/**
  - file_write: services/res/{src,tests,migrations}/**
  - bash: cd services/res && cargo test flags

disallowed_tools:
  - hardcode thresholds outside DEC-2050 (per spec — version-pinned)

effort_hours: 4
subtasks:
  - "0.3h: 0003_allocation_flags.sql"
  - "0.3h: flags/mod.rs"
  - "0.4h: computer.rs"
  - "0.5h: weekly_digest.rs"
  - "0.3h: audit/flags_events.rs"
  - "1.8h: tests — 4 test files"
  - "0.4h: docs + cron registration"

risk_if_skipped: "Without flags, CHRO scans entire matrix manually → over-allocation persists weeks. Without DEC-2053 digest, even flag column ignored. Without DEC-2050 thresholds, ad-hoc judgments vary by week."
---

## §1 — Description (BCP-14 normative)

The RES service **MUST** ship flag computation + weekly digest at `services/res/src/flags/` with 110/60 thresholds + Friday CHRO digest, 2 memory audit kinds.

1. **MUST** validate `allocation_flag` against closed enum per DEC-2051.

2. **MUST** compute flag at `computer.rs::compute(matrix_row)` per DEC-2050:
- utilization_pct = allocated_hours / capacity_hours
- >= 110% → over_allocated
- >= 60% AND < 110% → healthy
- < 60% → under_utilized

3. **MUST** cache flag on TASK-RES-001 matrix row:
   ```sql
   ALTER TABLE res_capacity_matrix ADD COLUMN allocation_flag TEXT
     CHECK (allocation_flag IS NULL OR allocation_flag IN ('over_allocated','healthy','under_utilized'));
   ALTER TABLE res_capacity_matrix ADD COLUMN flag_computed_at TIMESTAMPTZ;
   GRANT UPDATE (allocation_flag, flag_computed_at) ON res_capacity_matrix TO cyberos_app;
   ```

4. **MUST** schedule weekly digest Friday 16:00 tenant_tz per DEC-2053 via TASK-MCP-007 — count flagged members + send to CHRO via TASK-EMAIL-009 + TASK-CHAT-005.

5. **MUST** define digest table at migration `0003`:
   ```sql
   CREATE TABLE res_weekly_digests (
     digest_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     iso_week CHAR(8) NOT NULL,
     over_count INT NOT NULL,
     under_count INT NOT NULL,
     healthy_count INT NOT NULL,
     flagged_members_jsonb JSONB NOT NULL,
     sent_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, iso_week)
   );
   ALTER TABLE res_weekly_digests ENABLE ROW LEVEL SECURITY;
   CREATE POLICY digests_rls ON res_weekly_digests
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_weekly_digests FROM cyberos_app;
   GRANT UPDATE (sent_at) ON res_weekly_digests TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   GET /v1/res/flags/summary               (current week counts)
   GET /v1/res/weekly-digests              (list)
   ```

7. **MUST** emit 2 memory audit kinds per DEC-2054. PII per TASK-MEMORY-111: flag enums (public) ok; counts ok.

8. **MUST** thread trace_id from compute / cron → digest → audit.

9. **MUST NOT** hardcode thresholds outside DEC-2050 (consider future per-tenant override).

---

## §2 — Why this design

**Why 110%/60% (DEC-2050)?** Industry standard; balance burnout prevention + bench cost.

**Why cache on matrix row (DEC-2052)?** Avoids recomputing on every query; UI shows flag with row.

**Why weekly digest (DEC-2053)?** Daily would be noise; monthly too late.

---

## §3 — API contract

Sample summary:
```json
{
  "iso_week": "2026-W20",
  "over_count": 3,
  "healthy_count": 12,
  "under_count": 2,
  "over_members": [
    {"member_id": "uuid", "name": "Alice", "utilization_pct": 115}
  ]
}
```

---

## §4 — Acceptance criteria
1. **allocation_flag enum cardinality 3**. 2. **>=110% → over_allocated**. 3. **<60% → under_utilized**. 4. **Threshold boundaries exact (110 and 60 inclusive of healthy edge)**. 5. **Flag cached on matrix row**. 6. **Weekly digest Friday 16:00**. 7. **Digest sent via email + chat**. 8. **UNIQUE(tenant_id, iso_week) idempotency**. 9. **2 memory audit kinds emitted**. 10. **PII: flag enums + counts ok**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except sent_at**. 14. **0-capacity member → flag=null + sev-3**. 15. **Recompute on TASK-RES-002 commit**. 16. **Recompute on TASK-RES-001 batch**. 17. **Digest skip if 0 members**. 18. **rust_decimal precision for utilization**. 19. **CHRO-only digest config**. 20. **flag_computed_at populated**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn over_allocated_at_115pct() {
    let row = matrix_row_with(capacity: 40, allocated: 46);
    assert_eq!(flags::compute(&row), Flag::OverAllocated);
}

#[tokio::test]
async fn under_at_50pct() {
    let row = matrix_row_with(capacity: 40, allocated: 20);
    assert_eq!(flags::compute(&row), Flag::UnderUtilized);
}

#[tokio::test]
async fn weekly_digest_sent_friday() {
    let ctx = TestContext::with_3_over_2_under_members().await;
    ctx.run_friday_digest_cron().await;
    let digest = ctx.fetch_latest_digest().await;
    assert_eq!(digest.over_count, 3);
    assert_eq!(digest.under_count, 2);
    let email_sent = ctx.email_send_count().await;
    assert_eq!(email_sent, 1);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-RES-001. **Cross-module:** TASK-RES-002 (UI hook to recompute), TASK-EMAIL-009, TASK-CHAT-005, TASK-MCP-007, TASK-AUTH-101, TASK-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| 0 capacity | null flag + sev-3 | inherent | data fix |
| Cron skipped | catch-up | inherent | inherent |
| Duplicate digest | UNIQUE | skip | inherent |
| Email send fail | sev-2 | retry | inherent |
| Chat send fail | sev-2 | retry | inherent |
| Negative allocation | flag=under (clamp 0) | sev-3 | data fix |
| Decimal precision | rust_decimal | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Threshold mid-batch change | inherent | inherent | future tenant override |
| Bench member (contractor) | inherent flag computed | inherent | inherent |

## §11 — Implementation notes
- §11.1 Flag computer pure function: `(capacity, allocated) → Flag`.
- §11.2 Digest cron via TASK-MCP-007 `kind: 'res.weekly_flags_digest'`, Friday 16:00.
- §11.3 memory audit body: tenant_id, week, counts; member uuids included for flagged.
- §11.4 Future: per-tenant threshold override table for industries with different norms.
- §11.5 Digest excludes contractors (per TASK-HR-002 type override).

---

*End of TASK-RES-003 spec.*
