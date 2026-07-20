---
id: TASK-HR-002
title: "HR 5 contract types — indefinite + fixed_term + probation + part_time + contractor with per-type leave + benefit rules"
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
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-HR-004, TASK-HR-005, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: []

source_pages:
  - website/docs/modules/hr.html#contract-types
  # VN Labour Code 45/2019 Art. 20
  - https://thuvienphapluat.vn/

source_decisions:
  - DEC-1810 2026-05-17 — 5 contract types per VN Labour Code 45/2019 Art. 20 + business need: indefinite, fixed_term, probation, part_time, contractor
  - DEC-1811 2026-05-17 — Closed enum `contract_type` = {indefinite, fixed_term, probation, part_time, contractor}; cardinality 5
  - DEC-1812 2026-05-17 — Per-type rules: probation ≤ 60d for skilled / 30d unskilled (Art. 25); fixed_term ≤ 36 months (Art. 20); only 1 fixed_term renewal allowed before becoming indefinite (Art. 20.3)
  - DEC-1813 2026-05-17 — Per-type leave entitlement override: contractor=0 paid leave; part_time pro-rated; others = full
  - DEC-1814 2026-05-17 — Per-type SI participation: indefinite/fixed_term/part_time/probation = required; contractor = exempt (per Decree 152)
  - DEC-1815 2026-05-17 — memory audit kinds: hr.contract_type_set, hr.contract_renewal_attempted, hr.contract_violation_detected

language: rust 1.81
service: cyberos/services/hr/
new_files:
  - services/hr/migrations/0002_contract_types.sql
  - services/hr/src/contract/mod.rs
  - services/hr/src/contract/rules_enforcer.rs
  - services/hr/src/audit/contract_events.rs
  - services/hr/tests/contract_type_enum_cardinality_test.rs
  - services/hr/tests/contract_probation_max_60d_test.rs
  - services/hr/tests/contract_fixed_term_renewal_limit_test.rs
  - services/hr/tests/contract_leave_per_type_test.rs
  - services/hr/tests/contract_si_per_type_test.rs
  - services/hr/tests/contract_audit_emission_test.rs

modified_files:
  - services/hr/src/members.rs

