---
id: TASK-DOC-003
title: "DOC AATL CA integration — Adobe Approved Trust List CA partner (DigiCert / Entrust / IdenTrust) for US/non-EU/non-VN residency"
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
module: DOC
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 3
slice: 3
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-DOC-001, TASK-DOC-005, TASK-DOC-006, TASK-DOC-011, TASK-AUTH-105, TASK-MEMORY-111]
depends_on: [TASK-DOC-001]
blocks: []

source_pages:
  - website/docs/modules/doc.html#aatl-ca
  # AATL membership
  - https://helpx.adobe.com/acrobat/kb/approved-trust-list1.html

source_decisions:
  - DEC-1780 2026-05-17 — Partner with AATL-listed CA (DigiCert / Entrust / IdenTrust — selection per CLO); abstraction layer hides specifics
  - DEC-1781 2026-05-17 — Closed enum `aatl_partner` = {digicert, entrust, identrust}; cardinality 3
  - DEC-1782 2026-05-17 — Closed enum `aatl_request_kind` = {certificate_enroll, signature_request, validation_check, certificate_revoke}; cardinality 4
  - DEC-1783 2026-05-17 — Per-tenant AATL creds in KMS (CISO-only); tenant selects partner at first US/non-EU signing
  - DEC-1784 2026-05-17 — Returns PAdES-B-T signatures (similar to QTSP but US Adobe-trust chain); composes with TASK-DOC-011 LTV
  - DEC-1785 2026-05-17 — memory audit kinds: doc.aatl_signature_requested, doc.aatl_signature_received, doc.aatl_cert_validated, doc.aatl_failed

language: rust 1.81
service: cyberos/services/doc/
new_files:
  - services/doc/migrations/0009_aatl_signatures.sql
  - services/doc/src/aatl/mod.rs
  - services/doc/src/aatl/abstraction.rs
  - services/doc/src/aatl/digicert_client.rs
  - services/doc/src/aatl/entrust_client.rs
  - services/doc/src/aatl/identrust_client.rs
  - services/doc/src/aatl/aatl_root_validator.rs
  - services/doc/src/handlers/aatl_routes.rs
  - services/doc/src/audit/aatl_events.rs
  - services/doc/tests/aatl_digicert_test.rs
  - services/doc/tests/aatl_entrust_test.rs
  - services/doc/tests/aatl_identrust_test.rs
  - services/doc/tests/aatl_partner_enum_cardinality_test.rs
  - services/doc/tests/aatl_request_kind_enum_cardinality_test.rs
  - services/doc/tests/aatl_audit_emission_test.rs

modified_files:
  - services/doc/src/lib.rs

