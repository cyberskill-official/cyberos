---
id: TASK-CUO-104
title: "CUO topological walk of `depends_on` chain — orchestrates multi-step skill invocations with composite audit row + per-step sub-rows"
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
module: CUO
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CUO-101, TASK-CUO-102, TASK-CUO-105, TASK-SKILL-001, TASK-MEMORY-111]
depends_on: [TASK-CUO-101]
blocks: [TASK-CUO-105]

source_pages:
  - website/docs/modules/cuo.html#topo-walk

source_decisions:
  - DEC-2340 2026-05-17 — Supervisor can chain skills via declared depends_on; walker topologically sorts, executes in order; composite audit row captures whole chain + sub-rows per step
  - DEC-2341 2026-05-17 — Closed enum `chain_status` = {planning, executing, completed, failed, partial_rolled_back}; cardinality 5
  - DEC-2342 2026-05-17 — Cycle detection at plan time; reject chain if cycle found
  - DEC-2343 2026-05-17 — Sub-row immutable; chain composite row updated only via status transitions
  - DEC-2344 2026-05-17 — memory audit kinds: cuo.chain_planned, cuo.chain_step_started, cuo.chain_step_completed, cuo.chain_step_failed, cuo.chain_completed, cuo.chain_failed

language: rust 1.81
service: cyberos/services/cuo/
new_files:
  - services/cuo/migrations/0004_chain_walks.sql
  - services/cuo/src/chain/mod.rs
  - services/cuo/src/chain/topological_sorter.rs
  - services/cuo/src/chain/walker.rs
  - services/cuo/src/chain/cycle_detector.rs
  - services/cuo/src/handlers/chain_routes.rs
  - services/cuo/src/audit/chain_events.rs
  - services/cuo/tests/chain_status_enum_cardinality_test.rs
  - services/cuo/tests/topological_sort_test.rs
  - services/cuo/tests/cycle_detection_test.rs
  - services/cuo/tests/chain_step_failure_test.rs
  - services/cuo/tests/chain_audit_emission_test.rs

modified_files:
  - services/cuo/src/supervisor.rs

