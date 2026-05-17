---
id: FR-HR-007
title: "HR onboarding saga — orchestrates AUTH + TIME + LEARN + KB + CHAT + REW provisioning on member.active transition with compensating rollback"
module: HR
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-HR-001, FR-AUTH-101, FR-TIME-001, FR-LEARN-001, FR-KB-001, FR-CHAT-005, FR-REW-001, FR-BRAIN-111]
depends_on: [FR-HR-001]
blocks: []

source_pages:
  - website/docs/modules/hr.html#onboarding-saga

source_decisions:
  - DEC-1880 2026-05-17 — Saga pattern: orchestrates 6 module setups in defined order; each step idempotent; compensating rollback on any failure
  - DEC-1881 2026-05-17 — Closed enum `saga_step` = {auth_provision, time_init, learn_assign_starter, kb_grant_scope, chat_create_channel, rew_init_baseline}; cardinality 6
  - DEC-1882 2026-05-17 — Closed enum `saga_status` = {pending, in_progress, completed, failed, compensating, compensated}; cardinality 6
  - DEC-1883 2026-05-17 — Trigger: member.active transition (from probation/inactive); FR-HR-002 contract type must be set
  - DEC-1884 2026-05-17 — Compensation: each step has reverse op (auth_deprovision, etc.); compensation runs in REVERSE order on failure
  - DEC-1885 2026-05-17 — BRAIN audit kinds: hr.onboarding_saga_started, hr.onboarding_step_completed, hr.onboarding_step_failed, hr.onboarding_compensation_started, hr.onboarding_saga_completed, hr.onboarding_saga_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/hr/
  new_files:
    - services/hr/migrations/0009_onboarding_sagas.sql
    - services/hr/src/onboarding/mod.rs
    - services/hr/src/onboarding/saga_orchestrator.rs
    - services/hr/src/onboarding/step_handlers.rs
    - services/hr/src/onboarding/compensation.rs
    - services/hr/src/handlers/onboarding_routes.rs
    - services/hr/src/audit/onboarding_events.rs
    - services/hr/tests/onboarding_full_flow_test.rs
    - services/hr/tests/onboarding_step_failure_rollback_test.rs
    - services/hr/tests/onboarding_step_enum_cardinality_test.rs
    - services/hr/tests/onboarding_status_enum_cardinality_test.rs
    - services/hr/tests/onboarding_idempotent_test.rs
    - services/hr/tests/onboarding_audit_emission_test.rs

  modified_files:
    - services/hr/src/members.rs

  allowed_tools:
    - file_read: services/{hr,auth,time,learn,kb,chat,rew}/**
    - file_write: services/hr/{src,tests,migrations}/**
    - bash: cd services/hr && cargo test onboarding

  disallowed_tools:
    - skip compensation on failure (per DEC-1884)
    - skip steps (per DEC-1880)

effort_hours: 10
sub_tasks:
  - "0.4h: 0009_onboarding_sagas.sql"
  - "0.5h: onboarding/mod.rs"
  - "1.5h: saga_orchestrator.rs"
  - "2.0h: step_handlers.rs (6 step + 6 compensations)"
  - "0.8h: compensation.rs"
  - "0.5h: handlers/onboarding_routes.rs"
  - "0.4h: audit/onboarding_events.rs"
  - "0.3h: members.rs hook"
  - "3.0h: tests — 6 test files"
  - "0.6h: docs"

risk_if_skipped: "Without onboarding saga, manual setup misses steps (member can't log in, no time-tracking, etc.). Without DEC-1884 compensation, mid-failure leaves member half-provisioned (auth but no chat = lockout). Without DEC-1880 ordering, race conditions (chat needs auth)."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship onboarding saga at `services/hr/src/onboarding/` orchestrating 6 module provisions in order with compensating rollback, immutable saga state, 6 BRAIN audit kinds.

1. **MUST** trigger on `member.status` transition to 'active' per DEC-1883 — hook at `services/hr/src/members.rs`.

2. **MUST** validate `saga_step` per DEC-1881, `saga_status` per DEC-1882.

3. **MUST** execute steps in fixed order per DEC-1880 at `saga_orchestrator.rs::run(member)`:
   1. auth_provision — FR-AUTH-101 create user with role per contract type
   2. time_init — FR-TIME-001 create member time profile
   3. learn_assign_starter — FR-LEARN-001 assign starter pack
   4. kb_grant_scope — FR-KB-001 grant team scope
   5. chat_create_channel — FR-CHAT-005 add to team channels
   6. rew_init_baseline — FR-REW-001 init comp record

4. **MUST** be idempotent per DEC-1880 — each step checks "already done" before acting.

5. **MUST** compensate per DEC-1884 on any failure at `compensation.rs::compensate(saga, failed_step)`:
   - Run reverse ops in REVERSE order of completed steps.
   - E.g. if rew_init fails, compensate chat → kb → learn → time → auth.

6. **MUST** define table at migration `0009`:
   ```sql
   CREATE TABLE hr_onboarding_sagas (
     saga_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL UNIQUE,
     current_step TEXT
       CHECK (current_step IS NULL OR current_step IN
         ('auth_provision','time_init','learn_assign_starter','kb_grant_scope','chat_create_channel','rew_init_baseline')),
     completed_steps TEXT[] NOT NULL DEFAULT '{}',
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','in_progress','completed','failed','compensating','compensated')),
     failed_step TEXT,
     failure_reason TEXT,
     compensation_log JSONB,
     trace_id CHAR(32),
     started_at TIMESTAMPTZ,
     completed_at TIMESTAMPTZ,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE hr_onboarding_sagas ENABLE ROW LEVEL SECURITY;
   CREATE POLICY saga_rls ON hr_onboarding_sagas
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_onboarding_sagas FROM cyberos_app;
   GRANT UPDATE (current_step, completed_steps, status, failed_step,
                 failure_reason, compensation_log, started_at, completed_at) ON hr_onboarding_sagas TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/hr/onboarding/start                (CHRO; manual trigger if hook missed)
   POST   /v1/hr/onboarding/{saga_id}/retry      (CHRO; resume failed at step)
   POST   /v1/hr/onboarding/{saga_id}/compensate (CHRO; force rollback)
   GET    /v1/hr/onboarding/sagas/{id}            (status)
   ```

8. **MUST** emit 6 BRAIN audit kinds per DEC-1885. PII per FR-BRAIN-111: member_id (uuid) ok; failure_reason hashed.

9. **MUST** thread trace_id across all 6 steps + compensation; visible in each module's audit chain.

10. **MUST NOT** skip compensation on failure per DEC-1884 — must execute completed_steps in reverse.

11. **MUST NOT** skip steps per DEC-1880 — ordering matters (chat needs auth user).

12. **MUST NOT** double-onboard same member (UNIQUE on member_id).

---

## §2 — Why this design

**Why saga pattern (DEC-1880)?** Cross-module orchestration without 2PC; each step transactional + compensable.

**Why fixed order (DEC-1880)?** Dependencies: chat needs auth user; learn needs member profile; rew needs contract type. Order is contract.

**Why compensation (DEC-1884)?** Partial state = worse than no state; rollback ensures clean retry.

**Why idempotency (DEC-1880)?** Saga retries on transient failure; double-execution must be safe.

---

## §3 — API contract

Sample saga state:
```json
{
  "saga_id": "uuid",
  "member_id": "uuid",
  "current_step": "kb_grant_scope",
  "completed_steps": ["auth_provision", "time_init", "learn_assign_starter"],
  "status": "in_progress",
  "started_at": "2026-05-17T10:00:00Z"
}
```

Failure state:
```json
{
  "saga_id": "uuid",
  "current_step": "rew_init_baseline",
  "completed_steps": ["auth_provision", "time_init", "learn_assign_starter", "kb_grant_scope", "chat_create_channel"],
  "status": "compensating",
  "failed_step": "rew_init_baseline",
  "failure_reason": "REW partner API timeout",
  "compensation_log": [{"step": "chat_create_channel", "compensated_at": "..."}]
}
```

---

## §4 — Acceptance criteria
1. **6-step enum + cardinality test**. 2. **6-status enum + cardinality test**. 3. **Triggered on member.status → active**. 4. **Steps execute in order**. 5. **Idempotent steps**. 6. **Compensation on failure (reverse order)**. 7. **UNIQUE on member_id**. 8. **6 BRAIN audit kinds emitted**. 9. **PII scrubbed (failure_reason SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved across modules**. 12. **CHRO-only manual trigger/retry/compensate**. 13. **Append-only via REVOKE except status cols**. 14. **Retry resumes from failed step**. 15. **Compensation log JSONB tracks each reverse op**. 16. **Saga state queryable**. 17. **Contract type required (else error)**. 18. **Saga timeout 30min (sev-1 + compensate)**. 19. **Concurrent triggers UNIQUE-rejected**. 20. **All 6 modules return success on completion**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn full_flow_completes() {
    let ctx = TestContext::with_new_member_and_contract().await;
    ctx.activate_member(ctx.member_id).await;
    let saga = ctx.wait_for_saga_complete(ctx.member_id).await;
    assert_eq!(saga.status, "completed");
    assert_eq!(saga.completed_steps.len(), 6);
}

#[tokio::test]
async fn step_failure_triggers_compensation() {
    let ctx = TestContext::with_new_member_rew_will_fail().await;
    ctx.activate_member(ctx.member_id).await;
    let saga = ctx.wait_for_saga_state(ctx.member_id, "compensated").await;
    let auth_status = ctx.fetch_auth(ctx.member_id).await;
    assert!(auth_status.is_none());  // compensated back to nothing
}

#[tokio::test]
async fn idempotent_double_trigger() {
    let ctx = TestContext::with_new_member().await;
    ctx.activate_member(ctx.member_id).await;
    let r = ctx.try_activate_again(ctx.member_id).await;
    let sagas = ctx.fetch_sagas(ctx.member_id).await;
    assert_eq!(sagas.len(), 1);
}

#[tokio::test]
async fn trace_id_propagated() {
    let ctx = TestContext::with_traceable_activation().await;
    ctx.activate_member(ctx.member_id).await;
    let saga = ctx.wait_complete(ctx.member_id).await;
    let auth_audit = ctx.fetch_brain_audit("auth.user_created", saga.member_id).await;
    assert_eq!(auth_audit.trace_id, saga.trace_id);
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-001.
**Cross-module:** FR-AUTH-101 (provision), FR-TIME-001 (init), FR-LEARN-001 (starter pack), FR-KB-001 (scope grant), FR-CHAT-005 (channel), FR-REW-001 (comp baseline), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Step API timeout | retry 3x | failed; sev-1; compensate | manual retry |
| Step idempotency violated | step handler | sev-1; investigate | bug fix |
| Compensation step fails | sev-1 alert | partial compensation logged | manual intervention |
| Saga timeout 30min | cron check | sev-1; compensate | inherent |
| Concurrent activation | UNIQUE | second skipped | inherent |
| Member already onboarded | UNIQUE | 409 | use retry endpoint |
| Contract type missing | early validate | 400 | set contract first |
| Cross-tenant trigger | RLS | 403 | inherent |
| Module unavailable | sev-2; retry | pause saga | inherent |
| Compensation log corruption | sev-1 audit | manual review | bug fix |

## §11 — Implementation notes
- §11.1 Each step is async fn returning Result; step handler module groups all 12 (6 do + 6 undo).
- §11.2 Orchestrator state machine: pending → in_progress → (completed | failed → compensating → compensated).
- §11.3 Trace_id from saga propagates to each module's audit row (cross-module observability).
- §11.4 Saga timeout cron checks for status='in_progress' AND started_at < now() - 30min.
- §11.5 BRAIN audit body: saga_id, member_id, step, completed_count; failure_reason SHA256.

---

*End of FR-HR-007 spec.*
