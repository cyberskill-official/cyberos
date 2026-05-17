---
id: FR-DOC-006
title: "DOC identity verification — 4 methods (WebAuthn / VNeID / SMS-OTP / email-link) with per-document method selection + audit"
module: DOC
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CISO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-DOC-001, FR-AUTH-105, FR-DOC-005, FR-AI-003, FR-BRAIN-111]
depends_on: [FR-AUTH-105]
blocks: [FR-DOC-005]

source_pages:
  - website/docs/modules/doc.html#identity-verification
  - https://eur-lex.europa.eu/eli/reg/2014/910/oj  # eIDAS

source_decisions:
  - DEC-1740 2026-05-17 — 4 verification methods: webauthn (FIDO2/passkey), vneid (VN national ID), sms_otp, email_link; tenant configures per-document defaults
  - DEC-1741 2026-05-17 — Closed enum `verification_method` = {webauthn, vneid, sms_otp, email_link}; cardinality 4
  - DEC-1742 2026-05-17 — Closed enum `verification_result` = {verified, failed_invalid, failed_expired, failed_no_match}; cardinality 4
  - DEC-1743 2026-05-17 — eIDAS Level mapping: webauthn=high, vneid=substantial, sms_otp+email_link=low; document signing per FR-DOC-005 enforces minimum level
  - DEC-1744 2026-05-17 — Verification audit (immutable): method + result + assurance_level + signer_id + verification_at
  - DEC-1745 2026-05-17 — BRAIN audit kinds: doc.verification_initiated, doc.verification_succeeded, doc.verification_failed, doc.verification_expired

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0005_identity_verifications.sql
    - services/doc/src/verification/mod.rs
    - services/doc/src/verification/webauthn_handler.rs
    - services/doc/src/verification/vneid_handler.rs
    - services/doc/src/verification/otp_handler.rs
    - services/doc/src/verification/email_link_handler.rs
    - services/doc/src/handlers/verification_routes.rs
    - services/doc/src/audit/verification_events.rs
    - services/doc/tests/verification_webauthn_test.rs
    - services/doc/tests/verification_vneid_test.rs
    - services/doc/tests/verification_otp_test.rs
    - services/doc/tests/verification_email_link_test.rs
    - services/doc/tests/verification_method_enum_cardinality_test.rs
    - services/doc/tests/verification_result_enum_cardinality_test.rs
    - services/doc/tests/verification_eidas_level_test.rs
    - services/doc/tests/verification_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/{doc,auth}/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test verification

  disallowed_tools:
    - bypass eIDAS minimum (per DEC-1743)
    - mutate verification audit (per DEC-1744)

effort_hours: 8
sub_tasks:
  - "0.4h: 0005_identity_verifications.sql"
  - "0.5h: verification/mod.rs (dispatcher)"
  - "1.2h: webauthn_handler.rs (FIDO2 challenge+response)"
  - "1.0h: vneid_handler.rs (VN gov API)"
  - "0.8h: otp_handler.rs (SMS via Twilio or VN provider)"
  - "0.5h: email_link_handler.rs (magic-link token)"
  - "0.5h: handlers/verification_routes.rs"
  - "0.3h: audit/verification_events.rs"
  - "2.6h: tests — 7 test files"
  - "0.2h: docs"

risk_if_skipped: "Without verification, signatures lack signer identity proof → court-ineffective. Without DEC-1743 eIDAS level mapping, can't ship to EU customers. Without DEC-1744 immutable audit, verification claims unverifiable years later."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship identity verification at `services/doc/src/verification/` supporting 4 methods with eIDAS level mapping, immutable audit, 4 BRAIN audit kinds.

1. **MUST** support 4 methods per DEC-1740 — webauthn / vneid / sms_otp / email_link.

2. **MUST** validate `verification_method` against closed enum per DEC-1741, `verification_result` per DEC-1742.

3. **MUST** dispatch per method:
   - `webauthn_handler.rs::challenge()` + `verify(response)` — FIDO2/passkey
   - `vneid_handler.rs::redirect_to_vneid()` + `callback(token)` — VN national ID via gov OAuth
   - `otp_handler.rs::send_otp(phone)` + `verify(code)` — 6-digit SMS code
   - `email_link_handler.rs::send_link(email)` + `verify(token)` — magic-link

4. **MUST** map to eIDAS assurance level per DEC-1743:
   - webauthn → 'high'
   - vneid → 'substantial'
   - sms_otp / email_link → 'low'

5. **MUST** enforce minimum level per document per DEC-1743 — if document requires 'substantial', sms_otp rejected.

6. **MUST** define audit table at migration `0005`:
   ```sql
   CREATE TABLE doc_identity_verifications (
     verification_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     signer_id UUID NOT NULL,
     method TEXT NOT NULL CHECK (method IN ('webauthn','vneid','sms_otp','email_link')),
     result TEXT NOT NULL CHECK (result IN ('verified','failed_invalid','failed_expired','failed_no_match')),
     assurance_level TEXT NOT NULL CHECK (assurance_level IN ('low','substantial','high')),
     challenge_id TEXT,
     verified_at TIMESTAMPTZ,
     failure_reason TEXT,
     ip_address TEXT,  -- hashed per FR-BRAIN-111
     user_agent TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX verifications_doc_signer_idx ON doc_identity_verifications(tenant_id, document_id, signer_id);
   ALTER TABLE doc_identity_verifications ENABLE ROW LEVEL SECURITY;
   CREATE POLICY verif_rls ON doc_identity_verifications
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_identity_verifications FROM cyberos_app;
   -- Audit immutable per DEC-1744
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/doc/documents/{id}/verify/start       body: {method, signer_id}
   POST   /v1/doc/documents/{id}/verify/complete    body: {challenge_id, response_data}
   GET    /v1/doc/documents/{id}/verifications      (signer-scoped list)
   ```

