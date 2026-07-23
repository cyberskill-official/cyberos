---
id: TASK-AUTH-103
title: "AUTH SAML 2.0 SSO — SP-initiated flow + per-tenant IdP config + XML signature verification + assertion validation + JIT provisioning + attribute → role mapping + replay defense"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: auth
priority: p0
status: done
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-AUTH-002, TASK-AUTH-004, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-AUTH-104, TASK-PORTAL-003]
depends_on: [TASK-AUTH-004]
blocks: [TASK-PORTAL-003]

source_pages:
  - website/docs/modules/auth.html#saml-sso
  - https://docs.oasis-open.org/security/saml/v2.0/ (SAML 2.0 Core spec)
  - https://docs.oasis-open.org/security/saml/v2.0/saml-bindings-2.0-os.pdf (Bindings spec)
source_decisions:
  - DEC-520 (SAML 2.0 SP-initiated flow only at slice 1 — IdP-initiated unsolicited responses rejected per OASIS guidance; IdP-initiated is task-AUTH-2xx and requires per-tenant explicit opt-in)
  - DEC-521 (HTTP POST binding for SAMLResponse; HTTP Redirect binding for SAMLRequest — standard SP profile; HTTP Artifact deferred to task-AUTH-2xx)
  - DEC-522 (assertion + response signature BOTH verified — `WantAssertionsSigned=true` + `AuthnRequestsSigned=true`; signature-only-on-response (Microsoft default) rejected with `assertion_signature_required`)
  - "DEC-523 (SAML metadata fetched from IdP per RFC 7517 equivalent path: `metadata_url` configured; cached 24h with kid-style certificate rotation overlap)"
  - DEC-524 (NameIDFormat enforced — only `urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress` and `urn:oasis:names:tc:SAML:2.0:nameid-format:persistent` accepted at slice 1; transient + unspecified rejected)
  - DEC-525 (replay defense — `InResponseTo` must match a known issued AuthnRequest's ID; reuse → 401 + sev-2 audit; AuthnRequest TTL 10 minutes)
  - DEC-526 (assertion `NotBefore` + `NotOnOrAfter` validation with 60s clock skew tolerance per OASIS recommendation; SubjectConfirmation `Recipient` MUST match SP ACS URL)
  - DEC-527 (per-tenant IdP config stored in `auth_saml_idp_configs` with X.509 cert in PEM; private SP signing key KMS-encrypted)
  - DEC-528 (JIT subject provisioning via TASK-AUTH-002 internal helper; attribute-statement → role mapping per per-tenant YAML same shape as TASK-AUTH-104 OIDC claim mapping)
  - DEC-529 (memory audit kinds: auth.saml_login_succeeded, auth.saml_login_failed, auth.saml_jit_provisioned, auth.saml_metadata_refreshed, auth.saml_idp_config_changed, auth.saml_signature_invalid, auth.saml_assertion_replay_attempted)
  - DEC-530 (REVOKE UPDATE, DELETE on auth_saml_login_history from cyberos_app — append-only at SQL grant)
  - DEC-531 (XML canonicalisation — Exclusive Canonicalization (exc-c14n) per W3C; transforms enveloped + exc-c14n only; other transforms rejected per XSW (XML Signature Wrapping) attack defense)
  - DEC-532 (XML signature algorithm RSA-SHA256 minimum; SHA1 rejected per NIST guidance + recent collision research; ECDSA-SHA256/384 accepted)
  - DEC-533 (assertion encryption optional at slice 1 — `WantAssertionsEncrypted=false`; deferred to task-AUTH-2xx for high-sec tenants)
  - DEC-534 (per-tenant max 2 active SAML IdP configs — tenants typically have one corporate IdP + one fallback; ADR required to raise)
  - DEC-535 (single logout (SLO) deferred to task-AUTH-2xx — slice 1 ships login only; tenant-side session termination is handled by token expiry)
  - DEC-536 (Conditions/AudienceRestriction MUST match SP entity_id; mismatch → 401 audience_mismatch)
  - OASIS SAML 2.0 Core + Bindings + Profiles (Mar 2005)
  - W3C XML Signature Syntax and Processing (2008)
  - NIST SP 800-63B (federated identity assurance)
  - PDPL Art. 13 + GDPR Art. 5 (data minimisation — SAML attributes scrubbed in memory chain)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0021_saml_idp_configs.sql
  - services/auth/migrations/0022_saml_login_history.sql
  - services/auth/migrations/0023_saml_authn_request_log.sql
  - services/auth/migrations/0024_saml_subject_link.sql
  - services/auth/src/saml/mod.rs
  # IdP metadata fetch + cache + cert rotation
  - services/auth/src/saml/metadata.rs
  # SP-initiated request builder + ID + TTL
  - services/auth/src/saml/authn_request.rs
  # signature + assertion validation
  - services/auth/src/saml/response_verifier.rs
  # exc-c14n + transforms + XSW defense
  - services/auth/src/saml/xml_signature.rs
  # AttributeStatement → role mapping (reuses TASK-AUTH-104 YAML shape)
  - services/auth/src/saml/attribute_mapper.rs
  # first-login JIT subject creation
  - services/auth/src/saml/jit_provision.rs
  # CRUD across 4 SAML tables
  - services/auth/src/saml/repo.rs
  # 7 memory row builders
  - services/auth/src/saml/audit.rs
  # GET /v1/auth/saml/initiate + POST /v1/auth/saml/acs + POST /v1/auth/saml/idp-configs
  - services/auth/src/handlers/saml.rs
  - services/auth/tests/rbac_adr_gate_test.rs
  - services/auth/tests/saml_sp_initiated_flow_test.rs
  - services/auth/tests/saml_idp_initiated_rejected_test.rs
  - services/auth/tests/admin_cursor_pagination_test.rs
  - services/auth/tests/saml_assertion_signature_required_test.rs
  - services/auth/tests/saml_xsw_attack_defense_test.rs
  - services/auth/tests/admin_revoke_test.rs
  - services/auth/tests/saml_audience_mismatch_test.rs
  - services/auth/tests/saml_clock_skew_tolerance_test.rs
  - services/auth/tests/saml_nameid_format_closed_test.rs
  - services/auth/tests/saml_attribute_role_mapping_test.rs
  - services/auth/tests/admin_list_test.rs
  - services/auth/tests/admin_subject_create_test.rs
  - services/auth/tests/middleware_test.rs
  - services/auth/tests/admin_deny_list_test.rs
  - services/auth/tests/rls_isolation_test.rs
modified_files:
  # pub mod saml
  - services/auth/src/lib.rs

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test saml

disallowed_tools:
  - accept IdP-initiated unsolicited responses (per DEC-520)
  - accept assertion without signature (per DEC-522)
  - allow SHA-1 in XML signatures (per DEC-532)
  - allow non-exc-c14n canonicalization (per DEC-531)
  - skip InResponseTo validation (per DEC-525)
  - allow > 2 active SAML IdP configs per tenant (per DEC-534)
  - allow nameid format transient or unspecified (per DEC-524)
  - skip AudienceRestriction check (per DEC-536)

effort_hours: 12
subtasks:
  - "0.5h: 0021_saml_idp_configs.sql + RLS"
  - "0.3h: 0022_saml_login_history.sql append-only"
  - "0.3h: 0023_saml_authn_request_log.sql append-only + TTL"
  - "0.3h: 0024_saml_subject_link.sql"
  - "0.8h: metadata.rs — fetch + 24h cache + cert rotation"
  - "0.8h: authn_request.rs — SP-initiated request builder with ID generation"
  - "2.0h: response_verifier.rs — full SAML response + assertion validation"
  - "1.2h: xml_signature.rs — exc-c14n + transforms + XSW defense via samael/saml-rs crate"
  - "0.6h: attribute_mapper.rs — reuse TASK-AUTH-104 YAML mapping shape"
  - "0.7h: jit_provision.rs"
  - "0.5h: repo.rs"
  - "0.5h: audit.rs — 7 memory builders"
  - "0.8h: handlers/saml.rs — initiate + acs + idp_config CRUD"
  - "2.7h: tests — 16 test files including XSW attack defense + signature variants"

risk_if_skipped: "SAML 2.0 is the dominant enterprise SSO protocol — without it, TASK-PORTAL-003's external IdP integration cannot serve enterprise tenants. Many corporate IdPs (Okta, Azure AD legacy, ADFS, OneLogin) ship SAML-first; OIDC (TASK-AUTH-104) covers modern apps but enterprise IT teams configure SAML by default. Without DEC-522's assertion-signature-required, the classic 'unsigned assertion injection' attack succeeds (XSW). Without DEC-531's exc-c14n + restricted transforms, XML Signature Wrapping attacks succeed (real attack — see Somorovsky et al. CCS 2012). Without DEC-525's InResponseTo validation, SAML assertion replay succeeds. Without DEC-532's SHA-1 rejection, recent SHA-1 collision research means forged signatures are feasible. Without DEC-520's IdP-initiated rejection, an attacker hosting their own IdP can mint assertions for arbitrary subjects. The 12h effort lands the standards-compliant enterprise SSO with all known SAML-attack defenses."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship SAML 2.0 SP-initiated SSO with per-tenant IdP config + XML signature verification + assertion validation + JIT provisioning + attribute mapping + replay defense. Each requirement:

1. **MUST** define `auth_saml_idp_configs` table: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, name TEXT NOT NULL, entity_id_idp TEXT NOT NULL, sso_url TEXT NOT NULL, metadata_url TEXT, x509_cert_pem TEXT NOT NULL, sp_signing_key_kms_blob BYTEA NOT NULL, sp_signing_kms_key_id TEXT NOT NULL, sp_entity_id TEXT NOT NULL, acs_url TEXT NOT NULL, attribute_mapping_yaml TEXT NOT NULL, is_active BOOLEAN NOT NULL DEFAULT true, created_at TIMESTAMPTZ, created_by_subject_id UUID NOT NULL)`. UNIQUE `(tenant_id, name)`; partial unique `(tenant_id, entity_id_idp) WHERE is_active=true`.

2. **MUST** define `auth_saml_login_history` table: `(id BIGSERIAL, tenant_id UUID, idp_id UUID, subject_id UUID, nameid TEXT NOT NULL, nameid_format TEXT NOT NULL, outcome TEXT NOT NULL CHECK (outcome IN ('succeeded','failed','jit_provisioned')), failure_reason TEXT, source_ip_hash16 TEXT, ts TIMESTAMPTZ)`. `REVOKE UPDATE, DELETE FROM cyberos_app`.

3. **MUST** define `auth_saml_authn_request_log` table for replay defense: `(id UUID PRIMARY KEY, tenant_id UUID, idp_id UUID, issued_at TIMESTAMPTZ, expires_at TIMESTAMPTZ NOT NULL, consumed BOOLEAN NOT NULL DEFAULT false, consumed_at TIMESTAMPTZ)`. UNIQUE on `id`; TTL 10 minutes.

4. **MUST** define `auth_saml_subject_link` table: `(idp_id UUID, nameid TEXT, tenant_id UUID, subject_id UUID NOT NULL REFERENCES auth.subjects(id), linked_at TIMESTAMPTZ, PRIMARY KEY (idp_id, nameid))`. Per-(idp_id, nameid) uniqueness (same shape as TASK-AUTH-104 OIDC).

5. **MUST** enforce RLS with `USING + WITH CHECK` on all 4 tables; root-admin escape clause.

6. **MUST** implement SP-initiated flow only (per DEC-520):
- `GET /v1/auth/saml/initiate?idp_id=<>` — builds SAMLRequest XML, signs with SP signing key, base64-encodes, redirects to IdP's sso_url via HTTP Redirect binding.
- `POST /v1/auth/saml/acs` — receives SAMLResponse via HTTP POST binding; parses + verifies + provisions/links subject; redirects to tenant SPA on success. IdP-initiated unsolicited responses (POST to ACS without prior AuthnRequest) → 401 `unsolicited_response_rejected` + sev-2 audit per DEC-520.

7. **MUST** fetch IdP metadata at config save time (per DEC-523). Parse `EntityDescriptor` → extract `SingleSignOnService` URL + `X509Certificate`. Cache 24h with kid-style overlap on cert rotation (new cert accepted immediately; old cert accepted for 24h after observed rotation).

8. **MUST** require BOTH `WantAssertionsSigned=true` AND `AuthnRequestsSigned=true` (per DEC-522). Response-only-signature (Microsoft Azure AD default config) → reject with `assertion_signature_required` + emit `auth.saml_signature_invalid` memory row.

9. **MUST** enforce closed NameIDFormat (per DEC-524). Accepted formats: `urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress` + `urn:oasis:names:tc:SAML:2.0:nameid-format:persistent`. Others (transient, unspecified, x509SubjectName, etc.) → 401 `nameid_format_unsupported`.

10. **MUST** implement replay defense via InResponseTo (per DEC-525):
- AuthnRequest's ID stored in `auth_saml_authn_request_log` with 10-min TTL.
- SAMLResponse's `InResponseTo` attribute MUST match a known unconsumed row.
- Missing/unknown/expired/consumed → 401 `saml_replay_or_expired` + emit `auth.saml_assertion_replay_attempted` (sev-2) memory row.
- On success, mark consumed.

11. **MUST** validate assertion conditions (per DEC-526):
- `NotBefore` ≤ now ≤ `NotOnOrAfter` with ±60s clock skew tolerance.
- `SubjectConfirmation/SubjectConfirmationData.Recipient` MUST exactly match SP ACS URL.
- `SubjectConfirmation/SubjectConfirmationData.InResponseTo` MUST match outer InResponseTo.
- `Conditions/AudienceRestriction/Audience` MUST contain SP entity_id (per DEC-536). Any fail → 401 + `auth.saml_login_failed` + reason field.

12. **MUST** verify XML signature with restricted transforms (per DEC-531). Allowed transforms:
- `http://www.w3.org/2001/10/xml-exc-c14n#` (Exclusive Canonicalization).
- `http://www.w3.org/2000/09/xmldsig#enveloped-signature`. Any other transform (especially `xpath-2.0`, `xslt`) → reject with `unsupported_transform` per XSW attack defense.

