---
id: TASK-HR-008
title: "HR performance signal aggregator — read-only consumer of PROJ + TIME + LEARN signals for periodic performance snapshots"
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
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-PROJ-013, TASK-TIME-001, TASK-LEARN-001, TASK-MEMORY-111]
depends_on: [TASK-PROJ-013, TASK-TIME-001]
blocks: []

source_pages:
  - website/docs/modules/hr.html#performance-signals

source_decisions:
  - DEC-1860 2026-05-17 — Read-only aggregator — never writes to source modules; snapshots monthly per member
  - DEC-1861 2026-05-17 — Closed enum `perf_signal_kind` = {proj_issue_velocity, time_utilization_pct, learn_completion_rate, project_satisfaction_avg, ot_burnout_flag}; cardinality 5
  - DEC-1862 2026-05-17 — Monthly snapshot at month-end; per-signal current value + delta vs prior month
  - DEC-1863 2026-05-17 — Snapshots IMMUTABLE; corrections via prior-period adjustment row
  - DEC-1864 2026-05-17 — memory audit kinds: hr.perf_snapshot_taken, hr.perf_snapshot_failed (PII scrubbed)

build_envelope:
  language: rust 1.81
  service: cyberos/services/hr/
  new_files:
    - services/hr/migrations/0007_perf_snapshots.sql
    - services/hr/src/perf/mod.rs
    - services/hr/src/perf/signal_aggregator.rs
    - services/hr/src/perf/snapshot_cron.rs
    - services/hr/src/handlers/perf_routes.rs
    - services/hr/src/audit/perf_events.rs
    - services/hr/tests/perf_aggregator_test.rs
    - services/hr/tests/perf_signal_kind_enum_cardinality_test.rs
    - services/hr/tests/perf_snapshot_immutable_test.rs
    - services/hr/tests/perf_read_only_test.rs
    - services/hr/tests/perf_audit_emission_test.rs

  modified_files:
    - services/hr/src/lib.rs

  allowed_tools:
    - file_read: services/{hr,proj,time,learn}/**
    - file_write: services/hr/{src,tests,migrations}/**
    - bash: cd services/hr && cargo test perf

  disallowed_tools:
    - write to source modules (per DEC-1860)
    - mutate snapshots (per DEC-1863)

effort_hours: 6
subtasks:
  - "0.3h: 0007_perf_snapshots.sql"
  - "0.3h: perf/mod.rs"
  - "0.8h: signal_aggregator.rs (5 signal sources)"
  - "0.4h: snapshot_cron.rs (monthly EOM)"
  - "0.4h: handlers/perf_routes.rs"
  - "0.3h: audit/perf_events.rs"
  - "2.5h: tests — 5 test files"
  - "1.0h: CHRO UI for snapshot review"

risk_if_skipped: "Without performance signals, CHRO has no quantitative view of member trajectory → review meetings rely on memory. Without DEC-1860 read-only constraint, this task could mutate source modules (data integrity risk). Without DEC-1863 immutability, prior-period restatement breaks audit."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship performance signal aggregator at `services/hr/src/perf/` reading PROJ + TIME + LEARN, monthly snapshots, immutable, 2 memory audit kinds.

1. **MUST** be read-only per DEC-1860 — only SELECT against source modules; allowed_tools disallows file_write to non-HR services.

2. **MUST** validate `perf_signal_kind` against closed enum per DEC-1861.

3. **MUST** aggregate at `signal_aggregator.rs::aggregate(member, period)`:
   - proj_issue_velocity: count issues closed / period (TASK-PROJ-013 read)
   - time_utilization_pct: billable_hours / total_hours (TASK-TIME-001 read)
   - learn_completion_rate: courses_completed / assigned (TASK-LEARN-001 read)
   - project_satisfaction_avg: avg from PROJ retro feedback
   - ot_burnout_flag: TRUE if OT hours > 80% of cap last 3 months (TASK-TIME-007)

4. **MUST** run monthly cron at EOM per DEC-1862 via TASK-MCP-007.

5. **MUST** define table at migration `0007`:
   ```sql
   CREATE TABLE hr_perf_snapshots (
     snapshot_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     period_end DATE NOT NULL,
     signals JSONB NOT NULL,
     prior_period_deltas JSONB,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, member_id, period_end)
   );
   CREATE INDEX perf_snap_member_time_idx ON hr_perf_snapshots(tenant_id, member_id, period_end DESC);
   ALTER TABLE hr_perf_snapshots ENABLE ROW LEVEL SECURITY;
   CREATE POLICY perf_snap_rls ON hr_perf_snapshots
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_perf_snapshots FROM cyberos_app;
   -- Immutable per DEC-1863
   ```

6. **MUST** expose endpoints (read-only):
   ```text
   POST   /v1/hr/perf/snapshot              (CHRO manual trigger)
   GET    /v1/hr/members/{id}/perf-history  (snapshot list desc time)
   ```

7. **MUST** emit 2 memory audit kinds per DEC-1864. PII per TASK-MEMORY-111: signal values hashed; member_id (uuid) ok.

8. **MUST** thread trace_id from cron → aggregator → audit.

9. **MUST NOT** write to PROJ/TIME/LEARN per DEC-1860 (read-only).

10. **MUST NOT** mutate prior snapshot per DEC-1863.

---

## §2 — Why this design

**Why read-only (DEC-1860)?** Boundary discipline — HR observes performance, doesn't influence source data integrity.

**Why monthly EOM (DEC-1862)?** Aligns with performance review cadence; weekly = noise, quarterly = too late.

**Why immutable snapshots (DEC-1863)?** Performance history must replay deterministically; corrections via new row.

**Why 5 signal kinds (DEC-1861)?** Covers velocity (PROJ), utilization (TIME), learning (LEARN), satisfaction (cultural), burnout (early-warning). Bounded prevents signal sprawl.

---

## §3 — API contract

Sample snapshot:
```json
{
  "snapshot_id": "uuid",
  "member_id": "uuid",
  "period_end": "2026-05-31",
  "signals": {
    "proj_issue_velocity": 12,
    "time_utilization_pct": 0.82,
    "learn_completion_rate": 1.0,
    "project_satisfaction_avg": 4.5,
    "ot_burnout_flag": false
  },
  "prior_period_deltas": {
    "proj_issue_velocity": +2,
    "time_utilization_pct": -0.03,
    "learn_completion_rate": 0.0,
    "project_satisfaction_avg": +0.2,
    "ot_burnout_flag": false
  }
}
```

---

## §4 — Acceptance criteria
1. **5-signal enum + cardinality test**. 2. **Monthly EOM cron**. 3. **Read-only against PROJ/TIME/LEARN**. 4. **Snapshots immutable (no UPDATE/DELETE)**. 5. **UNIQUE on (member, period_end)**. 6. **Prior-period deltas computed**. 7. **2 memory audit kinds emitted**. 8. **PII scrubbed (signal values SHA256)**. 9. **RLS denies cross-tenant**. 10. **CHRO-only manual trigger**. 11. **Trace_id preserved**. 12. **Append-only via REVOKE**. 13. **Missing source data → null signal + sev-2 audit**. 14. **Inactive member skipped**. 15. **ot_burnout_flag computed from 3-month rolling**. 16. **History query desc time**. 17. **CHRO-only GET**. 18. **Performance review UI consumes this**. 19. **Cron skip if 0 active members**. 20. **JSONB schema validated per signal kind**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn aggregator_pulls_5_signals() {
    let ctx = TestContext::with_member_data_complete().await;
    let snap = ctx.aggregate(ctx.member_id, "2026-05-31").await;
    assert_eq!(snap.signals.len(), 5);
    assert!(snap.signals.contains_key("proj_issue_velocity"));
}

#[tokio::test]
async fn snapshots_immutable() {
    let ctx = TestContext::with_perf_snapshot().await;
    let r = ctx.try_mutate_snapshot(ctx.snapshot_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn read_only_against_sources() {
    let ctx = TestContext::with_member_data().await;
    ctx.run_perf_aggregation(ctx.member_id).await;
    let proj_writes = ctx.proj_write_count_since_test_start().await;
    let time_writes = ctx.time_write_count_since_test_start().await;
    assert_eq!(proj_writes, 0);
    assert_eq!(time_writes, 0);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-PROJ-013, TASK-TIME-001.
**Cross-module:** TASK-LEARN-001 (completion data), TASK-TIME-007 (OT for burnout flag), TASK-MCP-007 (cron), TASK-AUTH-101 (CHRO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Source module unavailable | retry | null signal + sev-2 | next snapshot |
| Snapshot mutation attempt | RLS + REVOKE | DB error | inherent |
| Duplicate snapshot per period | UNIQUE | skip | inherent |
| Inactive member | filter | skip | inherent |
| Cron skipped | catch-up next run | inherent | inherent |
| Member just hired (no history) | empty signals + sev-3 | inherent | inherent |
| TASK-TIME-007 burnout calc fail | flag=false default | sev-2 | data fix |
| Cross-tenant aggregate attempt | RLS | 0 rows | inherent |
| JSONB schema mismatch | validator | reject | bug fix |
| Decimal precision drift | rust_decimal | inherent | inherent |

## §11 — Implementation notes
- §11.1 Aggregator calls each source module's read API; no SQL joins across modules.
- §11.2 Burnout flag: `SELECT OT hours last 3 months / cap; flag if avg > 0.8`.
- §11.3 Prior-period deltas computed at snapshot time (lookup previous snapshot).
- §11.4 memory audit body: member_id, period_end, signal_count; signal values SHA256.
- §11.5 Performance review UI: read snapshots + deltas; manager + CHRO see trajectory.

---

*End of TASK-HR-008 spec.*
