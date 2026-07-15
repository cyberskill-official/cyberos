---
id: TASK-CRM-003
title: "CRM VN account types + MST — legal entity classification (Sole/LLC/JSC/FDI) + tax ID field with format validation"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: CRM
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CRM-001, TASK-CRM-008, TASK-INV-007, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CRM-001]
blocks: [TASK-CRM-008]

source_pages:
  - website/docs/modules/crm.html#vn-account-type
  - https://thuvienphapluat.vn/  # VN Enterprise Law 59/2020

source_decisions:
  - DEC-1630 2026-05-17 — VN legal entity types per Enterprise Law 59/2020: Sole proprietorship (Doanh nghiệp tư nhân), LLC (Công ty TNHH), JSC (Công ty cổ phần), FDI (Doanh nghiệp có vốn đầu tư nước ngoài)
  - DEC-1631 2026-05-17 — Closed enum `vn_account_type` = {sole, llc_1, llc_2plus, jsc, fdi, partnership}; cardinality 6
  - DEC-1632 2026-05-17 — MST (Mã số thuế) format: 10 or 13 digits (10 = main entity, 13 = branch with 3-digit suffix)
  - DEC-1633 2026-05-17 — Field is OPTIONAL for non-VN accounts; REQUIRED + validated for accounts with residency='vn-1'
  - DEC-1634 2026-05-17 — memory audit kinds: crm.vn_account_type_set, crm.mst_validated, crm.mst_validation_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0003_vn_account_fields.sql
    - services/crm/src/vn/account_type.rs
    - services/crm/src/vn/mst_format.rs
    - services/crm/src/audit/vn_account_events.rs
    - services/crm/tests/vn_account_type_enum_cardinality_test.rs
    - services/crm/tests/vn_mst_format_10_digit_test.rs
    - services/crm/tests/vn_mst_format_13_digit_test.rs
    - services/crm/tests/vn_mst_required_for_vn_residency_test.rs
    - services/crm/tests/vn_account_audit_emission_test.rs

  modified_files:
    - services/crm/src/accounts.rs

  allowed_tools:
    - file_read: services/crm/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test vn

  disallowed_tools:
    - MST not validated on VN account (per DEC-1633)
    - 11/12/14-digit MST accepted (per DEC-1632)

effort_hours: 4
subtasks:
  - "0.3h: 0003_vn_account_fields.sql"
  - "0.3h: vn/account_type.rs"
  - "0.4h: vn/mst_format.rs"
  - "0.3h: audit/vn_account_events.rs"
  - "1.6h: tests — 5 test files"
  - "0.5h: CRO UI for VN account type picker + MST validation"
  - "0.6h: docs"

risk_if_skipped: "Without VN account type, invoicing can't classify entity → wrong hóa đơn template per Decree 123. Without DEC-1632 MST format check, invalid MST passes through → hóa đơn rejected by GDT. Without DEC-1633 VN-residency gate, mandatory field becomes annoying for SG/EU accounts."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** extend Account schema with VN-specific fields at `services/crm/src/vn/` — legal entity type + MST + validation gated on residency, 3 memory audit kinds.

1. **MUST** define table extension at migration `0003`:
   ```sql
   ALTER TABLE crm_accounts ADD COLUMN vn_account_type TEXT
     CHECK (vn_account_type IS NULL OR vn_account_type IN
       ('sole','llc_1','llc_2plus','jsc','fdi','partnership'));
   ALTER TABLE crm_accounts ADD COLUMN mst TEXT;
   ALTER TABLE crm_accounts ADD COLUMN mst_validated_at TIMESTAMPTZ;
   ALTER TABLE crm_accounts ADD CONSTRAINT mst_format
     CHECK (mst IS NULL OR mst ~ '^[0-9]{10}(-[0-9]{3})?$');
   CREATE INDEX accounts_vn_mst_idx ON crm_accounts(tenant_id, mst) WHERE mst IS NOT NULL;
   GRANT UPDATE (vn_account_type, mst, mst_validated_at) ON crm_accounts TO cyberos_app;
   ```

2. **MUST** validate `vn_account_type` against closed enum per DEC-1631.

3. **MUST** validate MST format at `vn/mst_format.rs::validate(mst)` per DEC-1632:
   - 10 digits only OR 10 digits + `-` + 3 digits.
   - Reject letters, spaces, other separators.

4. **MUST** require both fields when `account.residency='vn-1'` per DEC-1633 — at INSERT/UPDATE, if residency=vn-1 and either NULL → reject 400.

5. **MUST** allow both fields NULL for non-VN accounts.

