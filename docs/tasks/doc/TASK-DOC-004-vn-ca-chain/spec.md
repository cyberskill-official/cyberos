---
id: TASK-DOC-004
title: "DOC VN CA chain — VNeID + VnPay/MK Group/Viettel-CA partners for VN-residency qualified digital signatures per Decree 130/2018"
module: DOC
priority: MUST
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
  - website/docs/modules/doc.html#vn-ca
  - https://thuvienphapluat.vn/  # Decree 130/2018 on electronic signatures

source_decisions:
  - DEC-1790 2026-05-17 — Partner with VN root-CA-trusted providers (VnPay, MK Group, Viettel-CA — selection per CLO); abstraction layer
  - DEC-1791 2026-05-17 — Closed enum `vn_ca_partner` = {vnpay, mk_group, viettel_ca}; cardinality 3
  - DEC-1792 2026-05-17 — Closed enum `vn_ca_request_kind` = {certificate_enroll, signature_request, vneid_link, validation_check, revocation_check}; cardinality 5
  - DEC-1793 2026-05-17 — VNeID linkage: signers verify identity via VNeID (national ID app) → CA issues qualified cert per Decree 130
  - DEC-1794 2026-05-17 — Per-tenant creds in KMS (CISO-only); tenant chooses partner at residency=vn-1 setup
  - DEC-1795 2026-05-17 — Returns signature in VN-compliant format (CMS + RootCA trust chain to VN National Root CA); composes with TASK-DOC-011
  - DEC-1796 2026-05-17 — memory audit kinds: doc.vn_ca_signature_requested, doc.vn_ca_signature_received, doc.vn_ca_vneid_linked, doc.vn_ca_cert_validated, doc.vn_ca_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0010_vn_ca_signatures.sql
    - services/doc/src/vn_ca/mod.rs
    - services/doc/src/vn_ca/abstraction.rs
    - services/doc/src/vn_ca/vnpay_client.rs
    - services/doc/src/vn_ca/mk_group_client.rs
    - services/doc/src/vn_ca/viettel_ca_client.rs
    - services/doc/src/vn_ca/vneid_linker.rs
    - services/doc/src/vn_ca/vn_root_validator.rs
    - services/doc/src/handlers/vn_ca_routes.rs
    - services/doc/src/audit/vn_ca_events.rs
    - services/doc/tests/vn_ca_vnpay_test.rs
    - services/doc/tests/vn_ca_mk_group_test.rs
    - services/doc/tests/vn_ca_viettel_test.rs
    - services/doc/tests/vn_ca_vneid_linkage_test.rs
    - services/doc/tests/vn_ca_root_validation_test.rs
    - services/doc/tests/vn_ca_partner_enum_cardinality_test.rs
    - services/doc/tests/vn_ca_request_kind_enum_cardinality_test.rs
    - services/doc/tests/vn_ca_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/{doc,auth}/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test vn_ca

  disallowed_tools:
    - non-CISO creds write (per DEC-1794)
    - skip VN Root CA validation (per DEC-1790)

effort_hours: 16
subtasks:
  - "0.5h: 0010_vn_ca_signatures.sql"
  - "0.6h: vn_ca/mod.rs"
  - "0.8h: abstraction.rs"
  - "2.0h: vnpay_client.rs"
  - "1.8h: mk_group_client.rs"
  - "1.7h: viettel_ca_client.rs"
  - "1.2h: vneid_linker.rs"
  - "1.0h: vn_root_validator.rs"
  - "0.5h: handlers/vn_ca_routes.rs"
  - "0.4h: audit/vn_ca_events.rs"
  - "4.0h: tests — 8 test files"
  - "1.5h: docs + CISO UI"

risk_if_skipped: "Without VN CA chain, VN-residency contracts can't get qualified digital signatures → not legally binding per Decree 130/2018. Without DEC-1793 VNeID linkage, signer identity unverifiable. Without DEC-1794 CISO gate, VN CA creds compromise = mass forgery."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship VN CA integration at `services/doc/src/vn_ca/` with 3 partners + VNeID linkage + VN Root CA validation, 5 memory audit kinds.

