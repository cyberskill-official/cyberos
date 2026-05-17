---
id: FR-RES-005
title: "RES VN Labour Code Art. 107 OT cap hard-block — propose-time validation gate preventing weekly + annual OT overflow"
module: RES
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 8
slice: 8
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-HR-005, FR-RES-002, FR-TIME-007, FR-BRAIN-111]
depends_on: [FR-HR-005]
blocks: []

source_pages:
  - website/docs/modules/res.html#ot-cap
  - https://thuvienphapluat.vn/  # Decree 145/2020 Art. 107

source_decisions:
  - DEC-2060 2026-05-17 — Hard-block at FR-RES-002 propose-time: weekly OT cap (12h regular / 30h with consent) + annual OT cap (200h / 300h industry-specific)
  - DEC-2061 2026-05-17 — Closed enum `ot_cap_decision` = {allowed, blocked_weekly, blocked_annual, blocked_consent_missing}; cardinality 4
  - DEC-2062 2026-05-17 — Consent record required for >12h/wk per VN Labour Code Art. 107; CHRO confirms member's prior agreement
  - DEC-2063 2026-05-17 — Cap values read from FR-HR-005 policy version (replay determinism)
  - DEC-2064 2026-05-17 — BRAIN audit kinds: res.ot_check_passed, res.ot_blocked_weekly, res.ot_blocked_annual, res.ot_blocked_consent