13. **MUST** require RSA-SHA256 minimum signature algorithm (per DEC-532). `SignatureMethod` must be one of:
- `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256` ✓
- `http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha256` ✓
- `http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha384` ✓ SHA-1 (`http://www.w3.org/2000/09/xmldsig#rsa-sha1`) → 401 `weak_signature_algorithm`.

14. **MUST** PII-scrub `nameid`, `failure_reason`, and the `AttributeStatement` value contents via TASK-MEMORY-111 before chain commit.

15. **MUST** JIT-provision subject on first login (per DEC-528). Same shape as TASK-AUTH-104:
- Lookup `auth_saml_subject_link WHERE idp_id=$1 AND nameid=$2` — found → use linked subject_id.
- Not found → call TASK-AUTH-002 internal helper with `email` from nameid (if emailAddress format) or AttributeStatement's `email` claim; assign role per attribute_mapping_yaml; INSERT link row; emit `auth.saml_jit_provisioned` memory row.

16. **MUST** apply per-tenant attribute → role mapping (per DEC-528). Reuses TASK-AUTH-104 YAML shape but with SAML AttributeStatement claim names:
   ```yaml
   default_role: tenant-member
   claim_rules:
     - attribute: "http://schemas.xmlsoap.org/claims/Group"   # AD-style
       contains: "Domain Engineers"
       grant_role: tenant-admin
     - attribute: "groups"   # generic
       contains: "Finance"
       grant_role: cfo
   ```
