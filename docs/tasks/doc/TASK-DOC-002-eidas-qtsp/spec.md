---
id: TASK-DOC-002
title: "DOC eIDAS QTSP integration — GlobalSign or Cryptomathic partner for EU residency qualified signatures"
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
module: doc
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
blocks: [TASK-DOC-011]

source_pages:
  - website/docs/modules/doc.html#eidas-qtsp
  # eIDAS Regulation 910/2014
  - https://eur-lex.europa.eu/eli/reg/2014/910/oj

source_decisions:
  - DEC-1770 2026-05-17 — Partner with QTSP (GlobalSign or Cryptomathic — vendor selection per CLO/CISO); abstraction layer hides specifics
  - DEC-1771 2026-05-17 — Closed enum `qtsp_partner` = {globalsign, cryptomathic}; cardinality 2 (extensible)
  - DEC-1772 2026-05-17 — Closed enum `qtsp_request_kind` = {certificate_request, signature_request, validation_request, revocation_check}; cardinality 4
  - DEC-1773 2026-05-17 — Per-tenant QTSP creds in KMS (CISO-only write); tenant chooses partner at residency=eu-1 setup
  - DEC-1774 2026-05-17 — Returns PAdES-B-LT signature with full LTV chain (cert + OCSP/CRL + timestamp); composes with TASK-DOC-011
  - DEC-1775 2026-05-17 — memory audit kinds: doc.qtsp_signature_requested, doc.qtsp_signature_received, doc.qtsp_cert_validated, doc.qtsp_failed

language: rust 1.81
service: cyberos/services/doc/
new_files:
  - services/doc/migrations/0008_qtsp_signatures.sql
  - services/doc/src/qtsp/mod.rs
  - services/doc/src/qtsp/abstraction.rs
  - services/doc/src/qtsp/globalsign_client.rs
  - services/doc/src/qtsp/cryptomathic_client.rs
  - services/doc/src/qtsp/cert_chain_validator.rs
  - services/doc/src/handlers/qtsp_routes.rs
  - services/doc/src/audit/qtsp_events.rs
  - services/doc/tests/qtsp_globalsign_test.rs
  - services/doc/tests/qtsp_cryptomathic_test.rs
  - services/doc/tests/qtsp_padesblt_format_test.rs
  - services/doc/tests/qtsp_partner_enum_cardinality_test.rs
  - services/doc/tests/qtsp_request_kind_enum_cardinality_test.rs
  - services/doc/tests/qtsp_audit_emission_test.rs

modified_files:
  - services/doc/src/lib.rs

