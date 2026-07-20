---
id: TASK-RES-002
title: "RES allocation Gantt UI — drag-rebalance interface over capacity matrix with optimistic concurrency + commit-on-save"
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
module: RES
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
related_tasks: [TASK-RES-001, TASK-RES-003, TASK-RES-005, TASK-PROJ-001, TASK-MEMORY-111]
depends_on: [TASK-RES-001]
blocks: []

source_pages:
  - website/docs/modules/res.html#gantt-ui

source_decisions:
  - DEC-2040 2026-05-17 — Gantt UI displays per-member-week capacity; drag-rebalance allows moving hours between projects within member's capacity
  - DEC-2041 2026-05-17 — Closed enum `allocation_change_status` = {proposed, validated, committed, rejected, conflicted}; cardinality 5
  - DEC-2042 2026-05-17 — Optimistic concurrency: each matrix row has version; UI changes verify version on commit
  - DEC-2043 2026-05-17 — Pre-commit validation: TASK-RES-005 OT cap check; TASK-RES-003 over-allocation flag; reject if violations
  - DEC-2044 2026-05-17 — Changes append to allocation_changes table; matrix row updated atomically; immutable history
  - DEC-2045 2026-05-17 — memory audit kinds: res.allocation_proposed, res.allocation_committed, res.allocation_rejected, res.allocation_conflicted

language: typescript / react + rust 1.81
service: cyberos/services/{res,portal-web}/
new_files:
  - services/res/migrations/0002_allocation_changes.sql
  - services/res/src/allocation/mod.rs
  - services/res/src/allocation/proposer.rs
  - services/res/src/allocation/validator.rs
  - services/res/src/allocation/commit_handler.rs
  - services/res/src/handlers/allocation_routes.rs
  - services/res/src/audit/allocation_events.rs
  - services/portal-web/src/res/GanttView.tsx
  - services/portal-web/src/res/DragHandler.tsx
  - services/portal-web/src/res/CommitDialog.tsx
  - services/res/tests/allocation_proposer_test.rs
  - services/res/tests/allocation_status_enum_cardinality_test.rs
  - services/res/tests/allocation_optimistic_concurrency_test.rs
  - services/res/tests/allocation_validation_test.rs
  - services/res/tests/allocation_commit_atomic_test.rs
  - services/res/tests/allocation_audit_emission_test.rs

modified_files:
  - services/portal-web/src/app/res/page.tsx

