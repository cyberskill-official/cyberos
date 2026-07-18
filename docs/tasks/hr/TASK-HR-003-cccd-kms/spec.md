---
id: TASK-HR-003
title: "HR CCCD photo KMS — separate keyspace for VN citizen ID photos with sev-1 access audit + ROOT-CHRO-only key access"
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
owner: Stephen Cheng (CISO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-AUTH-105, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: []

source_pages:
  - website/docs/modules/hr.html#cccd-kms
  # Law 91/2025 PDPL Art. 23 (sensitive personal data)
  - https://thuvienphapluat.vn/

source_decisions:
  - DEC-1820 2026-05-17 — CCCD photo encrypted with separate KMS key per-tenant; key access ROOT-CHRO only; no system service has decrypt grant
  - DEC-1821 2026-05-17 — Closed enum `cccd_access_kind` = {upload, decrypt_view, rotate_key, delete}; cardinality 4
  - DEC-1822 2026-05-17 — Every access (especially decrypt) emits sev-1 memory audit + CHRO email notification
  - DEC-1823 2026-05-17 — PDPL Law 91/2025 Art. 23 compliance: CCCD = sensitive personal data; requires explicit consent at upload
  - DEC-1824 2026-05-17 — memory audit kinds: hr.cccd_uploaded, hr.cccd_decrypted, hr.cccd_access_denied, hr.cccd_key_rotated, hr.cccd_deleted

build_envelope:
  language: rust 1.81
  service: cyberos/services/hr/
  new_files:
    - services/hr/migrations/0003_cccd_storage.sql
    - services/hr/src/cccd/mod.rs
    - services/hr/src/cccd/kms_wrapper.rs
    - services/hr/src/cccd/access_gate.rs
    - services/hr/src/handlers/cccd_routes.rs
    - services/hr/src/audit/cccd_events.rs
    - services/hr/tests/cccd_upload_test.rs
    - services/hr/tests/cccd_decrypt_root_chro_only_test.rs
    - services/hr/tests/cccd_access_kind_enum_cardinality_test.rs
    - services/hr/tests/cccd_consent_required_test.rs
    - services/hr/tests/cccd_sev1_audit_emission_test.rs

  modified_files:
    - services/hr/src/lib.rs

  allowed_tools:
    - file_read: services/{hr,auth}/**
    - file_write: services/hr/{src,tests,migrations}/**
    - bash: cd services/hr && cargo test cccd

  disallowed_tools:
    - decrypt CCCD via non-CHRO role (per DEC-1820)
    - upload without consent (per DEC-1823)

effort_hours: 5
subtasks:
  - "0.3h: 0003_cccd_storage.sql"
  - "0.3h: cccd/mod.rs"
  - "0.5h: kms_wrapper.rs (separate keyspace)"
  - "0.4h: access_gate.rs (ROOT-CHRO check)"
  - "0.4h: handlers/cccd_routes.rs"
  - "0.3h: audit/cccd_events.rs"
  - "2.0h: tests — 5 test files"
  - "0.8h: docs + UI for upload consent + CHRO decrypt"

risk_if_skipped: "Without separate KMS keyspace, single key compromise leaks all citizen IDs → PDPL violation + class-action exposure. Without DEC-1822 sev-1 audit, internal abuse goes undetected. Without DEC-1823 consent, PDPL Art. 23 violation."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship CCCD photo encryption at `services/hr/src/cccd/` with separate KMS keyspace per tenant, ROOT-CHRO-only decrypt, sev-1 audit per access, consent gate, 5 memory audit kinds.

1. **MUST** validate `cccd_access_kind` against closed enum per DEC-1821.

2. **MUST** encrypt at `kms_wrapper.rs::encrypt(photo_bytes, tenant_id)`:
   - Use tenant-specific KMS CMK with `KeyUsage=ENCRYPT_DECRYPT`, alias=`hr-cccd-{tenant_id}`.
   - Key created at tenant provisioning; rotation via CHRO action.

3. **MUST** require consent per DEC-1823 — `consent_token` parameter required at upload; validated against member's prior consent record.

4. **MUST** gate decrypt at `access_gate.rs::check(user, tenant)` per DEC-1820:
   - Caller must have ROOT-CHRO role via TASK-AUTH-101.
   - Other roles → 403 + emit `hr.cccd_access_denied` sev-1.

5. **MUST** emit sev-1 audit on every decrypt per DEC-1822:
   - `hr.cccd_decrypted` with severity=1
   - CHRO email notification dispatched

6. **MUST** define tables at migration `0003`:
   ```sql
   CREATE TABLE hr_cccd_storage (
     storage_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL UNIQUE,
     encrypted_photo BYTEA NOT NULL,
     encryption_kms_key_arn TEXT NOT NULL,
     consent_token UUID NOT NULL,
     consent_at TIMESTAMPTZ NOT NULL,
     uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     trace_id CHAR(32)
   );
   ALTER TABLE hr_cccd_storage ENABLE ROW LEVEL SECURITY;
   CREATE POLICY cccd_storage_rls ON hr_cccd_storage
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_cccd_storage FROM cyberos_app;
   GRANT DELETE ON hr_cccd_storage TO cyberos_app;  -- CHRO can delete

   CREATE TABLE hr_cccd_access_log (
     log_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     accessor_id UUID NOT NULL,
     access_kind TEXT NOT NULL
       CHECK (access_kind IN ('upload','decrypt_view','rotate_key','delete')),
     accessor_role TEXT NOT NULL,
     ip_address TEXT,  -- hashed
     succeeded BOOLEAN NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX cccd_access_log_member_time_idx ON hr_cccd_access_log(tenant_id, member_id, created_at DESC);
   ALTER TABLE hr_cccd_access_log ENABLE ROW LEVEL SECURITY;
   CREATE POLICY cccd_log_rls ON hr_cccd_access_log
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_cccd_access_log FROM cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/hr/members/{id}/cccd-photo          body: {photo_bytes, consent_token} (member-self or CHRO)
   GET    /v1/hr/members/{id}/cccd-photo          (ROOT-CHRO only; decrypts + audits sev-1)
   POST   /v1/hr/members/{id}/cccd-photo/rotate   (CHRO only — re-encrypt with new key)
   DELETE /v1/hr/members/{id}/cccd-photo          (member-self via DSAR or CHRO)
   ```

8. **MUST** emit 5 memory audit kinds per DEC-1824. PII per TASK-MEMORY-111: encrypted_photo not in chain; access_kind + role ok.

9. **MUST** thread trace_id through all access paths.

10. **MUST NOT** decrypt for non-ROOT-CHRO per DEC-1820.

11. **MUST NOT** upload without consent_token per DEC-1823.

---

## §2 — Why this design

**Why separate KMS keyspace (DEC-1820)?** Blast radius — compromise of system-wide key would leak all CCCDs; per-tenant key limits exposure to one tenant.

**Why ROOT-CHRO-only decrypt (DEC-1820)?** CCCD = government ID = sensitive; principle of least privilege. Audit team can't decrypt without explicit grant.

**Why sev-1 on every decrypt (DEC-1822)?** Internal abuse most common attack; every access surfaces immediately for CHRO review.

**Why consent token (DEC-1823)?** PDPL Law 91/2025 Art. 23 explicitly requires explicit consent for sensitive PII collection; token is consent proof.

---

## §3 — API contract (see §1.7)

Sample consent recording (precondition to upload):
```json
POST /v1/hr/members/{id}/cccd-consent
{ "consent_purpose": "ID verification for employment", "consent_text_hash": "sha256..." }
```

Returns `consent_token` UUID for use in upload.

---

## §4 — Acceptance criteria
1. **CCCD encrypted with separate KMS keyspace**. 2. **ROOT-CHRO-only decrypt**. 3. **Non-CHRO decrypt → 403 + access_denied sev-1 audit**. 4. **Sev-1 audit on every successful decrypt**. 5. **CHRO email notification on decrypt**. 6. **Consent token required for upload**. 7. **access_kind enum cardinality 4**. 8. **Per-tenant key alias**. 9. **Key rotation re-encrypts**. 10. **5 memory audit kinds emitted**. 11. **PII: encrypted_photo never in chain**. 12. **RLS denies cross-tenant**. 13. **IP address SHA256 hashed**. 14. **Trace_id preserved**. 15. **Access log immutable (no UPDATE/DELETE grant)**. 16. **One CCCD per member (UNIQUE)**. 17. **Delete via DSAR or CHRO action**. 18. **Member-self can upload + delete own**. 19. **Failed access logged with succeeded=false**. 20. **Key creation idempotent at tenant provisioning**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn non_chro_decrypt_denied() {
    let ctx = TestContext::with_uploaded_cccd().await;
    let r = ctx.try_decrypt_as(ctx.am_user, ctx.member_id).await;
    assert_eq!(r.status_code, 403);
    let logs = ctx.fetch_access_log(ctx.member_id).await;
    assert!(logs.iter().any(|l| l.access_kind == "decrypt_view" && !l.succeeded));
}

#[tokio::test]
async fn chro_decrypt_emits_sev1() {
    let ctx = TestContext::with_uploaded_cccd_as_chro().await;
    let r = ctx.decrypt_as(ctx.chro_user, ctx.member_id).await;
    assert!(r.is_ok());
    let audits = ctx.fetch_memory_audits("hr.cccd_decrypted").await;
    assert!(audits.iter().any(|a| a.severity == 1));
}

#[tokio::test]
async fn upload_without_consent_rejected() {
    let ctx = TestContext::with_member_no_consent().await;
    let r = ctx.upload_cccd_no_token(ctx.member_id).await;
    assert_eq!(r.status_code, 412);  // Precondition Failed
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001.
**Cross-module:** TASK-AUTH-105 (ROOT-CHRO role), TASK-MEMORY-111 (audit chain), TASK-AUTH-101 (RBAC).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| KMS decrypt fail | KMS err | sev-1; failed audit | retry; check IAM |
| Non-CHRO bypass attempt | role check | 403; sev-1 | investigate |
| Consent token expired | validate | 412 | re-consent |
| Photo size > 5MB | validate | 400 | resize |
| Key rotation mid-decrypt | re-fetch | inherent | inherent |
| Multiple consent tokens same member | use latest valid | inherent | inherent |
| KMS key disabled | sev-1 alert | inherent | CISO action |
| Cross-tenant decrypt attempt | RLS | 0 rows + sev-1 | inherent |
| DSAR delete after upload | cascade | inherent | per TASK-PORTAL-008 |
| Member-self decrypts own | OK (still sev-1) | inherent | inherent |

## §11 — Implementation notes
- §11.1 KMS key created at tenant provisioning via TASK-AUTH-105 KMS module.
- §11.2 Photo stored as encrypted bytes in S3 (TASK-DOC-001 path) + reference in DB.
- §11.3 CHRO email notification: separate TASK-EMAIL-009 send with subject "CCCD decrypted: {member_name}".
- §11.4 memory audit body: member_id, accessor_id, access_kind, succeeded; IP SHA256.
- §11.5 Consent token: signed JWT, 24h expiry, single-use; binds member_id + consent_purpose.

---

*End of TASK-HR-003 spec.*
