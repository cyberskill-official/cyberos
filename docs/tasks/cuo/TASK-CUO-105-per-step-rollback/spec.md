---
id: TASK-CUO-105
title: "CUO per-step rollback on chain failure — execute compensating actions in reverse order with partial-execution audit preserved"
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
module: cuo
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
related_tasks: [TASK-CUO-104, TASK-SKILL-001, TASK-MEMORY-111]
depends_on: [TASK-CUO-104]
blocks: []

source_pages:
  - website/docs/modules/cuo.html#rollback

source_decisions:
  - DEC-2370 2026-05-17 — On TASK-CUO-104 chain step failure, walker invokes registered compensating action for each completed step in REVERSE order; partial-execution preserved (not rolled back if no compensation registered)
  - DEC-2371 2026-05-17 — Closed enum `rollback_step_status` = {pending, compensating, compensated, no_compensation_registered, compensation_failed}; cardinality 5
  - DEC-2372 2026-05-17 — Skill registers optional compensating action at registration time (TASK-SKILL-001); if missing, step preserved + audit notes "no rollback path"
  - DEC-2373 2026-05-17 — Per-step compensation IMMUTABLE; chain composite gets partial_rolled_back status post-rollback completion
  - DEC-2374 2026-05-17 — memory audit kinds: cuo.rollback_initiated, cuo.rollback_step_started, cuo.rollback_step_compensated, cuo.rollback_step_no_compensation, cuo.rollback_completed, cuo.rollback_failed

language: rust 1.81
service: cyberos/services/cuo/
new_files:
  - services/cuo/migrations/0005_chain_rollbacks.sql
  - services/cuo/src/rollback/mod.rs
  - services/cuo/src/rollback/executor.rs
  - services/cuo/src/rollback/compensation_registry.rs
  - services/cuo/src/audit/rollback_events.rs
  - services/cuo/tests/rollback_step_status_enum_cardinality_test.rs
  - services/cuo/tests/rollback_reverse_order_test.rs
  - services/cuo/tests/rollback_no_compensation_preserved_test.rs
  - services/cuo/tests/rollback_compensation_failure_test.rs
  - services/cuo/tests/rollback_audit_emission_test.rs

modified_files:
  - services/cuo/src/chain/walker.rs

