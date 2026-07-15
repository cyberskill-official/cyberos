---
id: TASK-ESOP-005
title: "ESOP Good/Bad Leaver branch on HR offboarding — CFO+CEO co-sign to apply forfeiture/acceleration per termination_kind"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: ESOP
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-009, TASK-ESOP-001, TASK-ESOP-002, TASK-MEMORY-111]
depends_on: [TASK-HR-009]
blocks: []

source_pages:
  - website/docs/modules/esop.html#gl-bl

source_decisions:
  - DEC-2290 2026-05-17 — Triggered by TASK-HR-009 termination.executed; GL = full vesting up to term_date; BL = forfeit unvested + optional vested forfeit per policy
  - DEC-2291 2026-05-17 — Closed enum `leaver_outcome` = {good_leaver_full_vest, good_leaver_pro_rated, bad_leaver_unvested_forfeit, bad_leaver_full_forfeit, mutual_negotiated}; cardinality 5
  - DEC-2292 2026-05-17 — CFO+CEO dual-sign required to commit outcome (same-person rejected)
  - DEC-2293 2026-05-17 — Outcome IMMUTABLE post-committed; corrections require board sign + new outcome row
  - DEC-2294 2026-05-17 — memory audit kinds: esop.leaver_outcome_drafted, esop.leaver_outcome_signed, esop.leaver_outcome_committed, esop.leaver_forfeiture_applied, esop.leaver_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0005_leaver_outcomes.sql
    - services/esop/src/leaver/mod.rs
    - services/esop/src/leaver/dual_sign_gate.rs
    - services/esop/src/leaver/forfeiture_executor.rs
    - services/esop/src/handlers/leaver_routes.rs
    - services/esop/src/audit/leaver_events.rs
    - services/esop/tests/leaver_outcome_enum_cardinality_test.rs
    - services/esop/tests/leaver_dual_sign_test.rs
    - services/esop/tests/leaver_gl_full_vest_test.rs
    - services/esop/tests/leaver_bl_forfeiture_test.rs
    - services/esop/tests/leaver_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/{esop,hr}/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test leaver

  disallowed_tools:
    - commit without dual-sign (per DEC-2292)
    - mutate prior outcome without board sign (per DEC-2293)

effort_hours: 5
subtasks:
  - "0.3h: 0005_leaver_outcomes.sql"
  - "0.3h: leaver/mod.rs"
  - "0.4h: dual_sign_gate.rs"
  - "0.6h: forfeiture_executor.rs"
  - "0.4h: handlers/leaver_routes.rs"
  - "0.3h: audit/leaver_events.rs"
  - "1.9h: tests — 5 test files"
  - "0.8h: CFO+CEO UI"

risk_if_skipped: "Without GL/BL branch, equity offboarding ad-hoc → legal exposure. Without DEC-2292 dual-sign, single-signer governance gap. Without DEC-2293 immutability, retroactive changes harm member trust."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship GL/BL branch at `services/esop/src/leaver/` triggered by TASK-HR-009 with CFO+CEO dual-sign + forfeiture executor, 5 memory audit kinds.

1. **MUST** validate `leaver_outcome` against closed enum per DEC-2291.

2. **MUST** trigger on TASK-HR-009 termination.executed per DEC-2290.

3. **MUST** require CFO+CEO dual-sign at `dual_sign_gate.rs` per DEC-2292 — same-person rejected.

4. **MUST** execute forfeiture at `forfeiture_executor.rs::execute(outcome)` per DEC-2290:
   - good_leaver_full_vest: vesting halt at term_date; vested = at_term_date
   - good_leaver_pro_rated: vesting continues for X months post-term
   - bad_leaver_unvested_forfeit: vested kept; unvested forfeited
   - bad_leaver_full_forfeit: all shares forfeited (vested + unvested)
   - mutual_negotiated: per agreement; CFO+CEO specify

