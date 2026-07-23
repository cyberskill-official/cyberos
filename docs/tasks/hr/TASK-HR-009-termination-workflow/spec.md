---
id: TASK-HR-009
title: "HR termination workflow — Good-Leaver / Bad-Leaver branch with CFO+CEO co-sign + ESOP forfeiture + access revocation cascade"
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
module: hr
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
related_tasks: [TASK-HR-001, TASK-AUTH-101, TASK-ESOP-005, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: [TASK-ESOP-005]

source_pages:
  - website/docs/modules/hr.html#termination
  # VN Labour Code Art. 34-37
  - https://thuvienphapluat.vn/

source_decisions:
  - DEC-1870 2026-05-17 — GL/BL branch determines ESOP forfeiture rate, severance computation, references — per board policy + VN Labour Code Art. 36
  - DEC-1871 2026-05-17 — Closed enum `termination_kind` = {good_leaver_voluntary, good_leaver_redundancy, good_leaver_retirement, bad_leaver_misconduct, bad_leaver_breach_contract, mutual_separation}; cardinality 6
  - DEC-1872 2026-05-17 — Closed enum `termination_stage` = {initiated, cfo_signed, ceo_signed, executed, dispute}; cardinality 5
  - DEC-1873 2026-05-17 — Dual sign-off: CFO + CEO required for execution; either rejection stops workflow
  - DEC-1874 2026-05-17 — On executed: cascade to TASK-ESOP-005 (vesting halt), TASK-AUTH-101 (deprovision), TASK-PORTAL-008 (DSAR offer), TASK-PROJ-013 (issue reassignment)
  - DEC-1875 2026-05-17 — memory audit kinds: hr.termination_initiated, hr.termination_signed, hr.termination_executed, hr.termination_disputed, hr.termination_failed

language: rust 1.81
service: cyberos/services/hr/
new_files:
  - services/hr/migrations/0008_terminations.sql
  - services/hr/src/termination/mod.rs
  - services/hr/src/termination/dual_sign_gate.rs
  - services/hr/src/termination/cascade_executor.rs
  - services/hr/src/handlers/termination_routes.rs
  - services/hr/src/audit/termination_events.rs
  - services/hr/tests/termination_kind_enum_cardinality_test.rs
  - services/hr/tests/termination_stage_enum_cardinality_test.rs
  - services/hr/tests/termination_dual_sign_required_test.rs
  - services/hr/tests/termination_gl_bl_branch_test.rs
  - services/hr/tests/termination_cascade_test.rs
  - services/hr/tests/termination_audit_emission_test.rs

modified_files:
  - services/hr/src/members.rs

allowed_tools:
  - file_read: services/{hr,esop,auth,portal,proj}/**
  - file_write: services/hr/{src,tests,migrations}/**
  - bash: cd services/hr && cargo test termination

disallowed_tools:
  - execute without dual sign (per DEC-1873)
  - skip cascade (per DEC-1874)

effort_hours: 8
subtasks:
  - "0.4h: 0008_terminations.sql"
  - "0.4h: termination/mod.rs"
  - "0.6h: dual_sign_gate.rs"
  - "1.0h: cascade_executor.rs"
  - "0.5h: handlers/termination_routes.rs"
  - "0.4h: audit/termination_events.rs"
  - "0.3h: members.rs hook"
  - "2.4h: tests — 6 test files"
  - "2.0h: CHRO+CEO+CFO UI for sign-off"

risk_if_skipped: "Without GL/BL branch + dual-sign, terminations executed by single signature → fraud risk. Without DEC-1874 cascade, ex-member retains AUTH access + ESOP vesting (massive risk). Without DEC-1870 GL/BL distinction, fair-treatment audit fails."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship termination workflow at `services/hr/src/termination/` with GL/BL branch + CFO+CEO dual sign + cascade to ESOP/AUTH/PORTAL/PROJ, 5 memory audit kinds.

1. **MUST** validate `termination_kind` against closed enum per DEC-1871, `termination_stage` per DEC-1872.

2. **MUST** require dual sign-off per DEC-1873 at `dual_sign_gate.rs::can_execute(termination)`:
- Both CFO + CEO must sign.
- Either rejection halts.
- Same person cannot sign both roles (separation of duties).

3. **MUST** branch GL/BL per DEC-1870:
- Good Leaver (voluntary/redundancy/retirement): full ESOP vesting up to termination_date; standard severance.
- Bad Leaver (misconduct/breach): ESOP forfeiture per board policy; severance per VN Art. 41 (no severance if cause).
- Mutual: negotiated.

4. **MUST** cascade on executed per DEC-1874 at `cascade_executor.rs::execute(termination)`:
- TASK-ESOP-005 vesting halt + forfeiture per GL/BL
- TASK-AUTH-101 deprovision (all roles revoked)
- TASK-PORTAL-008 offer DSAR export to ex-member
- TASK-PROJ-013 reassign open issues to manager

5. **MUST** define table at migration `0008`:
   ```sql
   CREATE TABLE hr_terminations (
     termination_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL UNIQUE,  -- one termination per member
     kind TEXT NOT NULL
       CHECK (kind IN ('good_leaver_voluntary','good_leaver_redundancy','good_leaver_retirement','bad_leaver_misconduct','bad_leaver_breach_contract','mutual_separation')),
     stage TEXT NOT NULL DEFAULT 'initiated'
       CHECK (stage IN ('initiated','cfo_signed','ceo_signed','executed','dispute')),
     termination_date DATE NOT NULL,
     initiated_by UUID NOT NULL,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     executed_at TIMESTAMPTZ,
     reason TEXT,
     dispute_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE hr_terminations ENABLE ROW LEVEL SECURITY;
   CREATE POLICY term_rls ON hr_terminations
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_terminations FROM cyberos_app;
   GRANT UPDATE (stage, cfo_signed_by, cfo_signed_at, ceo_signed_by, ceo_signed_at, executed_at, dispute_reason) ON hr_terminations TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST   /v1/hr/terminations                  (CHRO initiates)
   POST   /v1/hr/terminations/{id}/cfo-sign    (CFO)
   POST   /v1/hr/terminations/{id}/ceo-sign    (CEO)
   POST   /v1/hr/terminations/{id}/dispute     (member-self or counsel)
   GET    /v1/hr/terminations/{id}             (status)
   ```

7. **MUST** emit 5 memory audit kinds per DEC-1875. PII per TASK-MEMORY-111: reason+dispute_reason hashed; ids ok.

8. **MUST** thread trace_id from initiate → sign → execute → cascade → audit.

9. **MUST NOT** execute without both signatures per DEC-1873.

10. **MUST NOT** allow same person both CFO+CEO roles per DEC-1873.

11. **MUST NOT** skip cascade on execute per DEC-1874.

---

## §2 — Why this design

**Why GL/BL branch (DEC-1870)?** ESOP forfeiture + severance differ materially; legal exposure if applied wrong.

**Why dual sign (DEC-1873)?** Termination = significant financial+legal action; single-signer risk = fraud or bias.

**Why separation of duties (DEC-1873)?** Same person signing both roles = no real second check; explicit constraint.

**Why cascade (DEC-1874)?** Ex-member retaining access = security disaster; auto-revoke on execute closes the loop.

---

## §3 — API contract

```text
POST   /v1/hr/terminations                body: {member_id, kind, termination_date, reason}
POST   /v1/hr/terminations/{id}/cfo-sign
POST   /v1/hr/terminations/{id}/ceo-sign
POST   /v1/hr/terminations/{id}/dispute   body: {dispute_reason}
GET    /v1/hr/terminations/{id}
```

Sample termination:
```json
{
  "member_id": "uuid",
  "kind": "good_leaver_voluntary",
  "termination_date": "2026-06-30",
  "reason": "Career change to startup"
}
```

---

## §4 — Acceptance criteria
1. **kind enum cardinality 6**. 2. **stage enum cardinality 5**. 3. **CFO+CEO dual sign required**. 4. **Same person can't sign both**. 5. **GL → ESOP fully vested up to term_date**. 6. **BL → ESOP forfeiture per policy**. 7. **Cascade to ESOP-005 + AUTH-101 + PORTAL-008 + PROJ-013**. 8. **5 memory audit kinds emitted**. 9. **PII scrubbed (reason SHA256)**. 10. **RLS denies cross-tenant**. 11. **CHRO initiate, CFO sign, CEO sign — role-gated**. 12. **Trace_id preserved**. 13. **UNIQUE on member_id (one termination per)**. 14. **Append-only via REVOKE except 7 cols**. 15. **Dispute halts execution**. 16. **AUTH deprovision verified post-execute**. 17. **DSAR offer email sent**. 18. **Open issues reassigned to manager**. 19. **Cascade failure → sev-1 + rollback termination**. 20. **No termination during probation without specific kind**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required_for_execute() {
    let ctx = TestContext::with_initiated_termination().await;
    ctx.cfo_sign(ctx.term_id).await;
    let r = ctx.try_execute(ctx.term_id).await;
    assert!(r.is_err());  // CEO not signed
    ctx.ceo_sign(ctx.term_id).await;
    let r2 = ctx.execute(ctx.term_id).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn same_person_both_roles_rejected() {
    let ctx = TestContext::with_initiated_termination().await;
    ctx.cfo_sign_as(ctx.user_a, ctx.term_id).await;
    let r = ctx.try_ceo_sign_as(ctx.user_a, ctx.term_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn cascade_revokes_auth() {
    let ctx = TestContext::with_dual_signed_termination().await;
    ctx.execute(ctx.term_id).await;
    let auth_status = ctx.fetch_member_auth(ctx.member_id).await;
    assert_eq!(auth_status.active, false);
}

#[tokio::test]
async fn good_leaver_full_vesting() {
    let ctx = TestContext::with_member_partially_vested().await;
    let r = ctx.execute_gl_termination(ctx.member_id, "2026-06-30").await;
    let esop = ctx.fetch_esop_vesting(ctx.member_id).await;
    assert_eq!(esop.vested_at(ctx.term_date), esop.entitled_at(ctx.term_date));
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001. **Cross-module:** TASK-AUTH-101 (CFO/CEO/CHRO roles + deprovision), TASK-ESOP-005 (GL/BL branch), TASK-PORTAL-008 (DSAR), TASK-PROJ-013 (issue reassign), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| One signer missing | gate check | execute rejected 409 | get sign |
| Same-person dual | validate | reject 403 | use different signer |
| Cascade ESOP fails | sev-1 | rollback termination stage | retry execute |
| Cascade AUTH fails | sev-1 | termination halted | retry |
| Dispute during sign | stage→dispute | halt execute | resolve dispute |
| Termination date past | warn | allow (backdated terms exist) | inherent |
| Re-termination attempt | UNIQUE | 409 | inherent |
| Concurrent sign | UPDATE WHERE pending | first wins | inherent |
| BL without misconduct evidence | warn audit | still execute (HR judgment) | inherent |
| Reason missing | optional but warn | inherent | inherent |

## §11 — Implementation notes
- §11.1 Cascade is transactional — all-or-nothing; rollback termination if any cascade step fails.
- §11.2 BL ESOP forfeiture per board policy doc (stored in tenant config).
- §11.3 Severance computation: GL = per VN Art. 46 (0.5mo per year); BL = none if cause.
- §11.4 memory audit body: termination_id, member_id, kind, stage; reasons SHA256.
- §11.5 Dispute opens 30-day window before execute; can convert to mutual_separation if resolved.

---

*End of TASK-HR-009 spec.*