allowed_tools:
  - file_read: services/hr/**
  - file_write: services/hr/{src,tests,migrations}/**
  - bash: cd services/hr && cargo test contract

disallowed_tools:
  - allow probation > 60d (per DEC-1812)
  - allow >1 fixed_term renewal (per DEC-1812)

effort_hours: 4
subtasks:
  - "0.3h: 0002_contract_types.sql"
  - "0.4h: contract/mod.rs"
  - "0.6h: rules_enforcer.rs"
  - "0.3h: audit/contract_events.rs"
  - "0.3h: members.rs hook"
  - "1.7h: tests — 6 test files"
  - "0.4h: docs"

risk_if_skipped: "Without contract type enforcement, illegal probation lengths + fixed-term abuse → VN Labour inspector penalties. Without DEC-1813 per-type leave, contractors accrue PTO (cost). Without DEC-1814 SI rules, contractor SI contribution leakage."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** extend member schema with contract types at `services/hr/src/contract/` enforcing VN Labour Code constraints + per-type leave/SI rules, 3 memory audit kinds.

1. **MUST** validate `contract_type` against closed enum per DEC-1811.

2. **MUST** enforce probation duration per DEC-1812 — if contract_type='probation' and end_date - start_date > 60d (skilled) or 30d (unskilled), reject.

3. **MUST** enforce fixed-term renewal limit per DEC-1812 — `SELECT COUNT(*) FROM contract_history WHERE member_id=... AND type='fixed_term'`; if ≥2 attempting → reject; must convert to indefinite.

4. **MUST** override leave entitlement per DEC-1813:
- contractor → 0 paid leave (overrides TASK-HR-004 defaults)
- part_time → pro-rated by hours_per_week / 48

5. **MUST** override SI participation per DEC-1814:
- contractor → SI participation=false
- others → SI participation=true

6. **MUST** define table extension at migration `0002`:
   ```sql
   ALTER TABLE hr_members ADD COLUMN contract_type TEXT
     CHECK (contract_type IS NULL OR contract_type IN ('indefinite','fixed_term','probation','part_time','contractor'));
   ALTER TABLE hr_members ADD COLUMN contract_start_date DATE;
   ALTER TABLE hr_members ADD COLUMN contract_end_date DATE;
   ALTER TABLE hr_members ADD COLUMN hours_per_week INT;
   ALTER TABLE hr_members ADD COLUMN is_skilled BOOLEAN NOT NULL DEFAULT true;
   GRANT UPDATE (contract_type, contract_start_date, contract_end_date, hours_per_week, is_skilled) ON hr_members TO cyberos_app;

   CREATE TABLE hr_contract_history (
     history_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     contract_type TEXT NOT NULL CHECK (contract_type IN ('indefinite','fixed_term','probation','part_time','contractor')),
     start_date DATE NOT NULL,
     end_date DATE,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX contract_hist_member_idx ON hr_contract_history(tenant_id, member_id, start_date DESC);
   ALTER TABLE hr_contract_history ENABLE ROW LEVEL SECURITY;
   CREATE POLICY contract_hist_rls ON hr_contract_history
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_contract_history FROM cyberos_app;
   ```

7. **MUST** emit 3 memory audit kinds per DEC-1815. PII per TASK-MEMORY-111: member_id (uuid) ok; type enum ok.

8. **MUST** thread trace_id from set/renewal → enforcer → audit.

9. **MUST NOT** allow probation > legal cap per DEC-1812.

10. **MUST NOT** allow >1 fixed-term renewal per DEC-1812.

---

## §2 — Why this design

**Why 5 types (DEC-1810)?** Covers VN Labour Code requirements + business need for contractor (non-employee). 5 types map to all current real-world scenarios at CyberSkill.

**Why renewal limit (DEC-1812)?** VN Labour Code Art. 20.3: max 1 fixed-term contract; second must be indefinite. Auto-enforcement prevents inspector findings.

**Why per-type leave (DEC-1813)?** Contractors don't get paid leave; part-timers pro-rated. Pre-computed at contract assignment avoids leave-calc errors.

**Why per-type SI (DEC-1814)?** Decree 152/2020 exempts contractors from SI; auto-enforcing avoids over-deduction.

---

## §3 — API contract

```text
PUT    /v1/hr/members/{id}/contract       body: {type, start_date, end_date?, hours_per_week?, is_skilled?}
GET    /v1/hr/members/{id}/contract-history
```

Sample contract assignment:
```json
{
  "type": "probation",
  "start_date": "2026-06-01",
  "end_date": "2026-07-30",
  "is_skilled": true
}
```

---

## §4 — Acceptance criteria
1. **5-type enum + cardinality test**. 2. **Probation skilled ≤60d enforced**. 3. **Probation unskilled ≤30d enforced**. 4. **Fixed_term renewal limit 1 enforced**. 5. **2nd fixed_term → reject; require indefinite conversion**. 6. **Contractor → 0 leave entitlement**. 7. **Part_time → pro-rated leave by hours_per_week/48**. 8. **Contractor → SI=false**. 9. **Other types → SI=true**. 10. **3 memory audit kinds emitted**. 11. **PII: member_id (uuid) ok**. 12. **RLS denies cross-tenant**. 13. **CHRO role only for write**. 14. **Trace_id preserved**. 15. **History append-only**. 16. **fixed_term ≤36 months enforced**. 17. **Type change creates new history row (no in-place update)**. 18. **End_date NULL allowed for indefinite/contractor**. 19. **Hours_per_week required for part_time (NULL rejected)**. 20. **Violation detection sev-2 audit**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn probation_max_60d_enforced() {
    let ctx = TestContext::with_member().await;
    let r = ctx.set_contract(ctx.member_id, "probation", "2026-06-01", Some("2026-09-01"), Some(true)).await;
    assert!(r.is_err());  // 92d > 60d cap
}

#[tokio::test]
async fn fixed_term_renewal_limit() {
    let ctx = TestContext::with_member_one_fixed_term().await;
    let r = ctx.try_renew_fixed_term(ctx.member_id).await;
    assert!(r.is_err());  // must convert to indefinite
}

#[tokio::test]
async fn contractor_zero_leave() {
    let ctx = TestContext::with_member().await;
    ctx.set_contract(ctx.member_id, "contractor", "2026-06-01", None, None).await;
    let m = ctx.fetch_member(ctx.member_id).await;
    assert_eq!(m.leave_entitlement_days, 0);
}

#[tokio::test]
async fn part_time_prorated_leave() {
    let ctx = TestContext::with_member().await;
    ctx.set_contract(ctx.member_id, "part_time", "2026-06-01", Some("2027-06-01"), Some(24)).await;
    let m = ctx.fetch_member(ctx.member_id).await;
    assert_eq!(m.leave_entitlement_days, 6);  // 12 × (24/48)
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001. **Cross-module:** TASK-HR-004 (leave types), TASK-HR-005 (working hours), TASK-REW-004 (SI), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Type invalid | CHECK constraint | 400 | use valid |
| Probation length excess | enforcer | 400 with rule cite | shorten |
| Fixed_term 2nd renewal | history count check | 400 | convert to indefinite |
| Part_time hours_per_week NULL | validate | 400 | provide hours |
| Contractor with paid leave attempt | override | leave_days=0 + sev-2 audit | inherent |
| Contract end before start | validate | 400 | fix dates |
| Concurrent contract change | optimistic lock | 409 | retry |
| Cross-tenant member | RLS | 404 | inherent |
| History pruning | append-only | grows unbounded | partition by year |
| Non-CHRO write | role check | 403 | request CHRO |

## §11 — Implementation notes
- §11.1 Enforcer is pure function: `(type, start, end, hours, is_skilled, history) → Result<(), Violation>`.
- §11.2 Leave entitlement computed at contract set; cached on member row; TASK-HR-004 reads it.
- §11.3 SI participation flag drives TASK-REW-004 deduction logic.
- §11.4 History row written on every contract type/dates change; full audit trail.
- §11.5 memory audit body: member_id, contract_type, dates; reason on violations.

---

*End of TASK-HR-002 spec.*