5. **MUST** define table at migration `0005`:
   ```sql
   CREATE TABLE esop_leaver_outcomes (
     outcome_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     termination_id UUID NOT NULL,  -- TASK-HR-009 ref
     grant_id UUID NOT NULL REFERENCES esop_sp_grants(grant_id),
     outcome TEXT NOT NULL
       CHECK (outcome IN ('good_leaver_full_vest','good_leaver_pro_rated','bad_leaver_unvested_forfeit','bad_leaver_full_forfeit','mutual_negotiated')),
     shares_vested_at_term BIGINT NOT NULL,
     shares_forfeited BIGINT NOT NULL,
     shares_retained BIGINT NOT NULL,
     status TEXT NOT NULL DEFAULT 'drafted'
       CHECK (status IN ('drafted','cfo_signed','ceo_signed','committed','dismissed')),
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     committed_at TIMESTAMPTZ,
     notes TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (termination_id, grant_id)
   );
   ALTER TABLE esop_leaver_outcomes ENABLE ROW LEVEL SECURITY;
   CREATE POLICY outcomes_rls ON esop_leaver_outcomes
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_leaver_outcomes FROM cyberos_app;
   GRANT UPDATE (status, cfo_signed_by, cfo_signed_at, ceo_signed_by, ceo_signed_at, committed_at) ON esop_leaver_outcomes TO cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/esop/leaver-outcomes                   (auto from TASK-HR-009 trigger; CFO/CEO can also draft)
   POST /v1/esop/leaver-outcomes/{id}/cfo-sign
   POST /v1/esop/leaver-outcomes/{id}/ceo-sign
   GET  /v1/esop/leaver-outcomes/{id}              (status)
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2294. PII per TASK-MEMORY-111: share counts SHA256.

8. **MUST** thread trace_id from termination trigger → sign → commit → audit.

9. **MUST NOT** commit without dual-sign per DEC-2292.

10. **MUST NOT** mutate committed per DEC-2293.

11. **MUST** be idempotent per UNIQUE(termination_id, grant_id) — one outcome per (termination, grant) pair.

---

## §2 — Why this design

**Why 5 outcomes (DEC-2291)?** Captures full spectrum — voluntary (GL full), redundancy (GL pro-rated), misconduct (BL forfeit), severe (BL full), negotiated.

**Why dual-sign (DEC-2292)?** Equity decisions = high-stakes; CFO budget + CEO governance.

**Why immutable (DEC-2293)?** Member trust depends on irrevocability; corrections require board oversight.

---

## §3 — API contract

Sample outcome:
```json
{
  "outcome_id": "uuid",
  "termination_id": "uuid",
  "grant_id": "uuid",
  "outcome": "good_leaver_full_vest",
  "shares_vested_at_term": 2500,
  "shares_forfeited": 0,
  "shares_retained": 2500,
  "status": "committed"
}
```

---

## §4 — Acceptance criteria
1. **leaver_outcome enum cardinality 5**. 2. **Triggered by TASK-HR-009**. 3. **CFO+CEO dual-sign**. 4. **Same-person rejected**. 5. **GL full → vesting halt at term**. 6. **BL unvested → vested retained**. 7. **BL full → all forfeited**. 8. **UNIQUE(termination_id, grant_id)**. 9. **5 memory audit kinds emitted**. 10. **PII scrubbed (shares SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except status cols**. 14. **bigint shares**. 15. **shares math: vested+forfeit+retained ≤ grant.total**. 16. **CFO/CEO-only sign**. 17. **Immutable post-commit**. 18. **Grant status → cancelled_unvested or fully_vested per outcome**. 19. **Notes field for mutual_negotiated**. 20. **Auto-draft on termination execute**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required_to_commit() {
    let ctx = TestContext::with_drafted_outcome().await;
    ctx.cfo_sign(ctx.outcome_id).await;
    let o = ctx.fetch_outcome(ctx.outcome_id).await;
    assert_ne!(o.status, "committed");
    ctx.ceo_sign(ctx.outcome_id).await;
    let o2 = ctx.fetch_outcome(ctx.outcome_id).await;
    assert_eq!(o2.status, "committed");
}

#[tokio::test]
async fn gl_full_vest_at_term() {
    let ctx = TestContext::with_member_grant_3y_vested_5000().await;
    ctx.terminate_gl(ctx.member_id).await;
    ctx.cfo_sign_outcome(...).await;
    ctx.ceo_sign_outcome(...).await;
    let o = ctx.fetch_outcome(...).await;
    assert_eq!(o.shares_retained, 5000);
    assert_eq!(o.shares_forfeited, 0);
}

#[tokio::test]
async fn bl_full_forfeit() {
    let ctx = TestContext::with_member_grant_vested_5000_total_10000().await;
    ctx.set_outcome(..., "bad_leaver_full_forfeit").await;
    ctx.both_sign(...).await;
    let o = ctx.fetch_outcome(...).await;
    assert_eq!(o.shares_retained, 0);
    assert_eq!(o.shares_forfeited, 10000);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-009.
**Cross-module:** TASK-ESOP-001 (grant), TASK-ESOP-002 (vested calc), TASK-AUTH-101 (CFO/CEO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| One signer missing | gate | reject commit | wait |
| Same-person | validate | 403 | different signer |
| Math invariant violation | check vested+forfeit+retained | sev-1; reject | bug fix |
| Cross-tenant | RLS | 403 | inherent |
| Duplicate outcome | UNIQUE | 409 | inherent |
| HR-009 termination not executed | check | 412 | wait |
| Mutual_negotiated without notes | validate | warn; accept | provide notes |
| Concurrent sign | UPDATE WHERE | first wins | inherent |
| Grant doesn't exist | FK | 404 | inherent |
| Decimal precision | bigint | inherent | inherent |

## §11 — Implementation notes
- §11.1 Outcome trigger from TASK-HR-009 termination.execute cascade (one outcome per grant per member).
- §11.2 vested_at_term computed from TASK-ESOP-002 accrual at term_date.
- §11.3 memory audit body: outcome_id, member_id, outcome enum, status; shares SHA256.
- §11.4 Grant status updated post-commit: bad_leaver_full_forfeit → cancelled_unvested; otherwise → fully_vested or remains active per residual shares.
- §11.5 Cap negotiated outcomes via CFO+CEO notes field; future could add structured negotiation params.

---

*End of TASK-ESOP-005 spec.*
