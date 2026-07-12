---
id: FR-REW-001
title: "REW 3P income schema — P1 Base + P2 Allowance + P3 Performance with separate encrypted comp keyspace isolated from HR"
module: REW
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-HR-001, FR-AUTH-105, FR-REW-002, FR-REW-003, FR-MEMORY-111]
depends_on: [FR-HR-001, FR-AUTH-101]
blocks: [FR-REW-002, FR-REW-003, FR-REW-005, FR-REW-007, FR-REW-010]

source_pages:
  - website/docs/modules/rew.html#3p-schema

source_decisions:
  - DEC-2150 2026-05-17 — 3P income model: P1 Base (fixed salary) + P2 Allowance (housing/transport/meal) + P3 Performance (bonus/commission); each isolated component, separate FK to comp record
  - DEC-2151 2026-05-17 — Closed enum `income_kind` = {p1_base, p2_allowance_housing, p2_allowance_transport, p2_allowance_meal, p2_allowance_other, p3_bonus_quarterly, p3_commission, p3_spot_award}; cardinality 8
  - DEC-2152 2026-05-17 — Comp data encrypted in separate KMS keyspace (rew-{tenant_id}) — distinct from HR keyspace; ROOT-CFO only can decrypt
  - DEC-2153 2026-05-17 — Per-Member comp records IMMUTABLE; changes via new row + valid_from/valid_to
  - DEC-2154 2026-05-17 — memory audit kinds: rew.comp_set, rew.comp_corrected, rew.comp_decrypted, rew.comp_encryption_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0001_comp_schema.sql
    - services/rew/src/comp/mod.rs
    - services/rew/src/comp/encryption.rs
    - services/rew/src/comp/access_gate.rs
    - services/rew/src/handlers/comp_routes.rs
    - services/rew/src/audit/comp_events.rs
    - services/rew/tests/income_kind_enum_cardinality_test.rs
    - services/rew/tests/comp_root_cfo_only_decrypt_test.rs
    - services/rew/tests/comp_separate_keyspace_test.rs
    - services/rew/tests/comp_immutable_test.rs
    - services/rew/tests/comp_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,hr,auth}/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test comp

  disallowed_tools:
    - decrypt comp via non-CFO role (per DEC-2152)
    - mutate prior comp row (per DEC-2153)

effort_hours: 6
sub_tasks:
  - "0.4h: 0001_comp_schema.sql"
  - "0.4h: comp/mod.rs"
  - "0.5h: encryption.rs"
  - "0.5h: access_gate.rs"
  - "0.4h: handlers/comp_routes.rs"
  - "0.3h: audit/comp_events.rs"
  - "2.5h: tests — 5 test files"
  - "1.0h: docs + CFO UI for comp set + decrypt"

risk_if_skipped: "Without 3P schema, comp scattered → governance impossible. Without DEC-2152 separate keyspace, HR-KMS compromise leaks comp. Without DEC-2153 immutability, retroactive changes break payroll replay."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship 3P income schema at `services/rew/src/comp/` with 8-kind enum + separate KMS keyspace + CFO-only decrypt + immutable rows, 4 memory audit kinds.

1. **MUST** validate `income_kind` against closed enum per DEC-2151.

2. **MUST** define tables at migration `0001`:
   ```sql
   CREATE TABLE rew_comp_records (
     comp_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     income_kind TEXT NOT NULL
       CHECK (income_kind IN ('p1_base','p2_allowance_housing','p2_allowance_transport','p2_allowance_meal','p2_allowance_other','p3_bonus_quarterly','p3_commission','p3_spot_award')),
     encrypted_amount_vnd BYTEA NOT NULL,  -- encrypted via REW keyspace KMS
     encryption_kms_key_arn TEXT NOT NULL,
     currency CHAR(3) NOT NULL DEFAULT 'VND',
     valid_from DATE NOT NULL,
     valid_to DATE,
     set_by UUID NOT NULL,
     correction_of UUID REFERENCES rew_comp_records(comp_id),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX comp_member_idx ON rew_comp_records(tenant_id, member_id, income_kind, valid_from DESC);
   ALTER TABLE rew_comp_records ENABLE ROW LEVEL SECURITY;
   CREATE POLICY comp_rls ON rew_comp_records
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_comp_records FROM cyberos_app;

   CREATE TABLE rew_comp_access_log (
     log_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     accessor_id UUID NOT NULL,
     comp_id UUID,
     accessor_role TEXT NOT NULL,
     access_kind TEXT NOT NULL,
     succeeded BOOLEAN NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE rew_comp_access_log ENABLE ROW LEVEL SECURITY;
   CREATE POLICY access_log_rls ON rew_comp_access_log
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_comp_access_log FROM cyberos_app;
   ```

3. **MUST** use separate KMS keyspace per DEC-2152 — `rew-{tenant_id}` alias, distinct from `hr-{tenant_id}` and `hr-cccd-{tenant_id}`.

4. **MUST** gate decrypt to ROOT-CFO only via `access_gate.rs::check(user)` per DEC-2152 — other roles → 403 + sev-1 audit.

5. **MUST** correct via new row per DEC-2153 — `correction_of` points to prior; prior's valid_to set.

