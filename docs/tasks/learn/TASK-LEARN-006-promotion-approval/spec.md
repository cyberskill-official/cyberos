---
id: TASK-LEARN-006
title: "LEARN promotion approval workflow — CEO + CHRO sign-off after council vote with cascade to HR + REW comp band update"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
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
related_tasks: [TASK-LEARN-004, TASK-LEARN-005, TASK-HR-001, TASK-REW-001, TASK-MEMORY-111]
depends_on: [TASK-LEARN-004]
blocks: []

source_pages:
  - website/docs/modules/learn.html#promotion-approval

source_decisions:
  - DEC-2130 2026-05-17 — Promotion triggered by TASK-LEARN-004 council recommendation=promote; CEO+CHRO dual-sign converts to action
  - DEC-2131 2026-05-17 — Closed enum `promotion_status` = {pending_council, council_recommended, ceo_signed, chro_signed, approved, declined, executed}; cardinality 7
  - DEC-2132 2026-05-17 — Cascade on executed: TASK-HR-001 mastery_level updated; TASK-REW-001 comp band updated; TASK-CHAT-005 announcement
  - DEC-2133 2026-05-17 — Same-person dual-sign rejected (separation of duties)
  - DEC-2134 2026-05-17 — memory audit kinds: learn.promotion_initiated, learn.promotion_signed, learn.promotion_executed, learn.promotion_declined, learn.promotion_cascade_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/learn/
  new_files:
    - services/learn/migrations/0006_promotions.sql
    - services/learn/src/promotion/mod.rs
    - services/learn/src/promotion/dual_sign_gate.rs
    - services/learn/src/promotion/cascade.rs
    - services/learn/src/handlers/promotion_routes.rs
    - services/learn/src/audit/promotion_events.rs
    - services/learn/tests/promotion_status_enum_cardinality_test.rs
    - services/learn/tests/promotion_dual_sign_test.rs
    - services/learn/tests/promotion_same_person_rejected_test.rs
    - services/learn/tests/promotion_cascade_test.rs
    - services/learn/tests/promotion_audit_emission_test.rs

  modified_files:
    - services/learn/src/lib.rs

  allowed_tools:
    - file_read: services/{learn,hr,rew,chat}/**
    - file_write: services/learn/{src,tests,migrations}/**
    - bash: cd services/learn && cargo test promotion

  disallowed_tools:
    - execute without dual-sign (per DEC-2130)
    - skip cascade (per DEC-2132)

effort_hours: 5
subtasks:
  - "0.3h: 0006_promotions.sql"
  - "0.3h: promotion/mod.rs"
  - "0.4h: dual_sign_gate.rs"
  - "0.6h: cascade.rs (HR + REW + CHAT)"
  - "0.4h: handlers/promotion_routes.rs"
  - "0.3h: audit/promotion_events.rs"
  - "1.8h: tests — 5 test files"
  - "0.9h: CEO+CHRO UI for sign + docs"

risk_if_skipped: "Without dual-sign workflow, promotions execute by single signature → governance failure. Without DEC-2132 cascade, HR mastery + REW comp out of sync."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship promotion workflow at `services/learn/src/promotion/` with CEO+CHRO dual-sign + cascade to HR + REW + CHAT, 5 memory audit kinds.

1. **MUST** validate `promotion_status` against closed enum per DEC-2131.

2. **MUST** initiate per DEC-2130 when TASK-LEARN-004 council completes with recommendation=promote.

3. **MUST** enforce dual-sign at `dual_sign_gate.rs::can_execute(promotion)` per DEC-2133:
   - Both CEO + CHRO signed
   - Same person can't sign both (separation of duties)

4. **MUST** cascade on executed at `cascade.rs::execute(promotion)` per DEC-2132:
   - TASK-HR-001 update member.mastery_level (transactional)
   - TASK-REW-001 update comp_band (transactional)
   - TASK-CHAT-005 announce in #all (non-blocking)
   - Rollback all on any failure

5. **MUST** define table at migration `0006`:
   ```sql
   CREATE TABLE learn_promotions (
     promotion_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     candidate_member_id UUID NOT NULL,
     council_id UUID NOT NULL,
     skill_id UUID NOT NULL,
     from_level INT NOT NULL CHECK (from_level >= 1 AND from_level <= 5),
     to_level INT NOT NULL CHECK (to_level >= 1 AND to_level <= 5),
     status TEXT NOT NULL DEFAULT 'pending_council'
       CHECK (status IN ('pending_council','council_recommended','ceo_signed','chro_signed','approved','declined','executed')),
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     chro_signed_by UUID,
     chro_signed_at TIMESTAMPTZ,
     executed_at TIMESTAMPTZ,
     decline_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (council_id)
   );
   ALTER TABLE learn_promotions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY promotions_rls ON learn_promotions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_promotions FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, chro_signed_by, chro_signed_at, executed_at, decline_reason) ON learn_promotions TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/learn/promotions/{id}/ceo-sign
   POST /v1/learn/promotions/{id}/chro-sign
   POST /v1/learn/promotions/{id}/decline    body: {reason}
   GET  /v1/learn/promotions/{id}
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2134. PII per TASK-MEMORY-111: decline_reason SHA-256 hashed; status enum + level ints ok.

8. **MUST** thread trace_id from initiate → sign → execute → cascade → audit.

9. **MUST NOT** execute without dual-sign per DEC-2130.

10. **MUST NOT** allow same-person dual-sign per DEC-2133.

11. **MUST NOT** skip cascade per DEC-2132.

---

## §2 — Why this design

**Why dual-sign (DEC-2130)?** Promotion = career + financial impact; single-signer governance gap.

**Why separation of duties (DEC-2133)?** Same person both roles = no real second check.

**Why cascade transactional (DEC-2132)?** HR + REW + announcement must align; partial = embarrassment.

---

## §3 — API contract

Sample promotion state:
```json
{
  "promotion_id": "uuid",
  "candidate_member_id": "uuid",
  "from_level": 3,
  "to_level": 4,
  "skill_id": "uuid",
  "status": "ceo_signed",
  "ceo_signed_at": "2026-05-17T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **promotion_status enum cardinality 7**. 2. **CEO+CHRO dual-sign required**. 3. **Same person rejected**. 4. **Cascade transactional**. 5. **HR mastery updated**. 6. **REW comp band updated**. 7. **CHAT announcement non-blocking**. 8. **UNIQUE(council_id) — one promotion per council**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (decline_reason SHA256)**. 11. **RLS denies cross-tenant**. 12. **CEO/CHRO role only**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE except status cols**. 15. **Decline allowed at any pre-execute stage**. 16. **from_level + to_level CHECK 1-5**. 17. **to_level > from_level enforced**. 18. **Status workflow enforced**. 19. **Cascade failure → rollback**. 20. **Council recommend=decline blocks promotion init**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required() {
    let ctx = TestContext::with_promotion_pending().await;
    ctx.ceo_sign(ctx.promotion_id).await;
    let r = ctx.try_execute(ctx.promotion_id).await;
    assert!(r.is_err());  // CHRO missing
    ctx.chro_sign(ctx.promotion_id).await;
    let r2 = ctx.execute(ctx.promotion_id).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn same_person_rejected() {
    let ctx = TestContext::with_promotion_pending().await;
    ctx.ceo_sign_as(ctx.user_a, ctx.promotion_id).await;
    let r = ctx.try_chro_sign_as(ctx.user_a, ctx.promotion_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn cascade_updates_hr_and_rew() {
    let ctx = TestContext::with_dual_signed_promotion().await;
    ctx.execute(ctx.promotion_id).await;
    let hr_level = ctx.fetch_hr_mastery(ctx.member_id, ctx.skill_id).await;
    let rew_band = ctx.fetch_rew_band(ctx.member_id).await;
    assert_eq!(hr_level, 4);
    assert!(rew_band.updated_at > ctx.before_execute_time);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-LEARN-004.
**Cross-module:** TASK-LEARN-005 (recommendation source), TASK-HR-001, TASK-REW-001, TASK-CHAT-005, TASK-AUTH-101 (CEO/CHRO), TASK-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| One signer missing | gate | reject execute | wait |
| Same-person dual | validate | 403 | different signer |
| HR cascade fails | rollback | sev-1; status=approved-pending | retry |
| REW cascade fails | rollback | sev-1 | retry |
| CHAT announce fails | non-blocking | sev-3 | inherent |
| Duplicate promotion per council | UNIQUE | 409 | inherent |
| Council recommend=decline | block init | 412 | re-convene |
| Status workflow violation | check | 409 | follow workflow |
| Cross-tenant promotion | RLS | 403 | inherent |
| from_level >= to_level | CHECK | 400 | fix levels |

## §11 — Implementation notes
- §11.1 Cascade order: HR (least-side-effect first) → REW (financial) → CHAT (visible last).
- §11.2 Cascade transaction wraps HR + REW; CHAT outside (non-blocking).
- §11.3 memory audit body: promotion_id, member_id, from/to level, status; decline SHA256.
- §11.4 Decline at any pre-execute stage; council can be re-convened if needed.
- §11.5 Future: peer announcement + congrats flow via TASK-CHAT-005 reactions.

---

*End of TASK-LEARN-006 spec.*