8. **MUST** emit 4 BRAIN audit kinds per DEC-1745. PII per FR-BRAIN-111: ip_address SHA-256 hashed; signer_id (uuid) ok.

9. **MUST** thread trace_id from start → complete → audit.

10. **MUST** enforce challenge expiry: webauthn 5min, otp 10min, email_link 24h.

11. **MUST NOT** allow lower-than-required assurance level per DEC-1743.

12. **MUST NOT** mutate verification audit per DEC-1744.

---

## §2 — Why this design

**Why 4 methods (DEC-1740)?** Covers global (webauthn/email), VN (vneid), and fallback (sms_otp); each tenant picks per-document.

**Why eIDAS levels (DEC-1743)?** EU contracts need 'substantial' or 'high' for legal validity; mapping enables enforcement.

**Why immutable audit (DEC-1744)?** Court evidence: years later, must prove who verified, when, how.

**Why per-method handler files (DEC-1740)?** Each has distinct protocols (FIDO2 vs OAuth vs SMS vs magic-link); separation enables independent testing.

---

## §3 — API contract

```text
POST   /v1/doc/documents/{id}/verify/start
POST   /v1/doc/documents/{id}/verify/complete
GET    /v1/doc/documents/{id}/verifications
```

Sample start:
```json
{
  "method": "webauthn",
  "signer_id": "uuid"
}
```

Response:
```json
{
  "challenge_id": "uuid",
  "challenge_data": "base64-challenge-bytes",
  "expires_at": "2026-05-17T10:05:00Z"
}
```

---

## §4 — Acceptance criteria
1. **4 methods enum + cardinality test**. 2. **4 results enum + cardinality test**. 3. **Assurance levels mapped correctly**. 4. **Min level enforced per document**. 5. **WebAuthn FIDO2 flow works**. 6. **VNeID OAuth callback works**. 7. **SMS OTP 6-digit format**. 8. **Email magic-link token**. 9. **Challenge expiry enforced (5/10/24min/h)**. 10. **4 BRAIN audit kinds emitted**. 11. **PII scrubbed (IP SHA256)**. 12. **RLS denies cross-tenant**. 13. **Audit immutable (no UPDATE/DELETE grant)**. 14. **Trace_id preserved**. 15. **Multiple verifications per signer allowed (retry on fail)**. 16. **Multiple signers per doc**. 17. **Verification status visible to AM**. 18. **Failed reasons categorized (invalid/expired/no_match)**. 19. **Challenge_id one-time use**. 20. **eIDAS level downgrade attempt rejected**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn webauthn_high_assurance() {
    let ctx = TestContext::with_webauthn_credential().await;
    let r = ctx.verify_webauthn(ctx.doc_id, ctx.signer_id).await;
    assert_eq!(r.result, "verified");
    assert_eq!(r.assurance_level, "high");
}

#[tokio::test]
async fn sms_otp_low_level() {
    let ctx = TestContext::with_phone().await;
    let start = ctx.start_verify(ctx.doc_id, "sms_otp").await;
    let r = ctx.complete_verify(start.challenge_id, "123456").await;
    assert_eq!(r.assurance_level, "low");
}

#[tokio::test]
async fn min_level_enforced() {
    let ctx = TestContext::doc_requires_substantial().await;
    let r = ctx.try_verify_method(ctx.doc_id, "sms_otp").await;
    assert!(r.is_err());  // sms_otp is 'low', doc requires 'substantial'
}

#[tokio::test]
async fn challenge_expiry() {
    let ctx = TestContext::otp_started().await;
    ctx.advance_time(Duration::minutes(11)).await;
    let r = ctx.complete_verify(ctx.challenge_id, "123456").await;
    assert_eq!(r.result, "failed_expired");
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-AUTH-105.
**Downstream:** FR-DOC-005 (multi-party signing uses this).
**Cross-module:** FR-BRAIN-111 (PII), FR-DOC-001 (document RLS context).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Challenge expired | timestamp check | failed_expired; new challenge | retry |
| OTP code wrong | compare | failed_invalid | retry within window |
| WebAuthn signature invalid | verify | failed_invalid | retry |
| VNeID OAuth denied | callback err | failed_no_match | retry |
| Email link reuse | one-time-use flag | failed_invalid | new link |
| SMS provider down | retry | sev-2; fall back to email | inherent |
| Phone number changed | challenge to current | inherent | data update |
| WebAuthn no credential | enroll first | sev-2; pick other method | enroll |
| eIDAS level downgrade attempt | reject | 403 | use higher method |
| Cross-tenant verification | RLS | 404 | inherent |

## §11 — Implementation notes
- §11.1 WebAuthn via `webauthn-rs` crate; RPID = tenant domain.
- §11.2 VNeID: integrate via official VN gov OAuth (placeholder until contracted).
- §11.3 OTP: 6-digit; Twilio for global, VN provider (Esms.vn) for VN tenants.
- §11.4 Email-link: signed token; 24h expiry; one-time-use.
- §11.5 BRAIN audit body: doc_id, signer_id, method, result, assurance_level; IP SHA256.

---

*End of FR-DOC-006 spec.*