allowed_tools:
  - file_read: services/{doc,auth}/**
  - file_write: services/doc/{src,tests,migrations}/**
  - bash: cd services/doc && cargo test qtsp

disallowed_tools:
  - non-CISO creds write (per DEC-1773)
  - skip cert chain validation (per DEC-1774)

effort_hours: 16
subtasks:
  - "0.5h: 0008_qtsp_signatures.sql"
  - "0.6h: qtsp/mod.rs"
  - "0.8h: abstraction.rs (partner dispatcher)"
  - "2.5h: globalsign_client.rs (DSS REST)"
  - "2.0h: cryptomathic_client.rs (DSS API)"
  - "1.2h: cert_chain_validator.rs"
  - "0.5h: handlers/qtsp_routes.rs"
  - "0.4h: audit/qtsp_events.rs"
  - "5.5h: tests — 6 test files w/ vendor mocks"
  - "1.5h: docs + CISO UI for partner selection + creds entry"
  - "0.5h: integration smoke with sandbox QTSP"

risk_if_skipped: "Without eIDAS QTSP, EU contracts lack qualified signature → not legally equivalent to handwritten (eIDAS Art. 25). Without DEC-1774 PAdES-B-LT, signature loses LTV → invalid years later. Without DEC-1773 CISO gate, QTSP creds compromise = mass fraud."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship eIDAS QTSP integration at `services/doc/src/qtsp/` with partner abstraction, cert chain validation, PAdES-B-LT signatures, 4 memory audit kinds.

1. **MUST** support 2 partners per DEC-1770 — GlobalSign + Cryptomathic; abstraction at `abstraction.rs::dispatcher(partner)`.

2. **MUST** validate `qtsp_partner` against closed enum per DEC-1771.

3. **MUST** validate `qtsp_request_kind` against closed enum per DEC-1772.

4. **MUST** dispatch per partner:
- `globalsign_client.rs::request_signature(pdf_hash, signer_cert_id, ts_authority)` — GlobalSign DSS REST
- `cryptomathic_client.rs::sign(pdf, signer_cert_id)` — Cryptomathic Signer API

5. **MUST** validate returned cert chain at `cert_chain_validator.rs::validate(signature)`:
- All certs in chain non-expired at signature time.
- Issuer chain to EU Trust List root.
- OCSP/CRL revocation check.
- Timestamp authority signed.

6. **MUST** return PAdES-B-LT signature per DEC-1774 — embedded validation data (cert chain + OCSP/CRL + timestamp) per ETSI EN 319 142-1.

7. **MUST** store QTSP creds in KMS per DEC-1773 — `tenant_qtsp_creds.encrypted_creds_arn`; CISO-only writes via TASK-AUTH-101.

8. **MUST** define tables at migration `0008`:
   ```sql
   CREATE TABLE tenant_qtsp_creds (
     tenant_id UUID PRIMARY KEY,
     partner TEXT NOT NULL CHECK (partner IN ('globalsign','cryptomathic')),
     encrypted_creds_arn TEXT NOT NULL,
     api_account_id TEXT,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE tenant_qtsp_creds ENABLE ROW LEVEL SECURITY;
   CREATE POLICY qtsp_creds_rls ON tenant_qtsp_creds
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (partner, encrypted_creds_arn, api_account_id, set_by, updated_at) ON tenant_qtsp_creds TO cyberos_app;

   CREATE TABLE doc_qtsp_signatures (
     qtsp_sig_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     signer_id UUID NOT NULL,
     partner TEXT NOT NULL CHECK (partner IN ('globalsign','cryptomathic')),
     request_kind TEXT NOT NULL CHECK (request_kind IN ('certificate_request','signature_request','validation_request','revocation_check')),
     cert_chain_pem TEXT NOT NULL,
     signature_value BYTEA NOT NULL,
     timestamp_token BYTEA NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','succeeded','failed','revoked')),
     failure_reason TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE doc_qtsp_signatures ENABLE ROW LEVEL SECURITY;
   CREATE POLICY qtsp_sigs_rls ON doc_qtsp_signatures
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_qtsp_signatures FROM cyberos_app;
   GRANT UPDATE (status, failure_reason) ON doc_qtsp_signatures TO cyberos_app;
   ```

9. **MUST** expose endpoints:
   ```text
   PUT    /v1/doc/qtsp/creds                  (CISO-only)
   POST   /v1/doc/qtsp/sign                   (internal — called by TASK-DOC-005)
   GET    /v1/doc/qtsp/signatures/{doc_id}    (audit query)
   ```

10. **MUST** emit 4 memory audit kinds per DEC-1775. PII per TASK-MEMORY-111: signature_value + cert_chain hashed; ids ok.

11. **MUST** thread trace_id from TASK-DOC-005 request → partner call → cert validation → audit.

12. **MUST NOT** allow non-CISO creds write per DEC-1773.

13. **MUST NOT** skip cert chain validation per DEC-1774.

---

## §2 — Why this design

**Why partner abstraction (DEC-1770)?** Vendor diversity reduces lock-in + enables failover; abstraction hides API differences.

**Why GlobalSign + Cryptomathic (DEC-1771)?** Both EU Trust List qualified; both have stable REST APIs; future partners extensible.

**Why CISO-gated creds (DEC-1773)?** QTSP creds = signing authority; compromise = mass fraud on EU contracts.

**Why PAdES-B-LT (DEC-1774)?** ETSI standard; enables LTV (long-term validation); TASK-DOC-011 re-stamps at year-9 to keep valid.

---

## §3 — API contract

```text
PUT    /v1/doc/qtsp/creds              body: {partner, creds, api_account_id}
POST   /v1/doc/qtsp/sign               body: {document_id, signer_id, signer_cert_request}
GET    /v1/doc/qtsp/signatures/{doc_id}
```

Sample sign request (internal, TASK-DOC-005 caller):
```json
{
  "document_id": "uuid",
  "signer_id": "uuid",
  "signer_cert_request": "PEM-encoded CSR"
}
```

Response:
```json
{
  "qtsp_sig_id": "uuid",
  "partner": "globalsign",
  "signature_value": "base64",
  "cert_chain_pem": "-----BEGIN CERTIFICATE-----...",
  "timestamp_token": "base64",
  "padesblt_blob_s3_key": "..."
}
```

---

## §4 — Acceptance criteria
1. **2 partners + enum cardinality test**. 2. **GlobalSign DSS REST works**. 3. **Cryptomathic Signer API works**. 4. **Cert chain validated to EU Trust List root**. 5. **OCSP/CRL revocation checked**. 6. **PAdES-B-LT format returned**. 7. **Timestamp authority signed**. 8. **request_kind enum cardinality 4**. 9. **CISO-only creds (403 for others)**. 10. **Creds in KMS only**. 11. **4 memory audit kinds emitted**. 12. **PII scrubbed (signature value + cert chain SHA256)**. 13. **RLS denies cross-tenant**. 14. **Trace_id preserved**. 15. **TASK-DOC-005 integration works**. 16. **Append-only sigs table via REVOKE except status cols**. 17. **Revoked cert detection → status=revoked**. 18. **Partner failover (if A down, manual switch to B)**. 19. **Composes with TASK-DOC-011 for LTV re-stamping**. 20. **Sandbox + prod environments per partner**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn globalsign_signature_round_trip() {
    let ctx = TestContext::with_globalsign_creds_sandbox().await;
    let r = ctx.qtsp_sign(ctx.doc_id, ctx.signer_id).await;
    assert_eq!(r.partner, "globalsign");
    assert!(!r.signature_value.is_empty());
    let validated = ctx.validate_chain(&r.cert_chain_pem).await;
    assert!(validated.eu_trust_list_root);
}

#[tokio::test]
async fn cert_chain_revocation_caught() {
    let ctx = TestContext::with_revoked_cert().await;
    let r = ctx.qtsp_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.is_err() || r.unwrap().status == "revoked");
}

#[tokio::test]
async fn padesblt_format_returned() {
    let ctx = TestContext::with_qtsp_creds().await;
    let r = ctx.qtsp_sign(ctx.doc_id, ctx.signer_id).await;
    let blob = ctx.fetch_s3(&r.padesblt_blob_s3_key).await;
    assert!(blob.starts_with(b"%PDF"));  // PAdES is PDF
    assert!(ctx.parse_pades_lt(blob).has_validation_data());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-001. **Cross-module:** TASK-DOC-005 (caller), TASK-DOC-006 (verification gate), TASK-DOC-011 (LTV re-stamping), TASK-AUTH-105 (KMS), TASK-AUTH-101 (CISO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Partner API down | retry 3x | sev-1; status=failed | switch partner or retry |
| Cert chain invalid | validator | status=failed; sev-1 | CSR re-issue |
| OCSP unreachable | fallback to CRL | sev-2; proceed | inherent |
| TS authority down | use alternate | sev-2 | inherent |
| Revoked signer cert | validator | status=revoked | new cert |
| Creds expired | 401 | sev-1; CISO notified | rotate |
| Sandbox vs prod confusion | env flag | hard reject if mismatch | inherent |
| Network partition mid-sign | retry idempotently | inherent | inherent |
| Cross-tenant cert use | RLS | 403 | inherent |
| EU Trust List update lag | refresh cache | sev-3 | maintenance |

## §11 — Implementation notes
- §11.1 GlobalSign DSS: REST endpoint per partner docs; signs PDF hash, returns CMS.
- §11.2 Cryptomathic: similar DSS shape; both return PAdES-compatible CMS structure.
- §11.3 Cert chain validator uses `x509-parser` + `oid-registry`; root list from EU Trust List XML.
- §11.4 memory audit body: doc_id, signer_id, partner, status; signature_value + cert_chain SHA256.
- §11.5 Future partners: add new enum value + new client file; abstraction is the seam.

---

*End of TASK-DOC-002 spec.*
