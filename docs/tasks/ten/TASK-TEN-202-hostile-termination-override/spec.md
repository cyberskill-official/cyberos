---
id: TASK-TEN-202
title: "TEN hostile-termination override — legal-trigger fast-track with CEO+CLO+CSO triple-sign for hostile actor offboarding"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: TEN
priority: p1
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CISO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-104, TASK-HR-009, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-TEN-104]
blocks: []

source_pages:
  - website/docs/modules/ten.html#hostile-termination

source_decisions:
  - DEC-2410 2026-05-17 — Fast-track TASK-HR-009 termination bypassing standard CEO+CFO co-sign with CEO+CLO+CSO triple-sign for hostile actors (legal trigger justification required)
  - DEC-2411 2026-05-17 — Closed enum `hostile_trigger_kind` = {data_exfil_evidence, harassment_violation, criminal_charge, immediate_threat, regulatory_demand}; cardinality 5
  - DEC-2412 2026-05-17 — Legal trigger documentation required (case ref + brief description in tracked legal doc via TASK-DOC-001)
  - DEC-2413 2026-05-17 — Override is sev-1 + CISO email; CISO can challenge within 24h
  - DEC-2414 2026-05-17 — memory audit kinds: ten.hostile_override_initiated, ten.hostile_override_signed, ten.hostile_override_executed, ten.hostile_override_challenged, ten.hostile_override_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0011_hostile_overrides.sql
    - services/ten/src/hostile/mod.rs
    - services/ten/src/hostile/triple_sign_gate.rs
    - services/ten/src/hostile/cascade.rs
    - services/ten/src/handlers/hostile_routes.rs
    - services/ten/src/audit/hostile_events.rs
    - services/ten/tests/hostile_trigger_enum_cardinality_test.rs
    - services/ten/tests/hostile_triple_sign_test.rs
    - services/ten/tests/hostile_legal_doc_required_test.rs
    - services/ten/tests/hostile_audit_emission_test.rs

  modified_files:
    - services/ten/src/lib.rs

  allowed_tools:
    - file_read: services/{ten,hr,auth,doc}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cd services/ten && cargo test hostile

  disallowed_tools:
    - execute without triple-sign (per DEC-2410)
    - skip legal doc (per DEC-2412)
    - bypass CISO notification (per DEC-2413)

effort_hours: 5
subtasks:
  - "0.3h: 0011_hostile_overrides.sql"
  - "0.3h: hostile/mod.rs"
  - "0.4h: triple_sign_gate.rs"
  - "0.5h: cascade.rs (HR-009 fast-path)"
  - "0.4h: handlers/hostile_routes.rs"
  - "0.3h: audit/hostile_events.rs"
  - "1.8h: tests — 4 test files"
  - "1.0h: docs + CISO challenge UI"

risk_if_skipped: "Without fast-track, hostile actor retains access during standard offboarding (data exfil window). Without DEC-2410 CEO+CLO+CSO sign, single-signer abuse. Without DEC-2413 CISO challenge, no oversight on potentially-abused override."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship hostile-termination override at `services/ten/src/hostile/` with CEO+CLO+CSO triple-sign + legal trigger doc + sev-1 CISO challenge window, 5 memory audit kinds.

1. **MUST** validate `hostile_trigger_kind` against closed enum per DEC-2411.

2. **MUST** require triple-sign at `triple_sign_gate.rs::can_execute(override)` per DEC-2410 — CEO + CLO + CSO; same-person across slots rejected.

3. **MUST** require legal trigger doc via TASK-DOC-001 per DEC-2412 — case ref + brief description.

4. **MUST** cascade via fast-path TASK-HR-009 termination + AUTH revocation at `cascade.rs::execute(override)` per DEC-2410 — bypass standard CEO+CFO sign (this override IS the sign).

5. **MUST** emit sev-1 CISO notification per DEC-2413 — 24h challenge window via `POST /v1/ten/hostile-overrides/{id}/challenge`.

6. **MUST** define table at migration `0011`:
   ```sql
   CREATE TABLE ten_hostile_overrides (
     override_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     subject_member_id UUID NOT NULL,
     trigger_kind TEXT NOT NULL
       CHECK (trigger_kind IN ('data_exfil_evidence','harassment_violation','criminal_charge','immediate_threat','regulatory_demand')),
     legal_doc_id UUID NOT NULL,  -- TASK-DOC-001 ref
     case_ref TEXT NOT NULL,
     brief_description TEXT NOT NULL,
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     clo_signed_by UUID,
     clo_signed_at TIMESTAMPTZ,
     cso_signed_by UUID,
     cso_signed_at TIMESTAMPTZ,
     status TEXT NOT NULL DEFAULT 'initiated'
       CHECK (status IN ('initiated','triple_signed','executed','challenged','reversed','failed')),
     executed_at TIMESTAMPTZ,
     ciso_challenge_deadline TIMESTAMPTZ,
     challenged_by UUID,
     challenged_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE ten_hostile_overrides ENABLE ROW LEVEL SECURITY;
   CREATE POLICY hostile_rls ON ten_hostile_overrides
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON ten_hostile_overrides FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, clo_signed_by, clo_signed_at, cso_signed_by, cso_signed_at, executed_at, ciso_challenge_deadline, challenged_by, challenged_reason) ON ten_hostile_overrides TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST /v1/ten/hostile-overrides                 (CEO/CLO/CSO initiates)
   POST /v1/ten/hostile-overrides/{id}/ceo-sign
   POST /v1/ten/hostile-overrides/{id}/clo-sign
   POST /v1/ten/hostile-overrides/{id}/cso-sign
   POST /v1/ten/hostile-overrides/{id}/execute    (auto on triple-sign)
   POST /v1/ten/hostile-overrides/{id}/challenge  (CISO within 24h)
   ```

