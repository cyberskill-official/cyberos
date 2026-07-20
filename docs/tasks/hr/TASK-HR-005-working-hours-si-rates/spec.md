---
id: TASK-HR-005
title: "HR Decree 145/2020 working-hour caps + Decree 152/2020 SI rates — version-pinned policy constants with annual refresh + tenant override"
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
related_tasks: [TASK-HR-001, TASK-TIME-007, TASK-REW-004, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: [TASK-RES-005, TASK-REW-004]

source_pages:
  - website/docs/modules/hr.html#policy-constants
  # Decree 145/2020 (working hours) + Decree 152/2020 (SI rates)
  - https://thuvienphapluat.vn/

source_decisions:
  - DEC-1840 2026-05-17 — Version-pinned policy table: working-hour caps (48h/wk regular, +12h OT/wk, +200h OT/yr — Art. 107), SI rates (BHXH 17.5%, BHYT 4.5%, BHTN 2% employer-side per Decree 152)
  - DEC-1841 2026-05-17 — Closed enum `policy_kind` = {working_hour_cap, si_rate_bhxh, si_rate_bhyt, si_rate_bhtn, pit_bracket, minimum_wage}; cardinality 6
  - DEC-1842 2026-05-17 — Per-version snapshot — immutable; effective_from + effective_to dates; replay determinism critical
  - DEC-1843 2026-05-17 — Tenant override allowed for non-statutory (working hours below cap OK); statutory rates immutable per tenant
  - DEC-1844 2026-05-17 — memory audit kinds: hr.policy_version_added, hr.policy_lookup_executed, hr.tenant_override_set

language: rust 1.81
service: cyberos/services/hr/
new_files:
  - services/hr/migrations/0005_policy_constants.sql
  - services/hr/src/policy/mod.rs
  - services/hr/src/policy/loader.rs
  - services/hr/src/policy/seed_decree_145_152.rs
  - services/hr/src/audit/policy_events.rs
  - services/hr/tests/policy_version_pinning_test.rs
  - services/hr/tests/policy_kind_enum_cardinality_test.rs
  - services/hr/tests/policy_immutability_test.rs
  - services/hr/tests/policy_tenant_override_test.rs
  - services/hr/tests/policy_audit_emission_test.rs

modified_files:
  - services/hr/src/lib.rs

allowed_tools:
  - file_read: services/hr/**
  - file_write: services/hr/{src,tests,migrations}/**
  - bash: cd services/hr && cargo test policy

disallowed_tools:
  - mutate prior version (per DEC-1842)
  - tenant override of statutory rates (per DEC-1843)

effort_hours: 4
subtasks:
  - "0.3h: 0005_policy_constants.sql"
  - "0.3h: policy/mod.rs"
  - "0.4h: loader.rs"
  - "0.5h: seed_decree_145_152.rs (initial seed)"
  - "0.3h: audit/policy_events.rs"
  - "1.8h: tests — 5 test files"
  - "0.4h: docs + annual-refresh runbook"

risk_if_skipped: "Without version-pinned policy, TASK-TIME-007 OT enforcement + TASK-REW-004 deductions drift over time → audit failures. Without DEC-1842 immutability, prior payslip recompute breaks (different rate replay). Without DEC-1843 statutory-vs-override distinction, tenants illegally lower SI rates."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship policy constants at `services/hr/src/policy/` with version-pinned snapshots (Decree 145 + 152), per-version immutability, tenant override for non-statutory, 3 memory audit kinds.

1. **MUST** validate `policy_kind` against closed enum per DEC-1841.

2. **MUST** seed initial Decree 145 + 152 values at `seed_decree_145_152.rs::seed()` — Phase 1 migration.

3. **MUST** lookup at `loader.rs::get(tenant_id, kind, effective_at)`:
- Returns version effective at the date (handles annual refresh).
- Determinism: same params → same result (critical for TASK-REW-004 replay).

4. **MUST** define tables at migration `0005`:
   ```sql
   CREATE TABLE hr_policy_versions (
     version_id UUID PRIMARY KEY,
     kind TEXT NOT NULL
       CHECK (kind IN ('working_hour_cap','si_rate_bhxh','si_rate_bhyt','si_rate_bhtn','pit_bracket','minimum_wage')),
     value_jsonb JSONB NOT NULL,
     effective_from DATE NOT NULL,
     effective_to DATE,
     source_law_reference TEXT NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     created_by UUID NOT NULL
   );
   CREATE INDEX policy_kind_effective_idx ON hr_policy_versions(kind, effective_from DESC, effective_to);
   REVOKE UPDATE, DELETE ON hr_policy_versions FROM cyberos_app;
   -- Append-only immutable; per DEC-1842

   CREATE TABLE hr_tenant_policy_overrides (
     tenant_id UUID NOT NULL,
     kind TEXT NOT NULL
       CHECK (kind IN ('working_hour_cap','minimum_wage')),  -- only non-statutory
     override_value_jsonb JSONB NOT NULL,
     effective_from DATE NOT NULL,
     set_by UUID NOT NULL,
     PRIMARY KEY (tenant_id, kind, effective_from)
   );
   ALTER TABLE hr_tenant_policy_overrides ENABLE ROW LEVEL SECURITY;
   CREATE POLICY tenant_override_rls ON hr_tenant_policy_overrides
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_tenant_policy_overrides FROM cyberos_app;
   ```

5. **MUST** allow tenant override only for non-statutory per DEC-1843 — CHECK constraint enforces.

6. **MUST** expose endpoints:
   ```text
   POST   /v1/hr/policy-versions          (sys-admin only)
   GET    /v1/hr/policy?kind=X&at=DATE
   PUT    /v1/hr/tenant-policy-override   (CHRO only; non-statutory only)
   ```

7. **MUST** emit 3 memory audit kinds per DEC-1844. PII: none (policy values public).

8. **MUST** thread trace_id from lookup → loader → audit.

9. **MUST NOT** mutate prior version per DEC-1842 (REVOKE UPDATE/DELETE).

10. **MUST NOT** allow tenant override of SI/PIT/BHXH per DEC-1843 (CHECK constraint).

---

## §2 — Why this design

**Why version-pinned (DEC-1842)?** TASK-REW-004 must reproduce historical payslips deterministically; mutable rates break replay.

**Why per-kind enum (DEC-1841)?** Closed set of statutory/business policy types; prevents ad-hoc additions.

**Why tenant override gate (DEC-1843)?** Tenants can offer better-than-minimum (e.g. shorter working hours), but cannot lower SI obligations.

**Why annual refresh (DEC-1840)?** VN policy law updates annually (esp. minimum wage); refresh adds new version row, doesn't mutate.

---

## §3 — API contract

Sample policy lookup:
```text
GET /v1/hr/policy?kind=working_hour_cap&at=2026-06-01
```

Response:
```json
{
  "kind": "working_hour_cap",
  "value_jsonb": {"regular_per_week": 48, "ot_per_week_max": 12, "ot_per_year_max": 200},
  "effective_from": "2021-01-01",
  "effective_to": null,
  "source_law_reference": "Decree 145/2020 Art. 107"
}
```

---

## §4 — Acceptance criteria
1. **6-kind enum + cardinality test**. 2. **Initial seed populated**. 3. **Version pinning correct (effective_at lookup)**. 4. **Immutability enforced (REVOKE UPDATE/DELETE)**. 5. **Tenant override gated to non-statutory (CHECK)**. 6. **Statutory override attempt rejected (400)**. 7. **3 memory audit kinds emitted**. 8. **Replay determinism**. 9. **Annual refresh adds new version row**. 10. **Source law reference required**. 11. **RLS on tenant_overrides**. 12. **Sys-admin only for global**. 13. **CHRO only for tenant override**. 14. **Trace_id preserved**. 15. **Lookup performance < 5ms (index)**. 16. **TASK-TIME-007 reads OT caps from this**. 17. **TASK-REW-004 reads SI rates from this**. 18. **JSONB schema validated per kind**. 19. **Effective_to NULL means current**. 20. **Annual seed runbook documented**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn working_hour_cap_seeded() {
    let ctx = TestContext::with_seed().await;
    let v = ctx.policy_get("working_hour_cap", "2026-01-01").await;
    assert_eq!(v.value_jsonb["regular_per_week"], 48);
}

#[tokio::test]
async fn version_pinning_replay() {
    let ctx = TestContext::with_two_versions("minimum_wage", "2024-01-01", "2025-01-01").await;
    let v1 = ctx.policy_get("minimum_wage", "2024-06-01").await;
    let v2 = ctx.policy_get("minimum_wage", "2025-06-01").await;
    assert_ne!(v1.value_jsonb, v2.value_jsonb);
}

#[tokio::test]
async fn immutability_enforced() {
    let ctx = TestContext::with_seed().await;
    let r = ctx.try_update_policy(ctx.version_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn statutory_override_rejected() {
    let ctx = TestContext::with_chro().await;
    let r = ctx.try_set_override("si_rate_bhxh", 10.0).await;
    assert_eq!(r.status_code, 400);
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001. **Downstream:** TASK-TIME-007 (OT caps), TASK-REW-004 (SI rates). **Cross-module:** TASK-AUTH-101 (CHRO + sys-admin roles), TASK-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Kind not in enum | CHECK | reject 400 | use valid |
| Effective_from in past | warn | allow (backdated) | inherent |
| Two versions same effective_from | UNIQUE | reject duplicate | use later date |
| JSONB schema invalid | validator | reject 400 | fix structure |
| Lookup at date with no version | error | 404 | seed gap |
| Statutory override attempt | CHECK | 400 | use non-statutory |
| Cross-tenant override lookup | RLS | 0 rows | inherent |
| Annual seed missed | sev-2 alert | use prior version | runbook fix |
| Policy law deprecated | new version | inherent | maintenance |
| Tenant override conflict | per-tenant index | inherent | inherent |

## §11 — Implementation notes
- §11.1 Seed values: Decree 145 working_hour_cap = {regular_per_week:48, ot_per_week_max:12, ot_per_year_max:200}; Decree 152 SI rates = {bhxh:17.5%, bhyt:4.5%, bhtn:2%}.
- §11.2 Loader chooses effective version: `WHERE effective_from <= $date AND (effective_to IS NULL OR effective_to > $date)`.
- §11.3 Tenant override merged at lookup: if tenant override exists for kind+effective, return override; else global.
- §11.4 memory audit body: kind, version_id, lookup date; no PII.
- §11.5 Annual refresh runbook: review VN gov gazette in December; add new version row with effective_from=Jan 1 next year.

---

*End of TASK-HR-005 spec.*