1. **MUST** support 3 partners per DEC-1790 — VnPay, MK Group, Viettel-CA; abstraction at `abstraction.rs::dispatcher`.

2. **MUST** validate `vn_ca_partner` enum cardinality 3 per DEC-1791.

3. **MUST** validate `vn_ca_request_kind` enum cardinality 5 per DEC-1792.

4. **MUST** support VNeID identity linkage per DEC-1793 at `vneid_linker.rs::link(signer, vneid_token)`:
   - Verify VNeID token via gov OAuth (TASK-DOC-006 vneid handler shared).
   - Submit verified identity to chosen CA for qualified cert enrollment.
   - Store cert in tenant's KMS for signer reuse.

5. **MUST** dispatch per partner — each implements `enroll_cert(signer, vneid_verified)`, `sign(pdf_hash, cert_id)`, `validate_chain(cert)`, `revoke(cert_id)`.

6. **MUST** validate chain to VN National Root CA per DEC-1790 at `vn_root_validator.rs::validate(cert)` — anchor must be in VN gov root list (refreshed quarterly).

7. **MUST** return CMS signature with VN trust chain per DEC-1795; TASK-DOC-011 extends to LT.

8. **MUST** store creds in KMS per DEC-1794 — CISO-only.

9. **MUST** define tables at migration `0010`:
   ```sql
   CREATE TABLE tenant_vn_ca_creds (
     tenant_id UUID PRIMARY KEY,
     partner TEXT NOT NULL CHECK (partner IN ('vnpay','mk_group','viettel_ca')),
     encrypted_creds_arn TEXT NOT NULL,
     api_account_id TEXT,
     set_by UUID NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE tenant_vn_ca_creds ENABLE ROW LEVEL SECURITY;
   CREATE POLICY vn_ca_creds_rls ON tenant_vn_ca_creds
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (partner, encrypted_creds_arn, api_account_id, set_by, updated_at) ON tenant_vn_ca_creds TO cyberos_app;

   CREATE TABLE doc_vn_ca_signatures (
     vn_ca_sig_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     signer_id UUID NOT NULL,
     vneid_subject_id TEXT,
     partner TEXT NOT NULL CHECK (partner IN ('vnpay','mk_group','viettel_ca')),
     request_kind TEXT NOT NULL
       CHECK (request_kind IN ('certificate_enroll','signature_request','vneid_link','validation_check','revocation_check')),
     cert_chain_pem TEXT NOT NULL,
     signature_value BYTEA NOT NULL,
     timestamp_token BYTEA,
     vn_root_validated BOOLEAN NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','succeeded','failed','revoked')),
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE doc_vn_ca_signatures ENABLE ROW LEVEL SECURITY;
   CREATE POLICY vn_ca_sigs_rls ON doc_vn_ca_signatures
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_vn_ca_signatures FROM cyberos_app;
   GRANT UPDATE (status) ON doc_vn_ca_signatures TO cyberos_app;
   ```

10. **MUST** expose endpoints:
    ```text
    PUT    /v1/doc/vn-ca/creds                   (CISO-only)
    POST   /v1/doc/vn-ca/sign                    (internal — TASK-DOC-005 caller)
    POST   /v1/doc/vn-ca/vneid-link              (signer-initiated VNeID linkage)
    GET    /v1/doc/vn-ca/signatures/{doc_id}
    ```

11. **MUST** emit 5 memory audit kinds per DEC-1796. PII per TASK-MEMORY-111: vneid_subject_id SHA-256 hashed (national ID is sensitive); signature + cert chain hashed.

12. **MUST** thread trace_id end-to-end.

13. **MUST NOT** allow non-CISO creds per DEC-1794.

14. **MUST NOT** skip VN Root CA validation per DEC-1790.

15. **MUST NOT** sign without VNeID-linked identity for qualified signatures per DEC-1793.

---

## §2 — Why this design

**Why VN-domestic partners (DEC-1790)?** Decree 130/2018 requires VN root CA chain for qualified signatures; EU/US CAs not accepted by VN courts.

**Why 3 partners (DEC-1791)?** Market diversity; CLO chooses based on industry vertical (VnPay = fintech focus, Viettel = enterprise, MK Group = SME).