allowed_tools:
  - file_read: services/{doc,auth}/**
  - file_write: services/doc/{src,tests,migrations}/**
  - bash: cd services/doc && cargo test aatl

disallowed_tools:
  - non-CISO creds write (per DEC-1783)
  - skip AATL root validation (per DEC-1780)

effort_hours: 12
subtasks:
  - "0.5h: 0009_aatl_signatures.sql"
  - "0.5h: aatl/mod.rs"
  - "0.7h: abstraction.rs"
  - "2.0h: digicert_client.rs"
  - "1.8h: entrust_client.rs"
  - "1.5h: identrust_client.rs"
  - "1.0h: aatl_root_validator.rs"
  - "0.5h: handlers/aatl_routes.rs"
  - "0.3h: audit/aatl_events.rs"
  - "2.5h: tests — 6 test files"
  - "0.7h: docs + CISO UI"

risk_if_skipped: "Without AATL CA, US/global contracts can't get Adobe-trusted signatures → recipients see warnings in Acrobat. Without DEC-1783 CISO gate, AATL creds compromise = mass forgery. Without DEC-1784 PAdES-B-T composability, no LTV path via TASK-DOC-011."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship AATL CA integration at `services/doc/src/aatl/` with 3 partners + abstraction, AATL root validation, PAdES-B-T signatures, 4 memory audit kinds.

1. **MUST** support 3 partners per DEC-1780 — DigiCert, Entrust, IdenTrust; abstraction at `abstraction.rs::dispatcher(partner)`.

2. **MUST** validate `aatl_partner` against closed enum per DEC-1781.

3. **MUST** validate `aatl_request_kind` against closed enum per DEC-1782.

4. **MUST** dispatch per partner — each client implements identical interface: `enroll_cert()`, `sign(pdf_hash, cert_id)`, `validate_chain(cert)`, `revoke(cert_id)`.

5. **MUST** validate cert chain to AATL root at `aatl_root_validator.rs::validate(cert)` — chain anchor must be in Adobe AATL list (refreshed quarterly from Adobe).

6. **MUST** return PAdES-B-T signature per DEC-1784 — includes timestamp; composes with TASK-DOC-011 for LTV.

7. **MUST** store creds in KMS per DEC-1783 — CISO-only.

8. **MUST** define tables at migration `0009`:
   ```sql
   CREATE TABLE tenant_aatl_creds (
     tenant_id UUID PRIMARY KEY,
     partner TEXT NOT NULL CHECK (partner IN ('digicert','entrust','identrust')),
     encrypted_creds_arn TEXT NOT NULL,
     api_account_id TEXT,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE tenant_aatl_creds ENABLE ROW LEVEL SECURITY;
   CREATE POLICY aatl_creds_rls ON tenant_aatl_creds
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (partner, encrypted_creds_arn, api_account_id, set_by, updated_at) ON tenant_aatl_creds TO cyberos_app;

   CREATE TABLE doc_aatl_signatures (
     aatl_sig_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     signer_id UUID NOT NULL,
     partner TEXT NOT NULL CHECK (partner IN ('digicert','entrust','identrust')),
     request_kind TEXT NOT NULL
       CHECK (request_kind IN ('certificate_enroll','signature_request','validation_check','certificate_revoke')),
     cert_chain_pem TEXT NOT NULL,
     signature_value BYTEA NOT NULL,
     timestamp_token BYTEA NOT NULL,
     aatl_root_validated BOOLEAN NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','succeeded','failed','revoked')),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE doc_aatl_signatures ENABLE ROW LEVEL SECURITY;
   CREATE POLICY aatl_sigs_rls ON doc_aatl_signatures
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_aatl_signatures FROM cyberos_app;
   GRANT UPDATE (status) ON doc_aatl_signatures TO cyberos_app;
   ```

9. **MUST** expose endpoints (mirror DOC-002 shape):
   ```text
   PUT    /v1/doc/aatl/creds                  (CISO-only)
   POST   /v1/doc/aatl/sign                   (internal — TASK-DOC-005 caller)
   GET    /v1/doc/aatl/signatures/{doc_id}
   ```

10. **MUST** emit 4 memory audit kinds per DEC-1785. PII per TASK-MEMORY-111: signature + cert chain SHA-256 hashed.

11. **MUST** thread trace_id end-to-end.

12. **MUST NOT** allow non-CISO creds write per DEC-1783.

13. **MUST NOT** skip AATL root validation per DEC-1780 — non-AATL chain = signature unacceptable.

---

## §2 — Why this design

**Why AATL (DEC-1780)?** Adobe Reader auto-trusts AATL certs — recipients see "Signature Valid" without warnings. Critical UX for B2B.

**Why 3 partners (DEC-1781)?** Vendor diversity + price competition; all three are AATL-listed.

**Why CISO creds (DEC-1783)?** Same as QTSP — signing authority compromise.

**Why PAdES-B-T not B-LT (DEC-1784)?** AATL CAs typically issue B-T (timestamp only); TASK-DOC-011 re-stamps to B-LT at year-9.

---

## §3 — API contract (mirrors DOC-002)

See §1.9 for endpoints. Same payload shape as TASK-DOC-002.

---

## §4 — Acceptance criteria
1. **3 partners + cardinality test**. 2. **DigiCert client works**. 3. **Entrust client works**. 4. **IdenTrust client works**. 5. **AATL root validation enforced**. 6. **PAdES-B-T returned**. 7. **Timestamp included**. 8. **4-request-kind enum + cardinality**. 9. **CISO-only creds**. 10. **Creds in KMS**. 11. **4 memory audit kinds emitted**. 12. **PII scrubbed (sig+chain SHA256)**. 13. **RLS denies cross-tenant**. 14. **Trace_id preserved**. 15. **TASK-DOC-005 integration**. 16. **Append-only sigs via REVOKE except status**. 17. **Non-AATL chain rejected**. 18. **Sandbox + prod env per partner**. 19. **Cert revoke path**. 20. **Composes with TASK-DOC-011 for LTV**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn digicert_signature_aatl_validated() {
    let ctx = TestContext::with_digicert_creds().await;
    let r = ctx.aatl_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.aatl_root_validated);
    assert_eq!(r.partner, "digicert");
}

#[tokio::test]
async fn non_aatl_chain_rejected() {
    let ctx = TestContext::with_self_signed_cert().await;
    let r = ctx.try_aatl_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn padesblt_via_doc_011_after_aatl() {
    let ctx = TestContext::with_aatl_signed_doc().await;
    let bt_sig = ctx.fetch_aatl_sig(ctx.doc_id).await;
    let blt = ctx.doc011_extend_to_lt(bt_sig).await;  // TASK-DOC-011 re-stamp
    assert!(blt.has_validation_data());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-001.
**Cross-module:** TASK-DOC-005 (caller), TASK-DOC-006 (verify gate), TASK-DOC-011 (LTV re-stamp), TASK-AUTH-105 (KMS), TASK-AUTH-101 (CISO), TASK-MEMORY-111 (PII).

## §10 — Failure modes (mirror DOC-002)
Same shape — partner down, cert expired, AATL list lag, etc.

## §11 — Implementation notes
- §11.1 Each partner has REST API; auth via API key + (sometimes) mTLS.
- §11.2 AATL root list refreshed quarterly from Adobe; cached in service.
- §11.3 PAdES-B-T includes signature timestamp; TASK-DOC-011 extends to B-LT.
- §11.4 memory audit body: doc_id, signer_id, partner, aatl_root_validated; signatures SHA256.
- §11.5 Same abstraction pattern as DOC-002; allows future partner additions.

---

*End of TASK-DOC-003 spec.*