Validation: every `grant_role` MUST parse to TASK-AUTH-101 closed Role enum at config save time.

17. **MUST** enforce max 2 active SAML IdP configs per tenant (per DEC-534). 3rd → 409 `idp_config_limit_exceeded`. ADR required.

18. **MUST** ship `POST /v1/auth/saml/idp-configs` handler for tenant-admin SAML IdP CRUD. Caller MUST have role `tenant-admin`. Validates:
- metadata_url fetch succeeds.
- x509_cert_pem parses + matches metadata.
- attribute_mapping_yaml parses + all roles valid.
- SP signing key generated (RSA-2048) + KMS-encrypted.
- Returns SP metadata for IdP-side configuration (entity_id, ACS URL, signing certificate).

19. **MUST** emit 7 memory audit row kinds (per DEC-529):
- `auth.saml_login_succeeded` — full flow → AUTH JWT issued.
- `auth.saml_login_failed` — any failure with reason.
- `auth.saml_jit_provisioned` — new subject on first login.
- `auth.saml_metadata_refreshed` — IdP metadata fetch updated cert.
- `auth.saml_idp_config_changed` — POST/PATCH/DELETE on idp_configs.
- `auth.saml_signature_invalid` — sev-2 (may signal attack).
- `auth.saml_assertion_replay_attempted` — sev-2 InResponseTo violation.

20. **MUST** complete ACS callback handler in ≤ 500 ms p95 (XML signature verify is the dominant cost). `saml_perf_test`.

21. **MUST** emit OTel span `auth.saml.{initiate,acs,jit_provision,idp_config_change,metadata_refresh}` with `outcome` attribute (success | jit_provisioned | unsolicited_response | replay_or_expired | signature_invalid | assertion_signature_required | weak_signature_algorithm | unsupported_transform | audience_mismatch | recipient_mismatch | not_before_violated | not_on_or_after_violated | nameid_format_unsupported | unknown_role | metadata_fetch_failed | sub_already_linked).

22. **MUST** emit OTel metrics:
- `auth_saml_login_total{tenant_id, idp_id, outcome}` (counter).
- `auth_saml_jit_provisioned_total{tenant_id}` (counter).
- `auth_saml_metadata_refreshes_total{tenant_id, idp_id}` (counter).
- `auth_saml_signature_failures_total{tenant_id, idp_id}` (counter — sev-2 alarm > 5/h).
- `auth_saml_replay_attempts_total{tenant_id}` (counter — sev-2 alarm > 3/h).
- `auth_saml_flow_latency_ms` (histogram; SLO p95 < 500ms).

23. **MUST** ship the `samael` crate (or equivalent Rust SAML library audited for security) for XML parsing + signature verification + canonicalization. Hand-rolled XML signature is forbidden — too easy to misimplement (see XSW + XML signature attack literature).

24. **MUST** support per-tenant SP entity_id format `https://auth.<tenant_slug>.cyberos.world/saml/sp` and ACS URL format `https://auth.<tenant_slug>.cyberos.world/v1/auth/saml/acs`. Validated at IdP config save time.

25. **MUST** sign every outbound AuthnRequest with the per-tenant SP signing key (per DEC-522 `AuthnRequestsSigned=true`). KMS-decrypt the private key for signing; never expose plaintext.

26. **MUST** include `RequestedAuthnContext` with `AuthnContextClassRef = urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport` at minimum; tenants requiring MFA enforcement at IdP MAY configure `MultiFactor` via per-tenant override.

27. **MUST** support metadata XML download at `GET /v1/auth/saml/idp-configs/{id}/sp-metadata` — returns SP-side metadata XML for IdP operators to configure their end (entity_id, ACS URL, X.509 signing cert).

---

## §2 — Why this design (rationale for humans)

**Why SP-initiated only at slice 1 (DEC-520)?** IdP-initiated unsolicited responses lack the InResponseTo binding to a known SP request — an attacker hosting their own IdP can mint assertions for arbitrary subjects (the "evil IdP" attack). OASIS recommends SP-initiated. Some legitimate use cases (bookmark-from-IdP-portal) need IdP-initiated; deferred to task-AUTH-2xx with per-tenant explicit opt-in + additional defenses.