build_envelope:
  language: rust 1.81
  service: cyberos/services/res/
  new_files:
    - services/res/migrations/0005_ot_consent.sql
    - services/res/src/ot_cap/mod.rs
    - services/res/src/ot_cap/checker.rs
    - services/res/src/handlers/ot_cap_routes.rs
    - services/res/src/audit/ot_cap_events.rs
    - services/res/tests/ot_cap_weekly_block_test.rs
    - services/res/tests/ot_cap_annual_block_test.rs
    - services/res/tests/ot_cap_consent_required_test.rs
    - services/res/tests/ot_cap_decision_enum_cardinality_test.rs
    - services/res/tests/ot_cap_audit_emission_test.rs

  modified_files:
    - services/res/src/allocation/validator.rs

  allowed_tools:
    - file_read: services/{res,hr,time}/**
    - file_write: services/res/{src,tests,migrations}/**
    - bash: cd services/res && cargo test ot_cap

  disallowed_tools:
    - bypass cap (per DEC-2060 — hard-block)
    - allow >12h/wk without consent (per DEC-2062)

effort_hours: 4
sub_tasks:
  - "0.3h: 0005_ot_consent.sql"
  - "0.3h: ot_cap/mod.rs"
  - "0.5h: checker.rs"
  - "0.3h: handlers/ot_cap_routes.rs"
  - "0.3h: audit/ot_cap_events.rs"
  - "0.3h: validator.rs hook"
  - "1.8h: tests — 5 test files"
  - "0.2h: docs"

risk_if_skipped: "Without hard-block, allocations breach Labour Code → Labour inspector fines + reputational damage. Without DEC-2062 consent gate, >12h OT without paperwork = illegal even if member agrees verbally."
---

## §1 — Description (BCP-14 normative)

The RES service **MUST** ship OT cap hard-block at `services/res/src/ot_cap/` enforcing Art. 107 weekly + annual + consent, 4 BRAIN audit kinds.

1. **MUST** validate `ot_cap_decision` against closed enum per DEC-2061.

2. **MUST** check at `checker.rs::check(member, week, proposed_hours)` per DEC-2060:
   - Compute proposed weekly OT = max(0, proposed_total - regular_weekly_cap)
   - Compute YTD annual OT from FR-TIME-007 history
   - Read caps from FR-HR-005 policy lookup
   - Decision per DEC-2061:
     - proposed_weekly_ot > regular_cap (12h) AND no consent → blocked_consent_missing
     - proposed_weekly_ot > consent_cap (30h) → blocked_weekly
     - YTD + proposed > annual_cap (200h) → blocked_annual
     - Else allowed

3. **MUST** define consent table at migration `0005`:
   ```sql
   CREATE TABLE res_ot_consent (
     consent_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     valid_from DATE NOT NULL,
     valid_to DATE NOT NULL,
     consented_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     consented_by_member UUID NOT NULL,
     recorded_by_chro UUID NOT NULL,
     consent_doc_id UUID,  -- FR-DOC-001 reference
     UNIQUE (tenant_id, member_id, valid_from, valid_to)
   );
   ALTER TABLE res_ot_consent ENABLE ROW LEVEL SECURITY;
   CREATE POLICY ot_consent_rls ON res_ot_consent
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_ot_consent FROM cyberos_app;
   ```

4. **MUST** integrate with FR-RES-002 validator — checker called pre-commit.

5. **MUST** expose endpoints:
   ```text
   POST /v1/res/ot-consent          (CHRO records consent)
   GET  /v1/res/members/{id}/ot-status   (current YTD + caps)
   ```

6. **MUST** emit 4 BRAIN audit kinds per DEC-2064. PII per FR-BRAIN-111: hours SHA-256 hashed.

7. **MUST** thread trace_id from FR-RES-002 propose → checker → audit.

8. **MUST NOT** bypass cap per DEC-2060.

9. **MUST NOT** allow >regular_cap without active consent per DEC-2062.

---

## §2 — Why this design

**Why hard-block (DEC-2060)?** Soft warnings get ignored; Labour Code is criminal liability for systemic violations.

**Why consent gate (DEC-2062)?** Art. 107 explicitly requires written consent for OT beyond 12h/wk; without it, allocation is per-se illegal.

**Why policy lookup (DEC-2063)?** Caps may change per VN gov updates; FR-HR-005 versions handle this.

---

## §3 — API contract

Sample check response:
```json
{
  "decision": "blocked_weekly",
  "weekly_ot_proposed": 35,
  "weekly_ot_cap_with_consent": 30,
  "consent_active": true,
  "rejection_message": "Weekly OT 35h exceeds 30h cap per Decree 145 Art. 107"
}
```

Sample consent record:
```json
POST /v1/res/ot-consent
{
  "member_id": "uuid",
  "valid_from": "2026-06-01",
  "valid_to": "2026-12-31",
  "consent_doc_id": "uuid-signed-consent-pdf"
}
```

---

## §4 — Acceptance criteria
1. **ot_cap_decision enum cardinality 4**. 2. **Weekly OT cap enforced (12h no consent)**. 3. **Consent cap enforced (30h with consent)**. 4. **Annual OT cap enforced (200h)**. 5. **Industry-specific 300h supported via policy**. 6. **Consent required for >12h/wk**. 7. **Caps read from FR-HR-005**. 8. **YTD computed from FR-TIME-007**. 9. **4 BRAIN audit kinds emitted**. 10. **PII scrubbed (hours SHA256)**. 11. **RLS denies cross-tenant**. 12. **CHRO-only consent record**. 13. **Trace_id preserved**. 14. **Consent immutable (append-only)**. 15. **Expired consent treated as missing**. 16. **Multiple consent rows per member allowed**. 17. **Cap exceeded → block with explanation**. 18. **rust_decimal precision**. 19. **Integration with FR-RES-002 validator tested**. 20. **YTD aggregation handles partial year**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn weekly_cap_no_consent_blocks_at_13h() {
    let ctx = TestContext::member_no_consent().await;
    let r = ot_cap::check(ctx.member_id, this_week(), dec!(53)).await;  // 40 reg + 13 OT
    assert_eq!(r.decision, "blocked_consent_missing");
}

#[tokio::test]
async fn weekly_cap_with_consent_blocks_at_31h() {
    let ctx = TestContext::member_with_consent().await;
    let r = ot_cap::check(ctx.member_id, this_week(), dec!(71)).await;  // 40 + 31 OT
    assert_eq!(r.decision, "blocked_weekly");
}

#[tokio::test]
async fn annual_cap_blocks_at_201h() {
    let ctx = TestContext::member_with_ytd_ot(190).await;
    let r = ot_cap::check(ctx.member_id, this_week(), dec!(52)).await;  // would push to 202
    assert_eq!(r.decision, "blocked_annual");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-005.
**Cross-module:** FR-RES-002 (validator integration), FR-TIME-007 (YTD), FR-DOC-001 (consent doc), FR-AUTH-101 (CHRO), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Policy lookup fail | catch | sev-1; reject conservatively | retry |
| YTD compute fail | catch | sev-1; reject (safer) | retry |
| Consent expired mid-week | exclude expired | block | renew consent |
| Industry-specific 300h applies | policy override | inherent | tenant config |
| Mid-week policy change | use version at week start | inherent | inherent |
| Cross-tenant consent | RLS | not found → blocked | inherent |
| Decimal precision | rust_decimal | inherent | inherent |
| Partial-year hire | YTD from hire date | inherent | inherent |
| Consent doc missing | warn | still valid if recorded | doc upload |
| Duplicate consent | UNIQUE | second 409 | use existing |

## §11 — Implementation notes
- §11.1 Checker pure function: `(member, week, proposed, ytd, policy) → Decision`.
- §11.2 Cap values: regular_weekly_cap, consent_cap, annual_cap fetched via FR-HR-005 policy.
- §11.3 Consent table immutable; new row per period rather than UPDATE.
- §11.4 BRAIN audit body: member_id, week, decision; hours SHA256.
- §11.5 Integration test simulates FR-RES-002 propose flow end-to-end.

---

*End of FR-RES-005 spec.*