**Why VNeID linkage (DEC-1793)?** Decree 130 requires verified national identity for qualified-level signatures; VNeID is the gov-blessed identity rail.

**Why CISO creds (DEC-1794)?** VN CA creds compromise = mass forgery on VN contracts; CISO has authority.

---

## §3 — API contract

```text
PUT    /v1/doc/vn-ca/creds                body: {partner, creds}
POST   /v1/doc/vn-ca/sign                 body: {document_id, signer_id, signer_cert_id}
POST   /v1/doc/vn-ca/vneid-link           body: {signer_id, vneid_oauth_callback_token}
GET    /v1/doc/vn-ca/signatures/{doc_id}
```

Sample VNeID link:
```json
{
  "signer_id": "uuid",
  "vneid_oauth_callback_token": "encoded-token"
}
```

Response:
```json
{
  "vneid_subject_id": "VN-citizen-id-hash",
  "cert_enrolled": true,
  "cert_id": "uuid",
  "expires_at": "2027-05-17"
}
```

---

## §4 — Acceptance criteria
1. **3 partners + cardinality test**. 2. **VnPay client works**. 3. **MK Group client works**. 4. **Viettel-CA client works**. 5. **VNeID link required for qualified**. 6. **VN Root CA validation enforced**. 7. **5 request_kind enum**. 8. **CISO-only creds**. 9. **Creds in KMS**. 10. **5 memory audit kinds emitted**. 11. **PII scrubbed (vneid_subject_id+signature+cert SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **TASK-DOC-005 integration**. 15. **Append-only sigs via REVOKE except status**. 16. **Non-VN chain rejected**. 17. **Sandbox + prod env per partner**. 18. **Cert revoke path**. 19. **Composes with TASK-DOC-011 for LT**. 20. **VNeID linkage 1-per-signer (idempotent)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn vnpay_signature_with_vneid_link() {
    let ctx = TestContext::with_vnpay_creds_and_vneid_signer().await;
    let r = ctx.vn_ca_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.vn_root_validated);
    assert_eq!(r.partner, "vnpay");
    assert!(r.vneid_subject_id.is_some());
}

#[tokio::test]
async fn non_vneid_signer_rejected_for_qualified() {
    let ctx = TestContext::vn_signer_no_vneid().await;
    let r = ctx.try_vn_ca_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn non_vn_root_chain_rejected() {
    let ctx = TestContext::with_eu_chain_cert().await;
    let r = ctx.try_vn_ca_sign(ctx.doc_id, ctx.signer_id).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-001.
**Cross-module:** TASK-DOC-005 (caller), TASK-DOC-006 (VNeID handler shared), TASK-DOC-011 (LT extend), TASK-AUTH-105 (KMS), TASK-AUTH-101 (CISO), TASK-MEMORY-111 (PII).

## §10 — Failure modes (similar to DOC-002/003)
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Partner API down | retry | sev-1; failed | switch partner |
| VNeID OAuth denied | callback | failed_no_match | retry |
| Cert chain not VN root | validator | reject; sev-1 | new enroll |
| Cert revoked | OCSP | status=revoked | new cert |
| Creds expired | 401 | sev-1; CISO | rotate |
| VN Root CA list lag | quarterly refresh | sev-3 | maintenance |
| Sandbox vs prod confusion | env flag | hard reject mismatch | inherent |
| Cross-tenant cert use | RLS | 403 | inherent |
| Signer VNeID not VN citizen | VNeID rejects | failed_no_match | inherent |
| Multi-partner conflict (tenant switched) | use current creds row | last-wins | inherent |

## §11 — Implementation notes
- §11.1 Each partner has REST API; VnPay = OAuth + signature endpoint, MK Group + Viettel similar.
- §11.2 VNeID OAuth: gov-managed identity rail; tokens short-lived (5min); cert enrollment downstream.
- §11.3 VN Root CA list maintained by Ministry of Information & Communication; refreshed quarterly.
- §11.4 memory audit body: doc_id, signer_id, partner, vn_root_validated; signatures + vneid_subject_id SHA256.
- §11.5 Future partners: extend enum + new client file (abstraction is the seam).

---

*End of TASK-DOC-004 spec.*