6. **MUST** emit 3 memory audit kinds per DEC-1634. Audit body: account_id, vn_account_type (enum); MST SHA-256 hashed per TASK-MEMORY-111 (treat as PII — could be confidential).

7. **MUST** thread trace_id from account create/update → validation → audit.

8. **MUST NOT** accept MST formats outside DEC-1632 — CHECK constraint enforces.

9. **MUST NOT** require fields on non-VN accounts per DEC-1633.

---

## §2 — Why this design

**Why 6 entity types (DEC-1631)?** Enterprise Law 59/2020 enumerates these as the legal forms; LLC is split into 1-owner vs 2+ because they have distinct registration/tax treatment.

**Why MST format (DEC-1632)?** GDT rejects malformed MST in hóa đơn; pre-validate at CRM level saves a downstream failure.

**Why residency-gated (DEC-1633)?** Non-VN accounts don't have MST; making it mandatory annoys global users.

---

## §3 — API contract

Account fields (extension):
```json
{
  "account_id": "uuid",
  "name": "Acme JSC",
  "residency": "vn-1",
  "vn_account_type": "jsc",
  "mst": "0312345678"
}
```

Branch MST (13 digits):
```json
{ "mst": "0312345678-001" }
```

---

## §4 — Acceptance criteria
1. **6 account types enum + cardinality test**. 2. **MST 10-digit accepted**. 3. **MST 13-digit (with dash) accepted**. 4. **MST 9/11/12/14 rejected (400 + CHECK)**. 5. **MST with letters rejected**. 6. **MST optional for non-VN**. 7. **MST required for vn-1 residency**. 8. **vn_account_type optional for non-VN**. 9. **vn_account_type required for vn-1**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (MST SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Index on mst for lookup**. 15. **mst_validated_at populated on successful validation**. 16. **Append-only via REVOKE UPDATE except 3 cols**. 17. **CRO UI picker shows 6 types**. 18. **TASK-CRM-008 future validation skill leverages this format check**. 19. **TASK-INV-007 hóa đơn emit reads mst from this column**. 20. **Multi-line FDI/JSC company name OK in name field (not affected)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn mst_10_digit_accepted() {
    let ctx = TestContext::vn_residency_account().await;
    let r = ctx.update_account_mst(ctx.account_id, "0312345678").await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn mst_13_digit_branch_accepted() {
    let ctx = TestContext::vn_residency_account().await;
    let r = ctx.update_account_mst(ctx.account_id, "0312345678-001").await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn mst_required_for_vn_residency() {
    let ctx = TestContext::new_tenant().await;
    let r = ctx.create_account_no_mst(ctx.tenant_id, "vn-1").await;
    assert!(r.is_err());
}

#[tokio::test]
async fn mst_not_required_for_sg() {
    let ctx = TestContext::new_tenant().await;
    let r = ctx.create_account_no_mst(ctx.tenant_id, "sg-1").await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn invalid_mst_rejected() {
    for bad in ["0312345", "12345678901", "031234567A", "0312345678-12", "0312345678 "] {
        let ctx = TestContext::vn_residency_account().await;
        let r = ctx.update_account_mst(ctx.account_id, bad).await;
        assert!(r.is_err(), "bad mst accepted: {bad}");
    }
}

// 5.6..5.9 — enum cardinality, audit emission
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-001.
**Downstream:** TASK-CRM-008 (validation skill), TASK-INV-007 (reads MST for hóa đơn).
**Cross-module:** TASK-MEMORY-111 (PII scrub).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| MST format invalid | CHECK constraint | 400 | fix input |
| Residency=vn-1 without MST | trigger/handler | 400 | provide MST |
| Residency change vn-1 → sg-1 | leave MST as-is | inherent | inherent |
| MST duplicate across tenants | per-tenant index OK (cross-tenant allowed) | inherent | inherent |
| TASK-CRM-008 skill validates external | future task | optional confirm via GDT | inherent |
| Account legacy missing fields | migration backfill | NULL preserved | manual fill |
| MST with whitespace | reject | 400 | trim client-side |

## §11 — Implementation notes
- §11.1 MST regex: `^[0-9]{10}(-[0-9]{3})?$` enforced in CHECK + Rust validator.
- §11.2 PII: MST is government identifier per TASK-MEMORY-111; SHA256 in audit chain.
- §11.3 TASK-CRM-008 future skill will call GDT MST verification API for external confirm.
- §11.4 Account type picker shows VN names: "Doanh nghiệp tư nhân" / "TNHH 1 thành viên" / etc.
- §11.5 Migration backfill: NULL allowed for existing rows; CRO updates over time.

---

*End of TASK-CRM-003 spec.*
