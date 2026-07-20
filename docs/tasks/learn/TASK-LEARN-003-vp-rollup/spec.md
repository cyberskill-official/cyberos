---
id: TASK-LEARN-003
title: "LEARN VP (Voting Power) deterministic nightly roll-up — aggregates PROJ + TIME + KB contributions into per-Member VP score"
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
module: LEARN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-LEARN-001, TASK-PROJ-013, TASK-TIME-001, TASK-KB-001, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-PROJ-013, TASK-TIME-001]
blocks: [TASK-LEARN-007]

source_pages:
  - website/docs/modules/learn.html#vp-rollup

source_decisions:
  - DEC-2100 2026-05-17 — VP is a deterministic score per Member computed from PROJ (issues closed weighted by complexity) + TIME (billable hours) + KB (docs authored × view count); rolled up nightly
  - DEC-2101 2026-05-17 — Closed enum `vp_component` = {proj_contribution, time_billable, kb_authorship}; cardinality 3
  - DEC-2102 2026-05-17 — Per-Member VP snapshot IMMUTABLE per rollup period (week); corrections via new snapshot with `correction_of` link
  - DEC-2103 2026-05-17 — Deterministic — same inputs + same weights → same VP; weights versioned (CEO-configurable)
  - DEC-2104 2026-05-17 — memory audit kinds: learn.vp_rollup_started, learn.vp_snapshot_created, learn.vp_rollup_completed, learn.vp_rollup_failed

language: rust 1.81
service: cyberos/services/learn/
new_files:
  - services/learn/migrations/0003_vp_snapshots.sql
  - services/learn/src/vp/mod.rs
  - services/learn/src/vp/aggregator.rs
  - services/learn/src/vp/weights_loader.rs
  - services/learn/src/vp/nightly_batch.rs
  - services/learn/src/handlers/vp_routes.rs
  - services/learn/src/audit/vp_events.rs
  - services/learn/tests/vp_aggregator_test.rs
  - services/learn/tests/vp_component_enum_cardinality_test.rs
  - services/learn/tests/vp_snapshot_immutable_test.rs
  - services/learn/tests/vp_deterministic_test.rs
  - services/learn/tests/vp_weights_version_test.rs
  - services/learn/tests/vp_audit_emission_test.rs

modified_files:
  - services/learn/src/lib.rs