6. **MUST** expose endpoints:
   ```text
   POST /v1/rew/comp                       (CFO sets/corrects)
   GET  /v1/rew/comp/{id}/decrypt          (ROOT-CFO only)
   GET  /v1/rew/members/{id}/comp-history  (CFO sees encrypted refs)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2154. PII per FR-MEMORY-111: encrypted_amount never in memory chain; income_kind enum ok; member_id (uuid) ok.

8. **MUST** thread trace_id from set / decrypt → audit.

9. **MUST NOT** decrypt for non-CFO per DEC-2152.

10. **MUST NOT** mutate prior comp row per DEC-2153.

11. **MUST NOT** share KMS keyspace with HR per DEC-2152.

---

## §2 — Why this design

**Why 8-kind enum (DEC-2151)?** Captures real 3P composition; bounded prevents add-hoc additions.

**Why separate keyspace (DEC-2152)?** Defense in depth — HR breach doesn't expose comp; comp breach doesn't expose HR PII.

**Why CFO-only decrypt (DEC-2152)?** Principle of least privilege — even CHRO doesn't need decrypt access.

**Why immutable (DEC-2153)?** Payroll replay (FR-REW-002) requires deterministic history; retroactive changes break audit.

---

## §3 — API contract

Sample comp set:
```json
POST /v1/rew/comp
{
  "member_id": "uuid",
  "income_kind": "p1_base",
  "amount_vnd": 30000000,
  "valid_from": "2026-06-01"
}
```

Sample decrypt response (CFO-only):
```json
{
  "comp_id": "uuid",
  "income_kind": "p1_base",
  "amount_vnd": 30000000,
  "valid_from": "2026-06-01"
}
```

---

## §4 — Acceptance criteria
1. **income_kind enum cardinality 8**. 2. **CFO-only decrypt**. 3. **Non-CFO 403 + sev-1 audit**. 4. **Separate KMS keyspace (rew-{tenant})**. 5. **Immutable rows**. 6. **Correction via correction_of**. 7. **4 memory audit kinds emitted**. 8. **encrypted_amount never in memory chain**. 9. **RLS denies cross-tenant**. 10. **Trace_id preserved**. 11. **Access log append-only**. 12. **Member-week comp query indexable**. 13. **valid_from + valid_to range**. 14. **Currency CHAR(3) default VND**. 15. **set_by audit-traceable**. 16. **Tenant comp KMS key created at provisioning**. 17. **Cross-tenant KMS rejection**. 18. **CFO email notification on decrypt (sev-1)**. 19. **Append-only via REVOKE UPDATE/DELETE**. 20. **3P composition: P1 + P2 + P3 = total comp**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn non_cfo_decrypt_denied() {
    let ctx = TestContext::with_encrypted_comp().await;
    let r = ctx.try_decrypt_as(ctx.am_user, ctx.comp_id).await;
    assert_eq!(r.status_code, 403);
    let logs = ctx.fetch_access_log(ctx.comp_id).await;
    assert!(logs.iter().any(|l| !l.succeeded));
}

#[tokio::test]
async fn separate_keyspace() {
    let ctx = TestContext::with_comp_and_hr_records().await;
    let comp_arn = ctx.fetch_comp_kms_arn(ctx.comp_id).await;
    let hr_arn = ctx.fetch_hr_kms_arn(ctx.member_id).await;
    assert_ne!(comp_arn, hr_arn);
    assert!(comp_arn.contains("rew-"));
    assert!(hr_arn.contains("hr-"));
}

#[tokio::test]
async fn immutable_append_only() {
    let ctx = TestContext::with_comp_record().await;
    let r = ctx.try_update_comp(ctx.comp_id, 50000000).await;
    assert!(r.is_err());
    let r2 = ctx.set_comp_correction(ctx.comp_id, 50000000).await;
    assert!(r2.is_ok());
    let row = ctx.fetch_comp_row(r2.new_comp_id).await;
    assert_eq!(row.correction_of, Some(ctx.comp_id));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-001.
**Downstream:** FR-REW-002 (versioning), FR-REW-003 (P1 invariant), FR-REW-005 (payroll), FR-REW-007 (BP), FR-REW-010 (memory exclusion).
**Cross-module:** FR-AUTH-105 (KMS), FR-AUTH-101 (CFO role), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| KMS decrypt fail | KMS err | sev-1 | retry; check IAM |
| Non-CFO bypass | role check | 403 + sev-1 | investigate |
| Cross-tenant KMS | RLS + arn check | 403 + sev-1 | inherent |
| Income kind invalid | CHECK | 400 | use valid |
| Currency invalid | validate | 400 | use ISO 4217 |
| Decimal precision loss | rust_decimal | inherent | inherent |
| Concurrent comp set | inherent | both append | inherent |
| Correction chain too deep | sanity warn | sev-3 | inherent |
| Cross-tenant comp view | RLS | 0 rows | inherent |
| KMS key disabled | sev-1 | inherent | CISO action |

## §11 — Implementation notes
- §11.1 KMS key created at tenant provisioning via FR-AUTH-105 KMS module; alias `rew-{tenant_id}`.
- §11.2 Amount stored encrypted at row level; decryption per-row via KMS.
- §11.3 Encryption envelope: amount_vnd → JSON → encrypt → BYTEA.
- §11.4 CFO email notification on decrypt via FR-EMAIL-009 (sev-1 path).
- §11.5 memory audit body: comp_id, member_id, income_kind, access_kind; amount NEVER in chain.

---

*End of FR-REW-001 spec.*