**Why both AuthnRequestsSigned AND WantAssertionsSigned (DEC-522)?** Signing AuthnRequests proves SP authenticity to IdP (prevents request forgery). Signing assertions proves IdP authenticity to SP (prevents assertion injection). Response-only signing (Azure AD default) is insufficient — the assertion within an unsigned envelope can be swapped via XSW. Both signatures together = full chain validated.

**Why exc-c14n only + restricted transforms (DEC-531)?** XML Signature Wrapping (XSW) attacks exploit canonicalization differences to validate a signature against one element while the application reads a different element. Restricting transforms to `exc-c14n + enveloped-signature` eliminates the attack surface; the alternative (xpath, xslt) lets attackers craft assertions that pass signature verification but contain attacker-controlled content.

**Why RSA-SHA256 minimum (DEC-532)?** SHA-1 collision attacks (SHAttered + Polonen 2020) make forged signatures feasible at nation-state level. NIST has deprecated SHA-1 for digital signatures. SHA-256 minimum is the current baseline; SHA-384 ECDSA accepted for high-security tenants.

**Why InResponseTo validation (DEC-525)?** Without it, an attacker captures a legitimate SAMLResponse + replays. With it, each response binds to a specific unconsumed AuthnRequest ID — replay fails. 10-min TTL covers slow IdP responses without leaving consumed requests usable indefinitely.

**Why ±60s clock skew (DEC-526)?** NTP drift between IdP + SP is typically < 1s but can spike during NTP outages. ±60s tolerates outages without false-rejection; tighter would false-reject legitimate flows.