allowed_tools:
  - file_read: services/{learn,proj,time,kb}/**
  - file_write: services/learn/{src,tests,migrations}/**
  - bash: cd services/learn && cargo test vp

disallowed_tools:
  - mutate prior snapshot (per DEC-2102)
  - non-deterministic computation (per DEC-2103)

effort_hours: 6
subtasks:
  - "0.4h: 0003_vp_snapshots.sql"
  - "0.3h: vp/mod.rs"
  - "0.7h: aggregator.rs"
  - "0.4h: weights_loader.rs"
  - "0.5h: nightly_batch.rs"
  - "0.3h: handlers/vp_routes.rs"
  - "0.3h: audit/vp_events.rs"
  - "2.5h: tests — 6 test files"
  - "0.6h: docs + cron registration"

risk_if_skipped: "Without VP rollup, contribution recognition subjective. Without DEC-2102 immutability, prior periods rewritable (audit fail). Without DEC-2103 determinism, governance impossible (replay must yield same result)."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship VP rollup at `services/learn/src/vp/` aggregating PROJ + TIME + KB nightly, immutable per-week snapshots, versioned weights, 4 memory audit kinds.

1. **MUST** validate `vp_component` against closed enum per DEC-2101.

2. **MUST** aggregate at `aggregator.rs::aggregate(member, week, weights)`:
- proj_contribution = sum(issue.complexity × weight_proj) closed in week
- time_billable = sum(billable_hours × weight_time) for week
- kb_authorship = sum(doc.view_count × weight_kb) for docs authored before week
- total_vp = sum of weighted components

3. **MUST** be deterministic per DEC-2103 — pure function; same data + same weights → same output.

4. **MUST** schedule nightly batch via TASK-MCP-007 at 03:30 tenant_tz.

5. **MUST** define tables at migration `0003`:
   ```sql
   CREATE TABLE learn_vp_weights (
     tenant_id UUID NOT NULL,
     version INT NOT NULL,
     weight_proj NUMERIC(5,4) NOT NULL,
     weight_time NUMERIC(5,4) NOT NULL,
     weight_kb NUMERIC(5,4) NOT NULL,
     effective_from DATE NOT NULL,
     set_by UUID NOT NULL,
     PRIMARY KEY (tenant_id, version)
   );
   ALTER TABLE learn_vp_weights ENABLE ROW LEVEL SECURITY;
   CREATE POLICY weights_rls ON learn_vp_weights
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_vp_weights FROM cyberos_app;

   CREATE TABLE learn_vp_snapshots (
     snapshot_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     iso_week CHAR(8) NOT NULL,
     proj_score NUMERIC(10,4) NOT NULL,
     time_score NUMERIC(10,4) NOT NULL,
     kb_score NUMERIC(10,4) NOT NULL,
     total_vp NUMERIC(10,4) NOT NULL,
     weights_version INT NOT NULL,
     correction_of UUID REFERENCES learn_vp_snapshots(snapshot_id),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, member_id, iso_week, weights_version)
   );
   CREATE INDEX vp_snap_member_idx ON learn_vp_snapshots(tenant_id, member_id, iso_week DESC);
   ALTER TABLE learn_vp_snapshots ENABLE ROW LEVEL SECURITY;
   CREATE POLICY vp_snap_rls ON learn_vp_snapshots
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_vp_snapshots FROM cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/learn/vp/weights         (CEO; new version)
   GET  /v1/learn/vp/weights         (latest)
   GET  /v1/learn/members/{id}/vp    (history)
   POST /v1/learn/vp/rollup/trigger  (CEO manual)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2104. PII per TASK-MEMORY-111: scores SHA256.

8. **MUST** thread trace_id from cron → aggregator → audit.

9. **MUST NOT** mutate prior snapshot per DEC-2102 (correction = new row).

10. **MUST NOT** use non-deterministic inputs (no now(), no random).

---

## §2 — Why this design

**Why deterministic (DEC-2103)?** VP drives governance + REW BP fund distribution; replay must yield same result for audit.

**Why versioned weights (DEC-2103)?** Weights change over time; snapshots must record which version applied.

**Why immutable snapshots (DEC-2102)?** Historical VP is institutional record; rewriting = governance failure.

**Why 3 components (DEC-2101)?** Captures core contribution dimensions; closed enum prevents add-hoc scope creep.

---

## §3 — API contract

Sample VP snapshot:
```json
{
  "member_id": "uuid",
  "iso_week": "2026-W20",
  "proj_score": 45.5,
  "time_score": 32.0,
  "kb_score": 8.5,
  "total_vp": 86.0,
  "weights_version": 3
}
```

---

## §4 — Acceptance criteria
1. **vp_component enum cardinality 3**. 2. **Deterministic (same input → same output)**. 3. **Snapshots immutable**. 4. **Weights versioned**. 5. **Nightly cron 03:30**. 6. **UNIQUE(tenant, member, week, weights_version) idempotency**. 7. **4 memory audit kinds emitted**. 8. **PII scrubbed (scores SHA256)**. 9. **RLS denies cross-tenant**. 10. **CEO-only weights + manual trigger**. 11. **Trace_id preserved**. 12. **rust_decimal precision (10,4)**. 13. **Correction via new snapshot with correction_of link**. 14. **Append-only via REVOKE**. 15. **Inactive member skipped**. 16. **0 active members skipped**. 17. **Weights version pinning enforced**. 18. **Backfill via manual trigger with iso_week**. 19. **History query desc time**. 20. **Per-component score visible**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn deterministic_replay() {
    let ctx = TestContext::with_member_data().await;
    let v1 = ctx.run_rollup(this_week()).await;
    ctx.run_rollup(this_week()).await;
    let v2 = ctx.fetch_snapshot(ctx.member_id, this_week()).await;
    assert_eq!(v1.total_vp, v2.total_vp);
}

#[tokio::test]
async fn snapshot_immutable() {
    let ctx = TestContext::with_vp_snapshot().await;
    let r = ctx.try_mutate_snapshot(ctx.snapshot_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn correction_via_new_row() {
    let ctx = TestContext::with_vp_snapshot().await;
    ctx.run_correction_with_new_weights(ctx.snapshot_id).await;
    let snaps = ctx.fetch_snapshots_for_week(ctx.iso_week).await;
    assert_eq!(snaps.len(), 2);
    let corrected = snaps.iter().find(|s| s.correction_of.is_some()).unwrap();
    assert_eq!(corrected.correction_of, Some(ctx.snapshot_id));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-PROJ-013, TASK-TIME-001. **Cross-module:** TASK-LEARN-001 (member context), TASK-KB-001 (doc authorship), TASK-MCP-007 (cron), TASK-AUTH-101 (CEO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Source module fail | catch | sev-2; skip component | retry |
| Weights missing | catch | sev-1; halt | seed weights |
| Cron skipped | catch-up | inherent | inherent |
| Duplicate snapshot | UNIQUE | skip | inherent |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Non-deterministic input | code review | inherent | bug fix |
| Cross-tenant query | RLS | 0 rows | inherent |
| Mid-rollup crash | resume | partial | retry |
| Weights mid-week change | use snapshot's weights_version | inherent | inherent |
| Inactive member | skip | inherent | inherent |

## §11 — Implementation notes
- §11.1 Cron via TASK-MCP-007 `kind: 'learn.vp_rollup'`, daily 03:30.
- §11.2 Aggregator pure: `(member_data, weights) → VpScores`.
- §11.3 Snapshots stored per (member, week, weights_version) — weights change creates new snapshot.
- §11.4 memory audit body: member_id, week, weights_version; scores SHA256.
- §11.5 Manual trigger backfills via iso_week parameter; uses weights effective on week's Monday.

---

*End of TASK-LEARN-003 spec.*