allowed_tools:
  - file_read: services/{cuo,skill}/**
  - file_write: services/cuo/{src,tests,migrations}/**
  - bash: cd services/cuo && cargo test chain

disallowed_tools:
  - execute chain with cycle (per DEC-2342)
  - mutate prior step row (per DEC-2343)

effort_hours: 10
subtasks:
  - "0.4h: 0004_chain_walks.sql"
  - "0.4h: chain/mod.rs"
  - "0.8h: topological_sorter.rs"
  - "1.0h: walker.rs"
  - "0.5h: cycle_detector.rs"
  - "0.5h: handlers/chain_routes.rs"
  - "0.4h: audit/chain_events.rs"
  - "3.5h: tests — 5 test files"
  - "2.0h: CDO UI for chain monitoring"
  - "0.5h: docs"

risk_if_skipped: "Without chain walker, complex multi-skill flows manual orchestration. Without DEC-2342 cycle detect, infinite loops. Without DEC-2340 composite audit, can't reconstruct chain post-hoc."
---

## §1 — Description (BCP-14 normative)

The CUO service **MUST** ship chain walker at `services/cuo/src/chain/` with topo sort + cycle detect + composite audit + sub-rows per step, 6 memory audit kinds.

1. **MUST** validate `chain_status` against closed enum per DEC-2341.

2. **MUST** sort topologically at `topological_sorter.rs::sort(skills_with_deps)` per DEC-2340:
- Build DAG from depends_on edges
- Kahn's algorithm or DFS topo
- Return ordered list of skills

3. **MUST** detect cycles at `cycle_detector.rs::has_cycle(graph)` per DEC-2342 — reject plan if true.

4. **MUST** walk at `walker.rs::walk(plan)` per DEC-2340:
- Execute skills in topo order
- Capture per-step sub-row (skill_id, started_at, completed_at, status, result)
- Update composite row status

5. **MUST** define tables at migration `0004`:
   ```sql
   CREATE TABLE cuo_chain_walks (
     chain_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_id UUID NOT NULL,
     plan_jsonb JSONB NOT NULL,
     status TEXT NOT NULL DEFAULT 'planning'
       CHECK (status IN ('planning','executing','completed','failed','partial_rolled_back')),
     started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE cuo_chain_walks ENABLE ROW LEVEL SECURITY;
   CREATE POLICY chain_rls ON cuo_chain_walks
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON cuo_chain_walks FROM cyberos_app;
   GRANT UPDATE (status, completed_at, failure_reason) ON cuo_chain_walks TO cyberos_app;

   CREATE TABLE cuo_chain_steps (
     step_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     chain_id UUID NOT NULL REFERENCES cuo_chain_walks(chain_id),
     step_order INT NOT NULL,
     skill_id TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','running','completed','failed','skipped')),
     started_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     result_jsonb JSONB,
     failure_reason TEXT,
     UNIQUE (chain_id, step_order)
   );
   ALTER TABLE cuo_chain_steps ENABLE ROW LEVEL SECURITY;
   CREATE POLICY steps_rls ON cuo_chain_steps
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON cuo_chain_steps FROM cyberos_app;
   GRANT UPDATE (status, started_at, completed_at, result_jsonb, failure_reason) ON cuo_chain_steps TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/cuo/chains              (CDO submits skills_with_deps)
   GET  /v1/cuo/chains/{id}         (status + steps)
   ```

7. **MUST** emit 6 memory audit kinds per DEC-2344. PII per TASK-MEMORY-111: result_jsonb SHA256.

8. **MUST** thread trace_id from plan → step → composite → audit.

9. **MUST NOT** execute chain with cycle per DEC-2342.

10. **MUST NOT** mutate prior step row per DEC-2343 (REVOKE UPDATE except status cols).

---

## §2 — Why this design

**Why topo sort (DEC-2340)?** Enforces depends_on order; prerequisites complete before dependents.

**Why cycle detect (DEC-2342)?** Cycles = infinite loops; must fail at plan time.

**Why composite + sub-rows (DEC-2340)?** Composite for chain-level summary; sub-rows for per-step debugging.

---

## §3 — API contract

Sample chain plan:
```json
POST /v1/cuo/chains
{
  "skills": [
    {"skill_id": "auth.user_lookup", "depends_on": []},
    {"skill_id": "calendar.list_events", "depends_on": ["auth.user_lookup"]},
    {"skill_id": "email.send", "depends_on": ["calendar.list_events"]}
  ]
}
```

Response:
```json
{
  "chain_id": "uuid",
  "status": "completed",
  "steps": [
    {"step_order": 0, "skill_id": "auth.user_lookup", "status": "completed"},
    {"step_order": 1, "skill_id": "calendar.list_events", "status": "completed"},
    {"step_order": 2, "skill_id": "email.send", "status": "completed"}
  ]
}
```

---

## §4 — Acceptance criteria
1. **chain_status enum cardinality 5**. 2. **Topo sort correct order**. 3. **Cycle detection rejects**. 4. **Per-step sub-row**. 5. **Composite audit row**. 6. **6 memory audit kinds emitted**. 7. **PII scrubbed (result SHA256)**. 8. **RLS denies cross-tenant**. 9. **Trace_id preserved**. 10. **Append-only via REVOKE except status cols**. 11. **UNIQUE(chain_id, step_order)**. 12. **Step failure → chain status updated**. 13. **Step status enum cardinality 5**. 14. **CDO-only chain submit**. 15. **Subsequent steps skipped on failure (no rollback yet — TASK-CUO-105)**. 16. **Plan JSON validated**. 17. **Empty skills list → 400**. 18. **Cycle test catches A→B→A**. 19. **Multiple paths handled (diamond DAG)**. 20. **Per-step skill_id from TASK-SKILL-001 registry**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn topo_sort_correct() {
    let skills = vec![
        ("c", vec!["b"]),
        ("a", vec![]),
        ("b", vec!["a"]),
    ];
    let sorted = topological_sorter::sort(&skills).unwrap();
    assert_eq!(sorted, vec!["a", "b", "c"]);
}

#[tokio::test]
async fn cycle_detection_rejects() {
    let skills = vec![
        ("a", vec!["b"]),
        ("b", vec!["a"]),
    ];
    let r = topological_sorter::sort(&skills);
    assert!(r.is_err());
}

#[tokio::test]
async fn step_failure_updates_chain() {
    let ctx = TestContext::with_3_step_chain_step_2_fails().await;
    ctx.run_chain(ctx.chain_id).await;
    let c = ctx.fetch_chain(ctx.chain_id).await;
    assert_eq!(c.status, "failed");
    let steps = ctx.fetch_steps(ctx.chain_id).await;
    assert_eq!(steps[1].status, "failed");
    assert_eq!(steps[2].status, "skipped");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CUO-101. **Downstream:** TASK-CUO-105 (rollback on failure). **Cross-module:** TASK-CUO-102 (checkpoint integration), TASK-SKILL-001 (skill registry), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cycle in plan | detector | reject 400 | fix plan |
| Empty skills | validate | 400 | provide skills |
| Skill not in registry | check | 404 | inherent |
| Step execution timeout | catch | step=failed | inherent |
| Cross-tenant chain | RLS | 403 | inherent |
| Mid-walk crash | resume | partial | retry from last completed |
| Concurrent chain submit | inherent | each isolated | inherent |
| Plan > 100 steps | validate | 400 | split |
| Step result > 5MB | validate | reject result | reduce |
| Bigint step_order overflow | unlikely | inherent | inherent |

## §11 — Implementation notes
- §11.1 Topo sort via Kahn's algorithm; cycle detection via in-degree check.
- §11.2 Walker runs steps sequentially v1; parallel walks (independent branches) future enhancement.
- §11.3 memory audit body: chain_id, step_order, skill_id, status; result SHA256.
- §11.4 Step failure → no further steps; TASK-CUO-105 handles rollback in next task.
- §11.5 Plan JSONB stored verbatim for replay.

---

*End of TASK-CUO-104 spec.*