**Why Audience + Recipient + InResponseTo checks (§1 #11, DEC-536)?** Defense in depth: even if signature verification fails to detect XSW (subtle bug), the Audience + Recipient + InResponseTo checks add second + third + fourth gates. Any missed claim = reject.

**Why closed NameIDFormat (DEC-524)?** Transient nameids are session-scoped (different each login) — useless for our subject_link table. Unspecified is operator-discretion — could be anything. EmailAddress + persistent are the two formats with stable subject identity over time. Closing the set prevents IdP config drift.

**Why per-tenant attribute mapping YAML (DEC-528, §1 #16)?** Different IdPs use different attribute names: Okta uses `groups`; Azure AD uses `http://schemas.xmlsoap.org/claims/Group`; Google uses scope; ADFS uses claim types. Per-tenant YAML lets each tenant configure their IdP-specific mapping without code changes. Reuses TASK-AUTH-104's YAML shape for consistency.

**Why max 2 IdP configs per tenant (DEC-534)?** Enterprise tenants typically have one primary IdP (corporate AD) and at most a fallback (e.g. during IdP migration). > 2 suggests config sprawl; ADR-required to raise.

**Why samael crate not hand-rolled (§1 #23)?** SAML signature verification is famously bug-prone (10+ public CVEs from incorrect parser implementations in 2015-2020 alone). Battle-tested crate is the only safe path. Hand-rolled XML signature is forbidden at slice 1.

**Why SLO deferred (DEC-535)?** Single Logout (SLO) is operationally complex (cascading logout across multiple SPs; partial-failure handling; user-visible session inconsistency). Slice 1 covers login; tenant session expiry handles logout adequately for slice 1. SLO ships in task-AUTH-2xx when enterprise tenants demand it.

**Why KMS-encrypt SP signing key (§1 #25, DEC-527)?** SP signing key proves to IdP that AuthnRequests are from us. Compromise = attacker forges AuthnRequests for any subject. KMS encryption requires KMS-decrypt permission to sign — meaningful additional barrier.

**Why metadata XML download endpoint (§1 #27)?** IdP operators (corporate IT) need our SP metadata (entity_id, ACS URL, signing cert) to configure their side. Standard SAML SP exposes this at a well-known URL. Per-tenant URL makes copy-paste setup easy.

**Why min RequestedAuthnContext = PasswordProtectedTransport (§1 #26)?** Baseline: IdP must authenticate user with at least password + TLS transport. Lower (e.g. `PreviousSession`) = relying on browser session — phishable. Tenants requiring MFA at IdP override via config; our slice 1 ships baseline + MFA-via-TASK-AUTH-102 at SP side.

**Why 7 memory audit kinds (DEC-529)?** Different operator queries: "show me all successful SAML logins this week" → `auth.saml_login_succeeded`. "Show me signature failures" → `auth.saml_signature_invalid`. "Show me replay attempts" → `auth.saml_assertion_replay_attempted`. Selectivity benefits at query time.

**Why metadata 24h cache + cert rotation overlap (§1 #7, DEC-523)?** IdPs rotate signing certs periodically. 24h cache avoids hot-path metadata refetch. Overlap (old cert + new cert both valid) prevents flap during rotation. Mirrors TASK-AUTH-104 JWKS pattern.

**Why per-tenant `entity_id` format (§1 #24)?** Distinct per-tenant entity_id lets IdP-side admins manage per-tenant SP registrations independently. Conflict with another tenant's entity_id is structurally impossible.

**Why append-only login_history at SQL grant (§1 #2, DEC-530)?** Forensic record of all login attempts including failures. UPDATE/DELETE blocked prevents tampering. Failed logins (with reason) help diagnose attacks + misconfigurations.

**Why sev-2 alarm on > 5/h signature failures + > 3/h replay attempts (§1 #22)?** Both indicate either (a) IdP misconfiguration + needs operator help, or (b) active attack. Sev-2 = operator investigates within an hour.

---

## §3 — API contract

### 3.1 — Migration 0021 — idp_configs

```sql
-- services/auth/migrations/0021_saml_idp_configs.sql

BEGIN;

CREATE TABLE auth_saml_idp_configs (
    id                          UUID         PRIMARY KEY,
    tenant_id                   UUID         NOT NULL,
    name                        TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    entity_id_idp               TEXT         NOT NULL,
    sso_url                     TEXT         NOT NULL,
    metadata_url                TEXT,
    x509_cert_pem               TEXT         NOT NULL,
    sp_signing_key_kms_blob     BYTEA        NOT NULL,
    sp_signing_kms_key_id       TEXT         NOT NULL,
    sp_entity_id                TEXT         NOT NULL,
    acs_url                     TEXT         NOT NULL,
    attribute_mapping_yaml      TEXT         NOT NULL,
    is_active                   BOOLEAN      NOT NULL DEFAULT true,
    created_at                  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id       UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX uniq_saml_idp_name ON auth_saml_idp_configs (tenant_id, name);
CREATE UNIQUE INDEX uniq_active_saml_idp_entity ON auth_saml_idp_configs (tenant_id, entity_id_idp) WHERE is_active = true;

ALTER TABLE auth_saml_idp_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY saml_idp_configs_tenant_iso ON auth_saml_idp_configs
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.2 — Migration 0022 — login_history

```sql
-- services/auth/migrations/0022_saml_login_history.sql

BEGIN;

CREATE TABLE auth_saml_login_history (
    id                  BIGSERIAL    PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    idp_id              UUID         NOT NULL REFERENCES auth_saml_idp_configs(id),
    subject_id          UUID         REFERENCES auth.subjects(id),
    nameid              TEXT         NOT NULL,
    nameid_format       TEXT         NOT NULL,
    outcome             TEXT         NOT NULL CHECK (outcome IN ('succeeded','failed','jit_provisioned')),
    failure_reason      TEXT,
    source_ip_hash16    TEXT,
    ts                  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX saml_login_history_tenant_ts_idx ON auth_saml_login_history (tenant_id, ts DESC);
CREATE INDEX saml_login_history_subject_idx ON auth_saml_login_history (subject_id) WHERE subject_id IS NOT NULL;

ALTER TABLE auth_saml_login_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY saml_login_history_tenant_iso ON auth_saml_login_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON auth_saml_login_history FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0023 — authn_request_log (replay defense)

```sql
-- services/auth/migrations/0023_saml_authn_request_log.sql

BEGIN;

CREATE TABLE auth_saml_authn_request_log (
    id              UUID         PRIMARY KEY,
    tenant_id       UUID         NOT NULL,
    idp_id          UUID         NOT NULL REFERENCES auth_saml_idp_configs(id),
    issued_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ  NOT NULL,
    consumed        BOOLEAN      NOT NULL DEFAULT false,
    consumed_at     TIMESTAMPTZ
);

CREATE INDEX saml_authn_request_expires_idx ON auth_saml_authn_request_log (expires_at) WHERE consumed = false;

ALTER TABLE auth_saml_authn_request_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY saml_authn_log_tenant_iso ON auth_saml_authn_request_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE DELETE ON auth_saml_authn_request_log FROM cyberos_app;
GRANT UPDATE (consumed, consumed_at) ON auth_saml_authn_request_log TO cyberos_app;

COMMIT;
```

### 3.4 — Migration 0024 — subject_link

```sql
-- services/auth/migrations/0024_saml_subject_link.sql

BEGIN;

CREATE TABLE auth_saml_subject_link (
    idp_id       UUID         NOT NULL REFERENCES auth_saml_idp_configs(id) ON DELETE RESTRICT,
    nameid       TEXT         NOT NULL,
    tenant_id    UUID         NOT NULL,
    subject_id   UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    linked_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (idp_id, nameid)
);

CREATE INDEX saml_subject_link_subject_idx ON auth_saml_subject_link (tenant_id, subject_id);

ALTER TABLE auth_saml_subject_link ENABLE ROW LEVEL SECURITY;
CREATE POLICY saml_subject_link_tenant_iso ON auth_saml_subject_link
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.5 — AuthnRequest builder

```rust
// services/auth/src/saml/authn_request.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct AuthnRequestParams {
    pub id: Uuid,
    pub issue_instant: DateTime<Utc>,
    pub destination: String,        // IdP SSO URL
    pub sp_entity_id: String,
    pub acs_url: String,
}

pub fn build_xml(p: &AuthnRequestParams) -> String {
    format!(r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
    ID="_{id}" Version="2.0" IssueInstant="{ts}" Destination="{dest}"
    AssertionConsumerServiceURL="{acs}" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>{sp}</saml:Issuer>
  <samlp:NameIDPolicy AllowCreate="true" Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"/>
  <samlp:RequestedAuthnContext Comparison="minimum">
    <saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef>
  </samlp:RequestedAuthnContext>
</samlp:AuthnRequest>"#,
        id = p.id.to_simple(), ts = p.issue_instant.to_rfc3339(),
        dest = p.destination, acs = p.acs_url, sp = p.sp_entity_id,
    )
}
```

### 3.6 — Response verifier (signature + assertion conditions)

```rust
// services/auth/src/saml/response_verifier.rs
use chrono::{DateTime, Duration, Utc};
use samael::{schema::Response, service_provider::ServiceProvider};

const CLOCK_SKEW_SECONDS: i64 = 60;
const ALLOWED_SIG_ALGS: &[&str] = &[
    "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
    "http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha256",
    "http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha384",
];
const ALLOWED_TRANSFORMS: &[&str] = &[
    "http://www.w3.org/2001/10/xml-exc-c14n#",
    "http://www.w3.org/2000/09/xmldsig#enveloped-signature",
];

#[derive(Debug, thiserror::Error)]
pub enum SamlVerifyError {
    #[error("response_signature_invalid")]
    ResponseSignatureInvalid,
    #[error("assertion_signature_required")]
    AssertionSignatureRequired,
    #[error("assertion_signature_invalid")]
    AssertionSignatureInvalid,
    #[error("weak_signature_algorithm: {0}")]
    WeakSignatureAlgorithm(String),
    #[error("unsupported_transform: {0}")]
    UnsupportedTransform(String),
    #[error("audience_mismatch")]
    AudienceMismatch,
    #[error("recipient_mismatch")]
    RecipientMismatch,
    #[error("not_before_violated")]
    NotBeforeViolated,
    #[error("not_on_or_after_violated")]
    NotOnOrAfterViolated,
    #[error("nameid_format_unsupported: {0}")]
    NameidFormatUnsupported(String),
    #[error("in_response_to_invalid")]
    InResponseToInvalid,
    #[error("unsolicited_response_rejected")]
    UnsolicitedResponseRejected,
}

pub fn verify_response(
    response_xml: &str,
    idp_cert_pem: &str,
    sp_entity_id: &str,
    sp_acs_url: &str,
    expected_in_response_to: Option<&str>,
) -> Result<VerifiedAssertion, SamlVerifyError> {
    // Parse + signature verify via samael
    let sp = ServiceProvider::default()
        .with_idp_certificate_pem(idp_cert_pem)
        .with_entity_id(sp_entity_id);
    let parsed: Response = samael::traits::ToXml::from_xml(response_xml)
        .map_err(|_| SamlVerifyError::ResponseSignatureInvalid)?;

    // (1) Unsolicited check (DEC-520)
    let in_response_to = parsed.in_response_to.as_deref().ok_or(SamlVerifyError::UnsolicitedResponseRejected)?;
    let expected = expected_in_response_to.ok_or(SamlVerifyError::UnsolicitedResponseRejected)?;
    if in_response_to != expected {
        return Err(SamlVerifyError::InResponseToInvalid);
    }

    // (2) Signature algorithm whitelist (DEC-532)
    let sig_alg = parsed.signature.as_ref()
        .map(|s| s.signed_info.signature_method.algorithm.as_str())
        .unwrap_or("");
    if !ALLOWED_SIG_ALGS.iter().any(|a| *a == sig_alg) {
        return Err(SamlVerifyError::WeakSignatureAlgorithm(sig_alg.to_string()));
    }

    // (3) Transforms whitelist (DEC-531 — XSW defense)
    for tr in parsed.signature.as_ref().map(|s| s.signed_info.references.iter()
        .flat_map(|r| r.transforms.iter()
            .flat_map(|t| t.transform.iter()
                .map(|tr| tr.algorithm.as_str())))) {
        for alg in tr {
            if !ALLOWED_TRANSFORMS.iter().any(|a| *a == alg) {
                return Err(SamlVerifyError::UnsupportedTransform(alg.to_string()));
            }
        }
    }

    // (4) Assertion signature required (DEC-522)
    let assertion = parsed.assertions.into_iter().next()
        .ok_or(SamlVerifyError::AssertionSignatureInvalid)?;
    if assertion.signature.is_none() {
        return Err(SamlVerifyError::AssertionSignatureRequired);
    }

    // (5) Audience check (DEC-536)
    let audiences: Vec<String> = assertion.conditions.as_ref()
        .into_iter()
        .flat_map(|c| c.audience_restrictions.iter())
        .flat_map(|ar| ar.audiences.iter().map(|a| a.audience.clone()))
        .collect();
    if !audiences.contains(&sp_entity_id.to_string()) {
        return Err(SamlVerifyError::AudienceMismatch);
    }

    // (6) Recipient check (per DEC-526)
    let recipient_match = assertion.subject.as_ref()
        .and_then(|s| s.subject_confirmations.iter().next())
        .and_then(|sc| sc.subject_confirmation_data.as_ref())
        .map(|d| d.recipient.as_deref() == Some(sp_acs_url))
        .unwrap_or(false);
    if !recipient_match {
        return Err(SamlVerifyError::RecipientMismatch);
    }

    // (7) NotBefore + NotOnOrAfter (60s skew)
    let now = Utc::now();
    if let Some(conds) = assertion.conditions.as_ref() {
        if let Some(nb) = conds.not_before {
            if now + Duration::seconds(CLOCK_SKEW_SECONDS) < nb {
                return Err(SamlVerifyError::NotBeforeViolated);
            }
        }
        if let Some(noa) = conds.not_on_or_after {
            if now - Duration::seconds(CLOCK_SKEW_SECONDS) >= noa {
                return Err(SamlVerifyError::NotOnOrAfterViolated);
            }
        }
    }

    // (8) NameIDFormat closed set
    let (nameid, nameid_format) = assertion.subject.as_ref()
        .and_then(|s| s.name_id.as_ref())
        .map(|n| (n.value.clone(), n.format.clone().unwrap_or_default()))
        .ok_or(SamlVerifyError::NameidFormatUnsupported("missing".into()))?;
    const ALLOWED_FORMATS: &[&str] = &[
        "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
        "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
    ];
    if !ALLOWED_FORMATS.iter().any(|f| *f == nameid_format) {
        return Err(SamlVerifyError::NameidFormatUnsupported(nameid_format));
    }

    Ok(VerifiedAssertion {
        nameid, nameid_format,
        attributes: extract_attributes(&assertion),
    })
}

pub struct VerifiedAssertion {
    pub nameid: String,
    pub nameid_format: String,
    pub attributes: std::collections::HashMap<String, Vec<String>>,
}

fn extract_attributes(assertion: &samael::schema::Assertion) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    for stmt in &assertion.attribute_statements {
        for attr in &stmt.attributes {
            let name = attr.name.clone();
            let values: Vec<String> = attr.values.iter().filter_map(|v| v.value.clone()).collect();
            map.insert(name, values);
        }
    }
    map
}
```

### 3.7 — ACS handler

```rust
// services/auth/src/handlers/saml.rs (excerpt)
use axum::{Form, extract::State, http::StatusCode, response::Redirect};
use crate::saml::{response_verifier, attribute_mapper, jit_provision, audit};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AcsForm {
    #[serde(rename = "SAMLResponse")]
    pub saml_response_b64: String,
    #[serde(rename = "RelayState")]
    pub relay_state: Option<String>,
}

pub async fn acs_callback(
    State(state): State<AppState>,
    Form(form): Form<AcsForm>,
) -> Result<Redirect, ApiError> {
    // (1) Base64-decode SAMLResponse
    let response_xml = String::from_utf8(
        base64::decode(&form.saml_response_b64).map_err(|_| ApiError::MalformedSamlResponse)?
    ).map_err(|_| ApiError::MalformedSamlResponse)?;

    // (2) Parse InResponseTo to find IdP + AuthnRequest record
    let in_response_to = quick_parse_in_response_to(&response_xml)?;
    let authn_log = state.repo.find_authn_request(in_response_to).await?
        .ok_or(ApiError::SamlUnsolicited)?;
    if authn_log.consumed || authn_log.expires_at < chrono::Utc::now() {
        audit::emit_assertion_replay_attempted(authn_log.tenant_id, in_response_to).await;
        return Err(ApiError::SamlReplayOrExpired);
    }

    // (3) Load IdP config
    let idp = state.repo.load_saml_idp_config(authn_log.idp_id).await?;

    // (4) Verify
    let assertion = match response_verifier::verify_response(
        &response_xml, &idp.x509_cert_pem, &idp.sp_entity_id, &idp.acs_url,
        Some(&in_response_to.to_string()),
    ) {
        Ok(a) => a,
        Err(e) => {
            audit::emit_login_failed(authn_log.tenant_id, authn_log.idp_id, &format!("{e:?}")).await;
            if matches!(e, response_verifier::SamlVerifyError::AssertionSignatureRequired
                       | response_verifier::SamlVerifyError::ResponseSignatureInvalid
                       | response_verifier::SamlVerifyError::AssertionSignatureInvalid
                       | response_verifier::SamlVerifyError::WeakSignatureAlgorithm(_)
                       | response_verifier::SamlVerifyError::UnsupportedTransform(_)) {
                audit::emit_signature_invalid(authn_log.tenant_id, authn_log.idp_id, &format!("{e:?}")).await;
            }
            return Err(ApiError::from(e));
        }
    };

    // (5) JIT provisioning + role mapping
    let role = attribute_mapper::resolve_role(&idp.attribute_mapping_yaml, &assertion.attributes)?;
    let (subject_id, was_jit) = jit_provision::provision_or_link(
        &state, authn_log.tenant_id, authn_log.idp_id, &assertion, role,
    ).await?;

    // (6) Mark AuthnRequest consumed
    state.repo.mark_authn_request_consumed(authn_log.id).await?;

    // (7) Issue AUTH JWT
    let token = state.jwt_issuer.issue_for_subject(subject_id, authn_log.tenant_id).await?;

    // (8) Audit + redirect
    audit::emit_login_succeeded(authn_log.tenant_id, authn_log.idp_id, subject_id, was_jit).await;
    let redirect_url = form.relay_state.unwrap_or_else(|| state.config.default_post_login_url.clone());
    Ok(Redirect::to(&format!("{redirect_url}?access_token={token}")))
}
```

---

## §4 — Acceptance criteria

1. **SP-initiated flow only** — GET /initiate generates AuthnRequest; POST /acs without prior request → 401 unsolicited_response_rejected.
2. **AuthnRequest signed** — outbound request XML carries signature.
3. **WantAssertionsSigned required** — response with unsigned assertion → 401 assertion_signature_required.
4. **Response signature verified** — invalid sig → 401.
5. **SHA-1 algorithm rejected** → 401 weak_signature_algorithm.
6. **RSA-SHA256 accepted** — happy path.
7. **ECDSA-SHA256 accepted** — alternate alg.
8. **Unsupported transform rejected** (XSW defense) — xpath transform → 401 unsupported_transform.
9. **InResponseTo replay rejected** — second use of same ID → 401 + sev-2 audit.
10. **InResponseTo expired (> 10min)** → 401.
11. **Unknown InResponseTo** → 401 unsolicited_response.
12. **Audience mismatch** → 401.
13. **Recipient mismatch** → 401.
14. **NotBefore in future > 60s** → 401.
15. **NotOnOrAfter past > 60s** → 401.
16. **Clock skew tolerated within 60s** — happy path.
17. **NameIDFormat emailAddress accepted** — happy path.
18. **NameIDFormat persistent accepted** — happy path.
19. **NameIDFormat transient rejected** → 401.
20. **JIT provisioning on first login** — new subject created; `auth.saml_jit_provisioned` row.
21. **Subject reused on repeat login** — link row hit.
22. **Attribute → role mapping applies** — Group/groups attribute matches → granted role.
23. **Unknown role in attribute_mapping_yaml rejected** at config save.
24. **Max 2 IdP configs per tenant** — 3rd → 409.
25. **SP signing key KMS-encrypted** — DB row carries BYTEA blob.
26. **append-only login_history** — UPDATE/DELETE blocked.
27. **GET sp-metadata returns SP metadata XML** — tenant-admin only.
28. **OTel span emitted** — `auth.saml.acs` with outcome.
29. **Counter `auth_saml_login_total{outcome=succeeded}` increments**.
30. **Counter `auth_saml_signature_failures_total` sev-2 at > 5/h**.
31. **Counter `auth_saml_replay_attempts_total` sev-2 at > 3/h**.
32. **ACS handler p95 < 500ms** — perf test.

---

## §5 — Verification

```rust
// services/auth/tests/saml_xsw_attack_defense_test.rs
#[test]
fn xpath_transform_rejected() {
    let xml_with_xpath_transform = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
        <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo><Reference URI="">
                <Transforms><Transform Algorithm="http://www.w3.org/TR/1999/REC-xpath-19991116"/></Transforms>
                ...
            </Reference></SignedInfo>
        </Signature>
    </samlp:Response>"#;
    let result = cyberos_auth::saml::response_verifier::verify_response(
        xml_with_xpath_transform, mock_cert(), "sp_entity", "https://acs", Some("known_id"),
    );
    assert!(matches!(result, Err(cyberos_auth::saml::response_verifier::SamlVerifyError::UnsupportedTransform(_))));
}
```

```rust
// services/auth/tests/admin_subject_create_test.rs
#[test]
fn sha1_signature_method_rejected() {
    let xml_with_sha1 = mock_response_with_sig_method("http://www.w3.org/2000/09/xmldsig#rsa-sha1");
    let result = cyberos_auth::saml::response_verifier::verify_response(
        &xml_with_sha1, mock_cert(), "sp_entity", "https://acs", Some("known_id"),
    );
    assert!(matches!(result, Err(cyberos_auth::saml::response_verifier::SamlVerifyError::WeakSignatureAlgorithm(_))));
}
```

```rust
// services/auth/tests/saml_assertion_signature_required_test.rs
#[test]
fn response_only_signed_rejected() {
    let xml = mock_response_with_unsigned_assertion();
    let result = cyberos_auth::saml::response_verifier::verify_response(
        &xml, mock_cert(), "sp_entity", "https://acs", Some("id"),
    );
    assert!(matches!(result, Err(cyberos_auth::saml::response_verifier::SamlVerifyError::AssertionSignatureRequired)));
}
```

```rust
// services/auth/tests/admin_revoke_test.rs
#[tokio::test]
async fn reused_in_response_to_rejected(ctx: TestCtx) {
    let req_id = ctx.initiate_flow().await;
    let response = ctx.build_idp_response_for(req_id).await;
    ctx.acs_callback(&response).await.unwrap();   // first use OK
    let err = ctx.acs_callback(&response).await.unwrap_err();
    assert!(format!("{err:?}").contains("SamlReplayOrExpired"));
    let rows = ctx.memory_audit_rows("auth.saml_assertion_replay_attempted").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["severity"], "sev-2");
}
```

```rust
// services/auth/tests/saml_nameid_format_closed_test.rs
#[test]
fn transient_nameid_rejected() {
    let xml = mock_response_with_nameid_format("urn:oasis:names:tc:SAML:2.0:nameid-format:transient");
    let result = cyberos_auth::saml::response_verifier::verify_response(
        &xml, mock_cert(), "sp_entity", "https://acs", Some("id"),
    );
    assert!(matches!(result, Err(cyberos_auth::saml::response_verifier::SamlVerifyError::NameidFormatUnsupported(_))));
}

#[test]
fn email_address_format_accepted() {
    let xml = mock_response_with_nameid_format("urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress");
    let result = cyberos_auth::saml::response_verifier::verify_response(
        &xml, mock_cert(), "sp_entity", "https://acs", Some("id"),
    );
    assert!(result.is_ok());
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; samael crate handles XML signature plumbing; 7 memory row builders follow canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-004** — JWT issuer (post-SAML callback issues our token).

**Downstream (1 placeholder):**
- **TASK-PORTAL-003** — external IdP SAML 2.0 + OIDC support; consumes both TASK-AUTH-103 + TASK-AUTH-104.

**Cross-module:**
- **TASK-AUTH-002** — subject create (JIT helper).
- **TASK-AUTH-101** — RBAC; attribute_mapping_yaml validates against closed Role enum.
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrubbing.
- **TASK-OBS-007** — sev-2 alarm on signature failures + replay attempts.

---

## §8 — Example payloads

### 8.1 — POST /v1/auth/saml/idp-configs

```json
{
  "name": "Acme ADFS",
  "entity_id_idp": "https://adfs.acme.example/adfs/services/trust",
  "sso_url": "https://adfs.acme.example/adfs/ls/",
  "metadata_url": "https://adfs.acme.example/FederationMetadata/2007-06/FederationMetadata.xml",
  "x509_cert_pem": "-----BEGIN CERTIFICATE-----\nMIID...==\n-----END CERTIFICATE-----",
  "attribute_mapping_yaml": "default_role: tenant-member\nclaim_rules:\n  - attribute: \"http://schemas.xmlsoap.org/claims/Group\"\n    contains: \"Domain Engineers\"\n    grant_role: tenant-admin\n  - attribute: \"http://schemas.xmlsoap.org/claims/Group\"\n    contains: \"CFO Office\"\n    grant_role: cfo\n"
}
```

### 8.2 — auth.saml_login_succeeded memory row

```json
{
  "kind": "auth.saml_login_succeeded",
  "tenant_id": "5e8f1d2a-...",
  "idp_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "nameid_hash16": "abc123def4567890",
  "nameid_format": "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
  "granted_role": "chief-financial-officer",
  "was_jit": false,
  "ts_ns": 1747920731000000000
}
```

### 8.3 — auth.saml_signature_invalid memory row (sev-2)

```json
{
  "kind": "auth.saml_signature_invalid",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "idp_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "reason": "WeakSignatureAlgorithm(\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\")",
  "source_ip_hash16": "fed0987654321abc",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — auth.saml_assertion_replay_attempted memory row (sev-2)

```json
{
  "kind": "auth.saml_assertion_replay_attempted",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "in_response_to": "_abc123-def456-...",
  "source_ip_hash16": "fed0987654321abc",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **IdP-initiated unsolicited responses** — task-AUTH-2xx with per-tenant opt-in.
- **Single Logout (SLO)** — task-AUTH-2xx.
- **Assertion encryption (WantAssertionsEncrypted)** — task-AUTH-2xx for high-sec tenants.
- **HTTP Artifact binding** — out of scope; POST + Redirect cover modern enterprise.
- **Dynamic SP metadata cert rotation** — slice 2.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unsolicited response (no InResponseTo) | check | 401 unsolicited_response | Designed |
| InResponseTo replay | consumed/expired check | 401 + sev-2 audit | Re-initiate |
| InResponseTo expired (>10min) | TTL | 401 | Re-initiate |
| Response signature invalid | samael verify | 401 + sev-2 audit | Investigate |
| Assertion signature missing | verifier | 401 + sev-2 | Configure IdP to sign assertions |
| SHA-1 algorithm | algorithm check | 401 weak_signature_algorithm | Upgrade IdP signing config |
| xpath transform in signature | transform check | 401 unsupported_transform | XSW attack defense |
| Audience mismatch | conditions check | 401 | Fix IdP config |
| Recipient mismatch | subject confirmation check | 401 | Fix IdP config |
| NotBefore in future | timestamp check | 401 | Clock sync |
| NotOnOrAfter past | timestamp check | 401 | Re-initiate |
| Clock skew > 60s | window check | 401 | Sync clocks |
| Transient nameid | format check | 401 | Configure IdP for persistent/emailAddress |
| Unknown nameid format | format check | 401 | Configure |
| Unknown role in attribute_mapping_yaml | parse_config | 400 at config save | Fix YAML |
| 3rd IdP config | handler check | 409 | ADR + cap raise |
| KMS decrypt fail (SP signing key) | aws-sdk error | 500 + sev-1 | Rotate key |
| Metadata fetch fail | reqwest error | 500 + sev-3 | IdP health check |
| Cross-tenant subject conflict | UNIQUE (idp_id, nameid) | 409 | Designed |
| JIT subject create fail | TASK-AUTH-002 error | 500 + audit | Investigate |
| append-only log UPDATE from app | SQL grant | permission denied | Designed |
| RLS bypass | USING | 0 rows | Designed |
| OTel span attribute missing | otel_test | CI fails | Fix |
| memory audit emit fails | tx rollback | 500 | memory_writer health |
| Malformed XML | parse error | 400 | Designed |
| Cert rotation: old + new cache overlap | 24h window | Designed | None |
| Signature verifies but XSW substituted element | samael's hardened verifier | None (defense in depth via Audience + Recipient + transform restrictions) | None |
| > 5/h signature failures sustained | OBS rule | sev-2 | Investigate |
| > 3/h replay attempts | OBS rule | sev-2 | Investigate |
| Disabled IdP login attempt | is_active check | 401 | Re-enable |
| client_secret leaked in API | handler omits | None | Designed |
| Concurrent ACS callbacks for same InResponseTo | row lock + consumed | One wins | Designed |

---

## §11 — Implementation notes

- **SP-initiated only** — IdP-initiated requires per-tenant opt-in (task-AUTH-2xx).
- **HTTP POST + Redirect bindings** — modern enterprise standard.
- **WantAssertionsSigned + AuthnRequestsSigned both required** — XSW defense.
- **exc-c14n + enveloped-signature only** — XSW transform whitelist.
- **RSA-SHA256 minimum** — SHA-1 deprecated; ECDSA accepted.
- **InResponseTo replay defense** — 10-min TTL; consumed flag.
- **±60s clock skew** — NTP-drift tolerance.
- **Closed NameIDFormat (email/persistent)** — stable subject identity.
- **Per-tenant attribute_mapping_yaml** — same shape as TASK-AUTH-104.
- **Max 2 IdP configs per tenant** — typical enterprise count.
- **samael crate** — battle-tested SAML library; hand-rolled forbidden.
- **Per-tenant SP entity_id + ACS URL** — IdP-side admin per-tenant.
- **SP signing key KMS-encrypted** — secret-at-rest hardening.
- **24h metadata cache + rotation overlap** — matches TASK-AUTH-104 JWKS pattern.
- **PII scrub nameid + attributes** — chain holds hashed.
- **7 memory audit kinds** — selective queries.
- **sev-2 alarms on signature failure + replay** — attack signal.
- **SP metadata download endpoint** — IdP operator setup.
- **RequestedAuthnContext PasswordProtectedTransport** — baseline assurance.
- **Append-only via SQL grant** — forensic.
- **SLO deferred** — token expiry suffices at slice 1.
- **Defense in depth** — signature + Audience + Recipient + InResponseTo + transform whitelist all required.

---

*End of TASK-AUTH-103.*