allowed_tools:
  - file_read: services/{cuo,skill}/**
  - file_write: services/cuo/{src,tests,migrations}/**
  - bash: cd services/cuo && cargo test rollback

disallowed_tools:
  - skip compensation if registered (per DEC-2370)
  - rollback in forward order (per DEC-2370)

effort_hours: 6
subtasks:
  - "0.3h: 0005_chain_rollbacks.sql"
  - "0.3h: rollback/mod.rs"
  - "0.7h: executor.rs"
  - "0.5h: compensation_registry.rs"
  - "0.3h: audit/rollback_events.rs"
  - "0.3h: walker.rs hook"
  - "2.6h: tests — 5 test files"
  - "1.0h: docs"

risk_if_skipped: "Without rollback, TASK-CUO-104 partial executions leave systems in inconsistent state. Without DEC-2370 reverse order, compensations applied wrong direction. Without DEC-2372 missing-compensation preservation, naive rollback breaks systems where compensation unsafe."
---

## §1 — Description (BCP-14 normative)

The CUO service **MUST** ship per-step rollback at `services/cuo/src/rollback/` triggered by TASK-CUO-104 step failure, compensations in reverse order, immutable audit, 6 memory audit kinds.

1. **MUST** validate `rollback_step_status` against closed enum per DEC-2371.

2. **MUST** trigger on TASK-CUO-104 step failure (chain status=failed) per DEC-2370.

3. **MUST** execute at `executor.rs::rollback(chain_id)` per DEC-2370:
- Fetch completed steps in REVERSE order
- For each: look up compensating action via TASK-SKILL-001 registry
- If registered: invoke compensation; mark compensated
- If not registered: mark no_compensation_registered + audit (don't fail rollback)

4. **MUST** lookup compensation at `compensation_registry.rs::get(skill_id)` per DEC-2372 — returns Option<compensating_skill_id>.

5. **MUST** define table at migration `0005`:
   ```sql
   CREATE TABLE cuo_chain_rollbacks (
     rollback_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     chain_id UUID NOT NULL REFERENCES cuo_chain_walks(chain_id),
     step_id UUID NOT NULL REFERENCES cuo_chain_steps(step_id),
     compensating_skill_id TEXT,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','compensating','compensated','no_compensation_registered','compensation_failed')),
     started_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (chain_id, step_id)
   );
   CREATE INDEX rollbacks_chain_idx ON cuo_chain_rollbacks(tenant_id, chain_id);
   ALTER TABLE cuo_chain_rollbacks ENABLE ROW LEVEL SECURITY;
   CREATE POLICY rollbacks_rls ON cuo_chain_rollbacks
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON cuo_chain_rollbacks FROM cyberos_app;
   GRANT UPDATE (status, started_at, completed_at, failure_reason) ON cuo_chain_rollbacks TO cyberos_app;
   ```

6. **MUST** update TASK-CUO-104 chain status to `partial_rolled_back` post-completion per DEC-2373.

7. **MUST** expose endpoints:
   ```text
   POST /v1/cuo/chains/{id}/rollback         (auto on step failure; CDO manual trigger)
   GET  /v1/cuo/chains/{id}/rollback-status  (per-step status)
   ```

8. **MUST** emit 6 memory audit kinds per DEC-2374. PII per TASK-MEMORY-111: failure_reason SHA256.

9. **MUST** thread trace_id from chain failure → rollback → audit.

10. **MUST NOT** skip compensation if registered per DEC-2370.

11. **MUST NOT** rollback in forward order per DEC-2370 (always reverse).

---

## §2 — Why this design

**Why reverse order (DEC-2370)?** Compensations undo state changes; must unwind in reverse to handle dependencies between steps.

**Why no-compensation preservation (DEC-2372)?** Some operations (sending email, charging card) have no safe undo; preserving + auditing is the only correct option.

**Why immutable rollback rows (DEC-2373)?** Audit lineage; rollback is a financial-grade operation.

---

## §3 — API contract

Sample rollback status:
```json
{
  "rollback_id": "uuid",
  "chain_id": "uuid",
  "steps": [
    {"step_order": 2, "skill_id": "email.send", "status": "no_compensation_registered"},
    {"step_order": 1, "skill_id": "calendar.book", "compensating_skill_id": "calendar.cancel", "status": "compensated"},
    {"step_order": 0, "skill_id": "auth.create_invite", "compensating_skill_id": "auth.revoke_invite", "status": "compensated"}
  ]
}
```

---

## §4 — Acceptance criteria
1. **rollback_step_status enum cardinality 5**. 2. **Triggered on TASK-CUO-104 step failure**. 3. **Reverse order execution**. 4. **Compensation looked up from TASK-SKILL-001 registry**. 5. **Missing compensation preserved with audit (not failure)**. 6. **6 memory audit kinds emitted**. 7. **PII scrubbed (failure_reason SHA256)**. 8. **RLS denies cross-tenant**. 9. **Trace_id preserved**. 10. **UNIQUE(chain_id, step_id)**. 11. **Chain status → partial_rolled_back post**. 12. **Append-only via REVOKE except status cols**. 13. **CDO manual trigger allowed**. 14. **Per-step compensation isolated (one failure doesn't halt rollback)**. 15. **compensation_failed audited sev-2**. 16. **Idempotent (re-trigger uses existing rollback_id per step)**. 17. **Rollback of failed step itself skipped (not completed)**. 18. **Skipped steps (TASK-CUO-104) also no-rollback**. 19. **Order matches inverse of step_order**. 20. **Rollback timeout per step 30s**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn rollback_in_reverse_order() {
    let ctx = TestContext::with_3_step_chain_step_3_fails().await;
    ctx.run_chain(ctx.chain_id).await;
    let rollbacks = ctx.fetch_rollbacks(ctx.chain_id).await;
    let order: Vec<i32> = rollbacks.iter().map(|r| r.step_order).collect();
    assert_eq!(order, vec![2, 1, 0]);  // reverse
}

#[tokio::test]
async fn no_compensation_preserved() {
    let ctx = TestContext::with_chain_step_no_compensation_registered().await;
    ctx.run_chain(ctx.chain_id).await;
    let r = ctx.fetch_rollback_for_step(ctx.step_id).await;
    assert_eq!(r.status, "no_compensation_registered");
}

#[tokio::test]
async fn compensation_failure_isolated() {
    let ctx = TestContext::with_3_step_chain_step2_compensation_fails().await;
    ctx.run_chain(ctx.chain_id).await;
    let rollbacks = ctx.fetch_rollbacks(ctx.chain_id).await;
    // step 2 compensation failed; step 1 still compensated
    assert_eq!(rollbacks.iter().find(|r| r.step_order == 2).unwrap().status, "compensation_failed");
    assert_eq!(rollbacks.iter().find(|r| r.step_order == 1).unwrap().status, "compensated");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CUO-104. **Cross-module:** TASK-SKILL-001 (compensation registry), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| No compensation registered | lookup miss | preserve + sev-3 audit | manual cleanup |
| Compensation execution fails | catch | compensation_failed sev-2 | manual or retry |
| Cross-tenant rollback | RLS | 403 | inherent |
| Concurrent rollback | UNIQUE on (chain, step) | second skipped | inherent |
| Mid-rollback crash | resume from last completed | partial | retry |
| Step still running on trigger | wait or abort | sev-2 | manual |
| Compensation skill missing | lookup fail | sev-2; treat as no-compensation | data fix |
| Rollback of skipped step | filter | no-op | inherent |
| Trace_id missing | sev-3 | use NIL_UUID | bug fix |
| Decimal precision N/A | inherent | inherent | inherent |

## §11 — Implementation notes
- §11.1 Compensation registered at TASK-SKILL-001 via optional `compensating_skill` field.
- §11.2 Walker invokes rollback via async task; allows main chain to mark failed quickly.
- §11.3 memory audit body: chain_id, step_id, status; reason SHA256.
- §11.4 Per-step timeout 30s; longer → mark compensation_failed.
- §11.5 Idempotency: re-trigger checks existing rollback row; only retries failed compensations.

---

*End of TASK-CUO-105 spec.*