allowed_tools:
  - file_read: services/{res,portal-web}/**
  - file_write: services/{res,portal-web}/{src,tests,migrations}/**
  - bash: cd services/res && cargo test allocation; cd services/portal-web && pnpm test

disallowed_tools:
  - bypass validation gate (per DEC-2043)
  - mutate prior change row (per DEC-2044)

effort_hours: 12
subtasks:
  - "0.4h: 0002_allocation_changes.sql"
  - "0.4h: allocation/mod.rs"
  - "0.6h: proposer.rs"
  - "0.7h: validator.rs"
  - "0.5h: commit_handler.rs"
  - "0.5h: handlers/allocation_routes.rs"
  - "0.4h: audit/allocation_events.rs"
  - "3.5h: GanttView.tsx + DragHandler.tsx"
  - "1.0h: CommitDialog.tsx"
  - "2.0h: Rust tests — 6 files"
  - "1.0h: TS tests"
  - "1.0h: docs"

risk_if_skipped: "Without Gantt UI, allocation in spreadsheets → drift from matrix. Without DEC-2042 optimistic concurrency, two CHROs save conflicting allocations. Without DEC-2043 validation, OT cap bypassed → Labour Code violation."
---

## §1 — Description (BCP-14 normative)

The RES service + portal-web frontend **MUST** ship allocation Gantt at `services/res/src/allocation/` with drag-rebalance + optimistic concurrency + pre-commit validation, 4 memory audit kinds.

1. **MUST** validate `allocation_change_status` against closed enum per DEC-2041.

2. **MUST** propose changes at `proposer.rs::propose(member_id, week, changes)` per DEC-2040 — changes array of `{project_id, delta_hours}`.

3. **MUST** validate at `validator.rs::validate(proposal)` per DEC-2043:
- Total hours stay within member capacity (TASK-RES-001)
- TASK-RES-005 OT cap check
- TASK-RES-003 thresholds (warn-not-block on 110%)
- On violation: return rejection with reason.

4. **MUST** commit at `commit_handler.rs::commit(proposal, version)` per DEC-2044:
- Verify matrix row version matches (optimistic concurrency per DEC-2042)
- Insert allocation_changes row (history)
- UPDATE matrix row + increment version
- Transaction: all or nothing

5. **MUST** define table at migration `0002`:
   ```sql
   ALTER TABLE res_capacity_matrix ADD COLUMN version INT NOT NULL DEFAULT 1;
   GRANT UPDATE (allocated_hours, version) ON res_capacity_matrix TO cyberos_app;

   CREATE TABLE res_allocation_changes (
     change_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     iso_week CHAR(8) NOT NULL,
     project_id UUID NOT NULL,
     old_allocated_hours NUMERIC(5,2) NOT NULL,
     new_allocated_hours NUMERIC(5,2) NOT NULL,
     status TEXT NOT NULL
       CHECK (status IN ('proposed','validated','committed','rejected','conflicted')),
     rejection_reason TEXT,
     proposed_by UUID NOT NULL,
     committed_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX changes_member_week_idx ON res_allocation_changes(tenant_id, member_id, iso_week, created_at DESC);
   ALTER TABLE res_allocation_changes ENABLE ROW LEVEL SECURITY;
   CREATE POLICY changes_rls ON res_allocation_changes
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_allocation_changes FROM cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/res/allocations/propose   body: {member_id, iso_week, changes: [...], expected_version}
   POST /v1/res/allocations/{id}/commit
   GET  /v1/res/allocations/changes?member_id=...&iso_week=...
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2045. PII per TASK-MEMORY-111: hours SHA-256 hashed.

8. **MUST** thread trace_id from UI action → propose → validate → commit → audit.

9. **MUST NOT** bypass validation per DEC-2043.

10. **MUST NOT** mutate prior change row per DEC-2044 (append-only).

---

## §2 — Why this design

**Why drag UI (DEC-2040)?** Spreadsheet-style allocation is tedious; visual drag matches mental model.

**Why optimistic concurrency (DEC-2042)?** Multiple CHROs editing concurrently; version check prevents last-write-wins overwrite.

**Why pre-commit validation (DEC-2043)?** Better UX to surface OT cap violation at propose-time than after save.

**Why append-only changes (DEC-2044)?** Audit trail — "who changed what when" must be preserved.

---

## §3 — API contract

Sample propose:
```json
POST /v1/res/allocations/propose
{
  "member_id": "uuid",
  "iso_week": "2026-W20",
  "expected_version": 3,
  "changes": [
    {"project_id": "uuid-a", "delta_hours": -5},
    {"project_id": "uuid-b", "delta_hours": +5}
  ]
}
```

Response (validated):
```json
{
  "proposal_id": "uuid",
  "status": "validated",
  "warnings": [{"kind": "over_threshold", "current_pct": 105}]
}
```

Response (rejected):
```json
{
  "status": "rejected",
  "rejection_reason": "OT cap exceeded: weekly OT would be 14h, max 12h per Decree 145"
}
```

---

## §4 — Acceptance criteria
1. **allocation_change_status enum cardinality 5**. 2. **Drag UI updates state**. 3. **Pre-commit validation enforced**. 4. **OT cap rejection (TASK-RES-005)**. 5. **Over-threshold warning (TASK-RES-003 110%)**. 6. **Optimistic concurrency via version check**. 7. **Version mismatch → status=conflicted**. 8. **Transactional commit (all-or-nothing)**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (hours SHA256)**. 11. **RLS denies cross-tenant**. 12. **CHRO/PM role only**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE**. 15. **rust_decimal precision**. 16. **History query desc time**. 17. **Drag UI disables locked rows**. 18. **Commit dialog confirms changes**. 19. **WebSocket update broadcasts to other CHRO clients**. 20. **Undo via new compensating change**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn optimistic_concurrency_rejects_stale() {
    let ctx = TestContext::with_matrix_v1().await;
    let p1 = ctx.propose(ctx.member_id, this_week(), changes(), 1).await;
    ctx.commit(p1.id).await;  // version → 2
    let p2 = ctx.propose(ctx.member_id, this_week(), changes(), 1).await;
    let r = ctx.try_commit(p2.id).await;
    assert!(r.status == "conflicted");
}

#[tokio::test]
async fn ot_cap_rejection() {
    let ctx = TestContext::member_at_60h_already().await;
    let r = ctx.propose(ctx.member_id, this_week(), add_5h_more(), 1).await;
    assert!(r.status == "rejected");
    assert!(r.rejection_reason.contains("OT cap"));
}

#[tokio::test]
async fn commit_atomic() {
    let ctx = TestContext::with_matrix().await;
    let p = ctx.propose(...).await;
    ctx.simulate_db_failure_mid_commit().await;
    let row = ctx.fetch_matrix_row(ctx.member_id, week).await;
    assert_eq!(row.version, 1);  // unchanged
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-RES-001. **Cross-module:** TASK-RES-003 (threshold flags), TASK-RES-005 (OT cap), TASK-PROJ-001 (project context), TASK-AUTH-101 (CHRO/PM role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Version mismatch | check | conflicted; UI refresh | retry |
| OT cap violation | validator | rejected | adjust |
| Negative allocation | validate | 400 | fix |
| DB transaction fail | rollback | sev-2; status=rejected | retry |
| Cross-tenant write | RLS | 403 | inherent |
| Decimal precision | rust_decimal | inherent | inherent |
| WebSocket disconnect | reconnect | stale view + sev-3 | refresh |
| Drag past capacity bounds | UI clamp | inherent | inherent |
| Mass-allocation request (>50 changes) | batch limit | 400 | split |
| Concurrent same-cell propose | UNIQUE on commit | first wins | retry |

## §11 — Implementation notes
- §11.1 Gantt UI built with d3 or recharts; drag handler emits debounced state.
- §11.2 Version increment via `UPDATE ... SET version = version + 1 WHERE version = $expected RETURNING version`.
- §11.3 Validation parallelizes OT + threshold checks for sub-100ms response.
- §11.4 memory audit body: member_id, week, project_id, old/new hours SHA256.
- §11.5 WebSocket broadcasts on commit; other UIs invalidate cache.

---

*End of TASK-RES-002 spec.*