8. **MUST** emit 5 memory audit kinds per DEC-2414. PII per TASK-MEMORY-111: brief_description SHA256.

9. **MUST** thread trace_id from initiate → sign → execute → challenge → audit.

10. **MUST NOT** execute without triple-sign per DEC-2410.

11. **MUST NOT** skip legal doc per DEC-2412 (TASK-DOC-001 FK enforced).

12. **MUST NOT** bypass CISO notification per DEC-2413.

---

## §2 — Why this design

**Why triple-sign (DEC-2410)?** Standard offboarding requires CEO+CFO; hostile fast-track substitutes CFO for CLO+CSO (legal + security). Even stronger gate.

**Why legal doc requirement (DEC-2412)?** Audit defense — "why was this person force-terminated?" must have case ref + description in writable doc.

**Why CISO challenge (DEC-2413)?** Prevent abuse (e.g. CEO+CLO+CSO collude to fire whistleblower). CISO can reverse within 24h.

---

## §3 — API contract

Sample override:
```json
POST /v1/ten/hostile-overrides
{
  "subject_member_id": "uuid",
  "trigger_kind": "data_exfil_evidence",
  "legal_doc_id": "uuid-case-file-pdf",
  "case_ref": "INC-2026-042",
  "brief_description": "Forensic evidence of mass S3 download to external account"
}
```

---

## §4 — Acceptance criteria
1. **hostile_trigger_kind enum cardinality 5**. 2. **CEO+CLO+CSO triple-sign**. 3. **Same-person across slots rejected**. 4. **Legal doc required (FK)**. 5. **case_ref + brief_description required**. 6. **Cascade to TASK-HR-009 fast-path**. 7. **AUTH revocation immediate**. 8. **sev-1 memory audit + CISO email**. 9. **24h CISO challenge window**. 10. **Challenge reverses + restores access**. 11. **5 memory audit kinds emitted**. 12. **PII scrubbed (description SHA256)**. 13. **RLS denies cross-tenant**. 14. **Trace_id preserved**. 15. **Append-only via REVOKE except status cols**. 16. **CEO/CLO/CSO + CISO role gates**. 17. **Status workflow enforced**. 18. **Reversal restores grants + access**. 19. **Override doesn't bypass labor law (CLO ensures)**. 20. **Audit log accessible to board**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn triple_sign_required() {
    let ctx = TestContext::with_legal_doc().await;
    let r = ctx.init_hostile(ctx.member, "data_exfil_evidence", ctx.doc_id).await;
    ctx.ceo_sign(r.id).await;
    ctx.clo_sign(r.id).await;
    let try_exec = ctx.try_execute(r.id).await;
    assert!(try_exec.is_err());  // CSO missing
    ctx.cso_sign(r.id).await;
    let exec = ctx.execute(r.id).await;
    assert!(exec.is_ok());
}

#[tokio::test]
async fn legal_doc_required() {
    let ctx = TestContext::with_member().await;
    let r = ctx.try_init_hostile_no_doc(ctx.member).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn ciso_challenge_reverses() {
    let ctx = TestContext::with_executed_override().await;
    ctx.ciso_challenge(ctx.override_id, "investigation incomplete").await;
    let o = ctx.fetch_override(ctx.override_id).await;
    assert_eq!(o.status, "challenged");
    let member = ctx.fetch_member(ctx.subject_member_id).await;
    assert_eq!(member.status, "active");  // restored
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-TEN-104.
**Cross-module:** TASK-HR-009 (termination cascade), TASK-AUTH-101 (roles), TASK-DOC-001 (legal doc), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| One sig missing | gate | reject exec | get sig |
| Same-person dual | validate | 403 | different signer |
| Legal doc missing | FK | reject | upload doc |
| HR-009 cascade fails | rollback | sev-1 | manual fix |
| AUTH revoke fails | sev-1 | retry | manual revoke |
| Cross-tenant override | RLS | 403 | inherent |
| CISO not notified | sev-1 alert | inherent | escalate |
| Challenge past 24h | reject | inherent | new override for re-fire |
| Concurrent sign | UPDATE WHERE | first wins | inherent |
| Reversal mid-execute | sev-1 | manual restore | inherent |

## §11 — Implementation notes
- §11.1 Triple-sign sequence: CEO + CLO + CSO; CISO not in sign chain (challenger role separate).
- §11.2 Cascade uses TASK-HR-009 internal API with `bypass_standard_co_sign=true` flag.
- §11.3 memory audit body: override_id, trigger_kind, status; description SHA256.
- §11.4 CISO email via TASK-EMAIL-009 with override URL + 24h countdown.
- §11.5 Reversal restores HR-009 termination + AUTH access; ESOP not reversed (out of scope — board decides).

---

*End of TASK-TEN-202 spec.*
