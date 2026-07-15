---
id: TASK-PORTAL-003
title: "PORTAL external IdP — SAML 2.0 + OIDC sign-in for client-tenant users + SCIM 2.0 JIT provisioning + per-Engagement IdP binding + claim → role mapping + signed-attr trust chain"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PORTAL
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-103, TASK-AUTH-104, TASK-AUTH-002, TASK-AUTH-004, TASK-AUTH-101, TASK-PORTAL-001, TASK-PORTAL-002, TASK-PORTAL-004, TASK-PORTAL-005, TASK-TEN-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-AUTH-103, TASK-AUTH-104]
blocks: [TASK-PORTAL-004, TASK-PORTAL-005]

source_pages:
  - website/docs/modules/portal.html#external-idp
  - website/docs/modules/portal.html#scim
  - https://datatracker.ietf.org/doc/html/rfc7522       # SAML JWT bearer
  - https://datatracker.ietf.org/doc/html/rfc7643       # SCIM 2.0 core schema
  - https://datatracker.ietf.org/doc/html/rfc7644       # SCIM 2.0 protocol
  - https://datatracker.ietf.org/doc/html/rfc8414       # OAuth/OIDC server metadata
  - https://docs.oasis-open.org/security/saml/v2.0/saml-core-2.0-os.pdf

source_decisions:
  - DEC-860 2026-05-17 — Per-Engagement IdP binding (not per-tenant) — a single tenant may host 50 client Engagements, each with their own IdP; binding is `(tenant_id, engagement_id) → idp_config`
  - DEC-861 2026-05-17 — SAML 2.0 + OIDC reuse TASK-AUTH-103 + TASK-AUTH-104 protocol primitives; PORTAL-003 adds the per-Engagement layer + SCIM
  - DEC-862 2026-05-17 — SCIM 2.0 JIT user provisioning at first SSO sign-in; user attributes populated from SAML AttributeStatement or OIDC ID-token claims
  - DEC-863 2026-05-17 — Closed claim → role mapping per Engagement IdP config; default role is `client_viewer`; elevation requires explicit claim match (e.g. `groups contains 'cyberos-admin'`)
  - DEC-864 2026-05-17 — Signed-attribute trust chain — only SAML AttributeStatements signed by the IdP's metadata-published signing cert are honoured; unsigned attributes ignored (defense against attribute injection)
  - DEC-865 2026-05-17 — IdP discovery via email-domain hint at PORTAL sign-in page (e.g. `alice@acme.com → IdP at https://idp.acme.com/saml2`); fallback to "Sign in with email" if no domain match
  - DEC-866 2026-05-17 — IdP config stored in `portal_idp_configs` table, KMS-encrypted for private keys + cert thumbprints; rotation handled by per-Engagement admin
  - DEC-867 2026-05-17 — SCIM endpoint at `/scim/v2/{engagement_slug}/Users` + `/Groups` per RFC 7644; bearer-token auth with per-Engagement SCIM token (rotated quarterly)
  - DEC-868 2026-05-17 — SCIM CREATE → AUTH subject create + Engagement membership grant; SCIM UPDATE → attribute sync; SCIM DELETE → TASK-PORTAL-004 deprovision flow (out-of-scope here)
  - DEC-869 2026-05-17 — Closed enum `portal_idp_kind` = {`saml`, `oidc`}; CI cardinality test asserts 2
  - DEC-870 2026-05-17 — SAML response replay window = 5 min; OIDC ID token `iat` skew tolerance = 60 s (industry standard)
  - DEC-871 2026-05-17 — Per-Engagement IdP config has `enforcement: optional | required` — `required` means email/password sign-in is DISABLED for that Engagement (SSO-only)
  - DEC-872 2026-05-17 — JIT-provisioned subjects carry `auth_method='external_idp'` + `idp_config_id` + `last_sso_at`; differentiated from internal-tenant subjects at every audit row
  - DEC-873 2026-05-17 — memory audit kinds: portal.idp_sign_in, portal.idp_sign_in_failed, portal.scim_user_created, portal.scim_user_updated, portal.idp_config_created, portal.idp_config_rotated, portal.signed_attr_trust_violation
  - DEC-874 2026-05-17 — SAML SP (Service Provider) metadata published at `/saml/v2/{engagement_slug}/metadata` for IdP self-service config
  - DEC-875 2026-05-17 — OIDC RP (Relying Party) discovery via standard JSON at `/oidc/v1/{engagement_slug}/.well-known/openid-configuration`
  - DEC-876 2026-05-17 — Idempotency on SCIM CREATE keyed by `externalId` claim (IdP's stable user identifier); duplicate externalId returns 409 with existing user
  - DEC-877 2026-05-17 — RLS on `portal_idp_configs` + `portal_scim_audit_log` scoped to `(tenant_id, engagement_id)`
  - DEC-878 2026-05-17 — Append-only `portal_scim_audit_log` via REVOKE UPDATE, DELETE per task-audit skill rule 12
  - DEC-879 2026-05-17 — JWT session lifetime for IdP-authenticated subjects: 8h max (vs 24h for internal); enforces re-validation against IdP for sensitive ops
  - DEC-880 2026-05-17 — SAML Assertion encryption (xmlenc) optional but recommended; OIDC ID tokens always signed; both verify against IdP-published JWKS / X.509 chain
  - DEC-881 2026-05-17 — SP-initiated AND IdP-initiated flows supported for SAML (per OASIS SAML 2.0 conformance); only SP-initiated for OIDC (per OAuth 2.1 PKCE requirement)
  - DEC-882 2026-05-17 — Per-Engagement IdP config requires `tenant_admin` role at TENANT level (not engagement_admin) — IdP misconfig has cross-engagement blast radius
  - DEC-883 2026-05-17 — Force re-auth at JWT mint when `last_sso_at > 8h ago`; expired SSO session → redirect to IdP
  - DEC-884 2026-05-17 — SCIM token rotation: quarterly mandatory + 60s overlap window; old token accepted during overlap, both emit `portal.scim_token_rotation` informational row
  - DEC-885 2026-05-17 — Group-to-role mapping is many-to-one: multiple IdP groups can map to one CyberOS role; one IdP group cannot map to multiple roles
  - DEC-886 2026-05-17 — Audit-row PII: SAML AttributeStatement claims + OIDC ID-token claims PII-scrubbed via TASK-MEMORY-111 (email → email_hash16; name → name_hash16; raw retained only in subject row, RLS-scoped)
  - eIDAS Reg. 910/2014 (QES baseline — external IdP must be eIDAS-conformant for EU regulated tenants; placeholder enforcement at slice 1 — full QES integration task-AUTH-2xx)
  - PDPL Law 91/2025 Art. 5 (data minimisation — only claims explicitly mapped to roles are persisted; unrecognised claims discarded at JIT)
  - GDPR Art. 28 (data processor — IdP acts as DPA-bound processor; PORTAL signs DPA template at IdP config time)

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/Cargo.toml                                       # new crate
    - services/portal/migrations/0001_portal_idp_configs.sql           # per-Engagement IdP binding
    - services/portal/migrations/0002_portal_scim_audit_log.sql        # append-only SCIM event journal
    - services/portal/migrations/0003_portal_idp_groups_map.sql        # IdP groups → CyberOS roles map
    - services/portal/migrations/0004_portal_scim_tokens.sql           # per-Engagement SCIM bearer tokens
    - services/portal/src/lib.rs
    - services/portal/src/idp/mod.rs                                   # IdP orchestrator
    - services/portal/src/idp/saml.rs                                  # SAML 2.0 SP (response verify + JIT)
    - services/portal/src/idp/oidc.rs                                  # OIDC RP (per-Engagement metadata + JIT)
    - services/portal/src/idp/discovery.rs                             # email-domain → IdP hint
    - services/portal/src/idp/claim_mapping.rs                         # claim → role resolver (closed enum)
    - services/portal/src/idp/signed_attr.rs                           # AttributeStatement signature verify
    - services/portal/src/scim/mod.rs                                  # SCIM 2.0 endpoint
    - services/portal/src/scim/users.rs                                # /scim/v2/{eng}/Users handlers
    - services/portal/src/scim/groups.rs                               # /scim/v2/{eng}/Groups handlers
    - services/portal/src/scim/token_auth.rs                           # bearer-token middleware
    - services/portal/src/scim/idempotency.rs                          # externalId-based idempotency
    - services/portal/src/repo/idp_configs.rs                          # IdP config CRUD (KMS-wrapped)
    - services/portal/src/repo/scim_audit.rs                           # SCIM audit log writer
    - services/portal/src/handlers/sp_metadata.rs                      # /saml/v2/{eng}/metadata
    - services/portal/src/handlers/rp_discovery.rs                     # /oidc/v1/{eng}/.well-known/openid-configuration
    - services/portal/src/handlers/idp_config_admin.rs                 # POST /v1/admin/engagements/{id}/idp + rotation
    - services/portal/src/audit/portal_events.rs                       # 7 memory row builders
    - services/portal/tests/saml_happy_test.rs
    - services/portal/tests/saml_replay_window_test.rs
    - services/portal/tests/saml_unsigned_attr_dropped_test.rs
    - services/portal/tests/oidc_happy_test.rs
    - services/portal/tests/oidc_iat_skew_test.rs
    - services/portal/tests/scim_create_idempotency_test.rs
    - services/portal/tests/scim_token_rotation_test.rs
    - services/portal/tests/claim_to_role_mapping_test.rs
    - services/portal/tests/idp_kind_enum_cardinality_test.rs
    - services/portal/tests/jit_provisioning_test.rs
    - services/portal/tests/per_engagement_isolation_test.rs
    - services/portal/tests/sso_enforcement_required_test.rs
    - services/portal/tests/8h_re_auth_test.rs
    - services/portal/tests/discovery_hint_test.rs
    - services/portal/tests/audit_emission_test.rs

  modified_files:
    - services/auth/src/admin/subjects.rs                              # expose JIT helper consumed by PORTAL
    - services/auth/src/handlers/login.rs                              # delegate to PORTAL when engagement_id present + IdP required

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/auth/src/{admin,handlers}/**
    - file_write: services/portal/{src,tests,migrations}/**
    - file_write: services/auth/src/admin/subjects.rs
    - file_write: services/auth/src/handlers/login.rs
    - bash: cd services/portal && cargo test

  disallowed_tools:
    - honour SAML attributes without signature verification (per DEC-864)
    - extend `portal_idp_kind` beyond {saml, oidc} without ADR (per DEC-869)
    - allow SCIM token in URL path (must be Authorization header per RFC 6750)
    - persist IdP private keys in plaintext (KMS-wrapped only per DEC-866)
    - bypass per-Engagement RLS (per DEC-877)
    - allow JIT-provisioned subject to elevate role beyond claim-mapped (per DEC-863)

effort_hours: 10
subtasks:
  - "0.6h: 0001..0004 migrations (idp_configs + scim_audit + groups_map + scim_tokens) + RLS + REVOKE"
  - "1.0h: saml.rs — Response verify + Assertion signature + replay window + AttributeStatement extract"
  - "0.6h: signed_attr.rs — signature verify against IdP metadata cert thumbprint"
  - "0.8h: oidc.rs — per-Engagement OIDC RP + ID token verify + nonce/state binding"
  - "0.5h: discovery.rs — email-domain → IdP routing"
  - "0.6h: claim_mapping.rs — closed enum + many-to-one group mapping"
  - "0.8h: scim/users.rs — Create/Get/Update/Delete + externalId idempotency"
  - "0.5h: scim/groups.rs — group sync + role mapping trigger"
  - "0.4h: scim/token_auth.rs + scim/idempotency.rs"
  - "0.5h: handlers/sp_metadata.rs + handlers/rp_discovery.rs"
  - "0.4h: handlers/idp_config_admin.rs (tenant_admin gated)"
  - "0.4h: audit/portal_events.rs (7 builders)"
  - "1.5h: tests — 15 test files covering happy + replay + signature drop + idempotency + isolation + 8h re-auth"
  - "0.4h: auth/handlers/login.rs delegation hook"
  - "0.5h: integration smoke against Okta+Azure+OneLogin sandbox tenants"

risk_if_skipped: "Without external IdP support, every client-tenant user creates a CyberOS-local password account — non-starter for enterprise prospects who require SSO (Okta/Azure AD/Google Workspace). TASK-PORTAL-005 (Branded Genie) needs JIT-provisioned subjects to attach the right scope_grants. TASK-PORTAL-004 (SCIM deprovision) is a security commitment (PDPL Art. 17 + GDPR Art. 17 right to erasure) that depends on SCIM endpoints existing. Without DEC-864's signed-attribute trust, attribute injection turns any IdP into a privilege-escalation vector. Without DEC-877's per-Engagement RLS, one Engagement's IdP config bleeds into another's. Without DEC-871's enforcement modes, regulated clients can't disable password fallback. Without DEC-885's many-to-one group mapping, common patterns ('Sales + Marketing both get viewer role') require duplicated config. The 10h effort lands the enterprise-grade external identity primitive that unblocks 4 PORTAL tasks + every regulated client deployment."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship per-Engagement external IdP (SAML 2.0 + OIDC) sign-in for client-tenant users with SCIM 2.0 JIT provisioning, claim → role mapping, signed-attribute trust chain, per-Engagement isolation, and 7 memory audit kinds.

1. **MUST** define the `portal_idp_configs` table at migration `0001`: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, idp_kind portal_idp_kind NOT NULL, idp_name TEXT NOT NULL, idp_entity_id TEXT NOT NULL, idp_metadata_url TEXT, idp_signing_cert_kms_blob BYTEA NOT NULL, idp_signing_cert_thumbprint TEXT NOT NULL, idp_kms_key_id TEXT NOT NULL, enforcement TEXT NOT NULL CHECK (enforcement IN ('optional','required')) DEFAULT 'optional', email_domain_hint TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), rotated_at TIMESTAMPTZ, status TEXT NOT NULL CHECK (status IN ('active','rotated','revoked')) DEFAULT 'active')`. Partial unique `(tenant_id, engagement_id) WHERE status='active'` enforces one active IdP per Engagement.

2. **MUST** define the closed `portal_idp_kind` enum at migration `0001` with exactly 2 values: `saml, oidc` (DEC-869). CI cardinality test asserts 2; adding a third requires schema migration + DEC entry.

3. **MUST** define `portal_idp_groups_map` at migration `0003`: `(idp_config_id UUID NOT NULL, idp_group_name TEXT NOT NULL, cyberos_role TEXT NOT NULL, PRIMARY KEY (idp_config_id, idp_group_name))`. Many-to-one (multiple IdP groups → one CyberOS role per DEC-885); one IdP group cannot map to multiple roles (composite-PK enforces).

4. **MUST** define `portal_scim_tokens` at migration `0004`: `(engagement_id UUID PRIMARY KEY, token_sha256 CHAR(64) NOT NULL, kms_key_id TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), rotated_at TIMESTAMPTZ, status TEXT NOT NULL CHECK (status IN ('active','rotated','revoked')))`. Bearer-token form per DEC-867; rotation per DEC-884.

5. **MUST** define `portal_scim_audit_log` at migration `0002`: append-only event journal: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, scim_operation TEXT NOT NULL CHECK (scim_operation IN ('user_create','user_update','user_delete','group_create','group_update','group_delete','token_rotated')), external_id TEXT, subject_id UUID, request_sha256 CHAR(64) NOT NULL, response_status INT NOT NULL, ts TIMESTAMPTZ NOT NULL DEFAULT now())`. REVOKE UPDATE, DELETE from `cyberos_app` (task-audit skill rule 12).

6. **MUST** enforce RLS with both USING and WITH CHECK on all 4 PORTAL tables (task-audit skill rule 13). Policy: `tenant_id = current_setting('auth.tenant_id')::uuid` PLUS per-Engagement scope check for staff-level reads.

7. **MUST** expose `POST /v1/admin/engagements/{engagement_id}/idp` for IdP config creation. Caller MUST have role `tenant_admin` at the TENANT level per DEC-882 (not engagement_admin — IdP misconfig has cross-engagement blast radius). Body validates: `idp_kind`, `idp_entity_id`, `idp_metadata_url` (HTTPS-only) OR `idp_signing_cert_pem` (one of two; if URL provided, fetched + parsed inline), `enforcement`, `email_domain_hint`. Handler KMS-encrypts the signing cert + persists, emits `portal.idp_config_created`.

8. **MUST** support SAML 2.0 SP per OASIS SAML 2.0 + DEC-861 + DEC-880. The SP endpoint at `/saml/v2/{engagement_slug}/acs` (Assertion Consumer Service) accepts POST-binding Responses. Verification:
    - Locate `idp_config` via `engagement_slug`; reject if no active config.
    - XML-parse the Response; reject if non-well-formed.
    - Verify the SAML Response signature against `idp_signing_cert_kms_blob` (xmldsig per W3C; cert chain validated against `idp_signing_cert_thumbprint`).
    - Verify `InResponseTo` matches a server-side pending RequestID (CSRF defense; 5-min TTL).
    - Verify `Conditions/NotBefore` and `Conditions/NotOnOrAfter` (replay window = 5 min per DEC-870).
    - Per DEC-864 + #15: only honour AttributeStatements whose containing Assertion is signed; unsigned attributes are dropped + emit `portal.signed_attr_trust_violation`.
    - Extract `NameID` (subject identifier) + signed AttributeStatement claims.
    - JIT-provision via #11.

9. **MUST** support OIDC RP per TASK-AUTH-104 protocol layer + per-Engagement metadata at `/oidc/v1/{engagement_slug}/.well-known/openid-configuration`. The callback at `/oidc/v1/{engagement_slug}/callback` verifies:
    - PKCE code_verifier matches the stored challenge (per TASK-AUTH-104 DEC-401).
    - `state` nonce matches the server-issued state (single-use, 5-min TTL).
    - ID token signature against IdP JWKS (cached 24h, kid-based rotation per TASK-AUTH-104 DEC-402).
    - `iat` skew ≤ 60 s per DEC-870.
    - `aud` claim matches the per-Engagement client_id.
    - JIT-provision via #11.

10. **MUST** expose SAML SP metadata at `GET /saml/v2/{engagement_slug}/metadata` per DEC-874 — XML document with `<EntityDescriptor>` + `<SPSSODescriptor>` + AssertionConsumerService URL + signing cert (if encryption requested). IdP admins self-configure their IdP with this URL.

11. **MUST** JIT-provision via TASK-AUTH-002 at first SSO sign-in per DEC-862. Flow:
    - Lookup existing subject by `(tenant_id, engagement_id, external_id)` where `external_id` = SAML NameID OR OIDC `sub` claim.
    - If exists: load + update `last_sso_at` + sync claim-derived role per #12 + return.
    - If not exists: INSERT subject via TASK-AUTH-002 with `auth_method='external_idp'`, `idp_config_id`, `external_id`, `email` (claim), `display_name` (claim), `role` (claim-mapped per #12).
    - Bind subject to Engagement via TASK-AUTH-101 RBAC grant.
    - Emit `portal.scim_user_created` (sev-2 — material identity event).

12. **MUST** apply closed claim → role mapping per DEC-863 + DEC-885:
    - Default role for any JIT-provisioned subject = `client_viewer`.
    - For each IdP group in the claim's `groups` array (SAML AttributeStatement `Group` attribute OR OIDC `groups` claim), lookup `portal_idp_groups_map(idp_config_id, idp_group_name)` → `cyberos_role`. Highest-privilege match wins (role ordinal: viewer < editor < admin).
    - Unrecognised groups silently ignored (no role elevation).
    - Role change at re-auth: if claim-mapped role differs from current, UPDATE subject + emit `portal.scim_user_updated`.

13. **MUST** expose SCIM 2.0 endpoints per RFC 7643 + RFC 7644 + DEC-867:
    - `POST   /scim/v2/{engagement_slug}/Users` — create (idempotent on `externalId`).
    - `GET    /scim/v2/{engagement_slug}/Users/{id}` — read.
    - `PATCH  /scim/v2/{engagement_slug}/Users/{id}` — partial update per RFC 7644 §3.5.2.
    - `PUT    /scim/v2/{engagement_slug}/Users/{id}` — full replace.
    - `DELETE /scim/v2/{engagement_slug}/Users/{id}` — soft-tombstone (delegate to TASK-PORTAL-004 for full session-invalidation flow).
    - `POST   /scim/v2/{engagement_slug}/Groups` + analogous CRUD.
    - Bearer-token auth via Authorization header (DEC-867); token validated against `portal_scim_tokens.token_sha256` SHA-256-hashed lookup.

14. **MUST** enforce SCIM idempotency on `externalId` claim per DEC-876. Duplicate create with same externalId returns 409 + body `{ "detail": "User with this externalId already exists", "existing_id": "<id>" }`. RFC 7644 §3.3 conformant.

15. **MUST** drop unsigned AttributeStatement claims per DEC-864 + #8. The signed_attr.rs module walks the XML tree, identifies the enclosing `<ds:Signature>` for each `<saml:AttributeStatement>`, verifies the signature against the configured cert thumbprint, and includes ONLY signed attributes in the JIT claim set. Any unsigned attribute encountered emits a `portal.signed_attr_trust_violation` sev-2 memory row with `(engagement_id, attribute_name, idp_entity_id)`.

16. **MUST** support per-Engagement `enforcement: required` per DEC-871. When `required`, the standard email/password login endpoint (TASK-AUTH-004) returns `403 + { error: "sso_required", idp_redirect_url: "https://portal.cyberos.world/sso/<engagement_slug>" }` for any user whose `auth_method='external_idp'` OR who belongs to a `required`-enforcement Engagement. The check is at JWT mint, before password verification.

17. **MUST** enforce JWT session lifetime 8h for IdP-authenticated subjects per DEC-879 + DEC-883. TASK-AUTH-004 JWT mint reads `subject.auth_method` — if `external_idp`, sets `exp = now + 8h`. At any sensitive operation, the handler checks `last_sso_at`; if > 8h ago AND `enforcement='required'`, redirect to IdP for re-authentication.

18. **MUST** support email-domain IdP discovery per DEC-865. The `/v1/portal/sign-in?email=alice@acme.com` endpoint extracts the email domain, matches against `portal_idp_configs.email_domain_hint`, and on match returns `{ redirect_url: "https://portal.cyberos.world/sso/<engagement_slug>", idp_name: "Acme SSO" }`. No match → `{ fallback: "email_password" }`.

19. **MUST** rotate SCIM tokens quarterly per DEC-884. The `POST /v1/admin/engagements/{id}/scim-token/rotate` endpoint generates a new 32-byte token (base64url-encoded), SHA-256-hashes for storage, marks the old token as `rotated`, sets a 60-second overlap window during which BOTH tokens are accepted. Old token cleanup after overlap. Emits `portal.scim_token_rotation` informational row (not in core 7-kind list per DEC-873).

20. **MUST** emit 7 memory audit row kinds per DEC-873 (task-audit skill rule 6 namespace pattern `^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$`):
    - `portal.idp_sign_in` (sev-2)
    - `portal.idp_sign_in_failed` (sev-2 — security signal)
    - `portal.scim_user_created` (sev-2)
    - `portal.scim_user_updated` (sev-3)
    - `portal.idp_config_created` (sev-1 — material security event)
    - `portal.idp_config_rotated` (sev-1)
    - `portal.signed_attr_trust_violation` (sev-1)

    Plus 1 supporting kind: `portal.scim_token_rotation` (sev-3 — informational). PII-scrubbed via TASK-MEMORY-111 per DEC-886.

21. **MUST** support SP-initiated AND IdP-initiated SAML flows per DEC-881. IdP-initiated POSTs lacking `InResponseTo` are accepted ONLY when `portal_idp_configs.allow_idp_initiated=true` (defaults false; explicit opt-in due to CSRF risk). OIDC supports SP-initiated only per OAuth 2.1 PKCE requirement.

22. **MUST** PII-scrub all audit rows per DEC-886 + task-audit skill rule 18. The 7 memory row kinds carry `email_hash16`, `name_hash16`, `external_id_hash16` — raw values retained in `subjects` (RLS-scoped to tenant) only.

23. **MUST** thread W3C `traceparent` across SAML/OIDC roundtrip + JIT provisioning + SCIM operations (task-audit skill rule 22 + 23 + 24). Trace_id persisted in `portal_scim_audit_log.trace_id` column (added via 0002 migration).

24. **MUST NOT** persist IdP private keys (CyberOS doesn't hold IdP signing keys — verification uses IdP-published public certs only). IdP public certs ARE KMS-wrapped per DEC-866 because operational hardening (secret-of-secrets pattern); not because they're cryptographically private.

25. **MUST NOT** allow a JIT-provisioned subject to elevate role beyond claim-mapped per DEC-863. Subsequent role mutations via TASK-AUTH-101 admin UI are permitted (tenant_admin can override claim-mapping), but the JIT path itself is closed.

26. **MUST** be rate-limited at the SP endpoint (`/saml/v2/{eng}/acs`) — 100 req/min/IP (legitimate IdP traffic well under this); excess returns 429 + `Retry-After` (defends against credential-stuffing via SAML brute force).

27. **MUST** emit per-Engagement metric `portal_idp_sign_in_duration_seconds` to OBS (TASK-OBS-005). Alarm sev-2 if p95 > 3 s sustained 10 min (latency indicator for IdP-side issues).

---

## §2 — Why this design (rationale for humans)

**Why per-Engagement (not per-tenant) IdP binding (§1 #1, DEC-860)?** A single CyberOS tenant frequently hosts many client Engagements — e.g., a consulting firm runs 50 client projects, each client having their own corporate IdP. Per-tenant IdP forces the consulting firm to pick one IdP for all clients — impossible. Per-Engagement matches the commercial reality.

**Why signed-attribute trust chain (§1 #15, DEC-864)?** SAML AttributeStatements are XML elements that can appear in unsigned form within a signed Response. Naively trusting all AttributeStatements lets an attacker (or compromised IdP intermediary) inject claims like `groups=["cyberos-admin"]`. The mitigation: walk the XML, verify the signature scope, drop anything not within the signature's coverage. This is the OWASP-recommended pattern for SAML SP implementations.

**Why claim → role mapping closed-set (§1 #12, DEC-863)?** Open-set mapping ("any group name in IdP → same-named role in CyberOS") is a privilege-escalation vector — an IdP-side admin who can edit group names can grant themselves CyberOS admin by renaming a group. Closed-set mapping ("only mappings explicitly configured in `portal_idp_groups_map` are honoured") makes the privilege boundary explicit and audit-able.

**Why SCIM 2.0 instead of custom user-sync API (§1 #13)?** SCIM is the de facto enterprise standard (Okta, Azure AD, OneLogin, Google Workspace all speak it). Custom API forces every enterprise to write a custom connector. SCIM is straightforward to implement (RFC 7644 is ~80 pages of clear spec) and unlocks ~100% of enterprise customers without custom work.

**Why 8h JWT lifetime for IdP-auth (§1 #17, DEC-879)?** Industry convention. Internal users get 24h sessions (Okta/Azure recommendation). External-IdP users get 8h because (a) IdP-side session may be revoked by the customer's security team mid-day, and (b) the IdP may want to enforce re-auth for sensitive ops. 8h is a reasonable middle ground between security and UX.

**Why required-enforcement mode (§1 #16, DEC-871)?** Regulated industries (finance, healthcare) often have compliance requirements like "all employees must SSO, no password fallback." Without `enforcement: required`, a tenant admin could enable IdP for convenience but leave password fallback open — defeating the compliance objective.

**Why many-to-one group mapping (§1 #12, DEC-885)?** Common enterprise pattern: "Sales group + Marketing group + Customer Success group all get the `client_viewer` role." One-to-many (one group → multiple roles) creates ambiguity at resolution. Many-to-one (multiple groups → one role) is the simpler, less-error-prone shape.

**Why tenant_admin (not engagement_admin) for IdP config (§1 #7, DEC-882)?** IdP misconfig has cross-Engagement blast radius — a wrong cert at Engagement A doesn't break Engagement A alone; it can cause cascading auth failures across the tenant if the misconfig hits a shared cert store. Tenant_admin is the smallest scope that owns the blast radius.

**Why 7 audit kinds with sev-1 weighting on config events (§1 #20, DEC-873)?** Identity-system config changes are forensically critical. Sev-1 ensures they appear in compliance dashboards (TASK-OBS-008) and trigger immediate operator review. Sign-in events are sev-2 (high-volume but important); transient sign-in failures are sev-2 (security signal); SCIM user updates are sev-3 (high-volume routine).

**Why SP-metadata published at a per-Engagement URL (§1 #10, DEC-874)?** IdP admins (Okta, Azure) self-configure by entering an SP metadata URL — they paste the URL, IdP fetches + parses. Per-Engagement URLs let each Engagement's IdP admin do this without operator hand-holding. Single global SP metadata wouldn't carry per-Engagement AssertionConsumerService URLs.

**Why drop unsigned attributes silently into the audit rather than fail the sign-in (§1 #15)?** Failing the sign-in on unsigned attributes would break IdPs that bundle some attributes unsigned (rare but real). The audit row surfaces the violation to operators; recurring violations from one IdP are a config issue to escalate. Mid-incident customer-facing breakage from an over-strict rejection is worse than the audit-row residue.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0001_portal_idp_configs.sql
CREATE TYPE portal_idp_kind AS ENUM ('saml','oidc');

CREATE TABLE portal_idp_configs (
  id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  idp_kind portal_idp_kind NOT NULL,
  idp_name TEXT NOT NULL,
  idp_entity_id TEXT NOT NULL,
  idp_metadata_url TEXT,
  idp_signing_cert_kms_blob BYTEA NOT NULL,
  idp_signing_cert_thumbprint TEXT NOT NULL,
  idp_kms_key_id TEXT NOT NULL,
  enforcement TEXT NOT NULL DEFAULT 'optional'
    CHECK (enforcement IN ('optional','required')),
  email_domain_hint TEXT,
  allow_idp_initiated BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  rotated_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active','rotated','revoked'))
);
CREATE UNIQUE INDEX uniq_idp_active_per_engagement
  ON portal_idp_configs(tenant_id, engagement_id)
  WHERE status='active';
ALTER TABLE portal_idp_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_idp_configs_rls ON portal_idp_configs
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_idp_configs FROM cyberos_app;
GRANT UPDATE (rotated_at, status) ON portal_idp_configs TO cyberos_app;

-- 0002_portal_scim_audit_log.sql
CREATE TABLE portal_scim_audit_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  scim_operation TEXT NOT NULL
    CHECK (scim_operation IN ('user_create','user_update','user_delete','group_create','group_update','group_delete','token_rotated')),
  external_id TEXT,
  subject_id UUID,
  request_sha256 CHAR(64) NOT NULL,
  response_status INT NOT NULL,
  trace_id CHAR(32),
  ts TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE portal_scim_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_scim_audit_log_rls ON portal_scim_audit_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_scim_audit_log FROM cyberos_app;

-- 0003_portal_idp_groups_map.sql
CREATE TABLE portal_idp_groups_map (
  idp_config_id UUID NOT NULL REFERENCES portal_idp_configs(id),
  idp_group_name TEXT NOT NULL,
  cyberos_role TEXT NOT NULL,
  PRIMARY KEY (idp_config_id, idp_group_name)
);
ALTER TABLE portal_idp_groups_map ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_idp_groups_map_rls ON portal_idp_groups_map
  USING (idp_config_id IN (SELECT id FROM portal_idp_configs
                            WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (idp_config_id IN (SELECT id FROM portal_idp_configs
                                  WHERE tenant_id = current_setting('auth.tenant_id')::uuid));

-- 0004_portal_scim_tokens.sql
CREATE TABLE portal_scim_tokens (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  token_sha256 CHAR(64) NOT NULL,
  kms_key_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  rotated_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active','rotated','revoked'))
);
CREATE UNIQUE INDEX uniq_active_scim_token
  ON portal_scim_tokens(engagement_id) WHERE status='active';
CREATE INDEX idx_scim_token_sha256 ON portal_scim_tokens(token_sha256);
ALTER TABLE portal_scim_tokens ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_scim_tokens_rls ON portal_scim_tokens
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_scim_tokens FROM cyberos_app;
GRANT UPDATE (rotated_at, status) ON portal_scim_tokens TO cyberos_app;
```

### 3.2 Rust types

```rust
// services/portal/src/idp/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type)]
pub enum PortalIdpKind { Saml, Oidc }

#[derive(Debug)]
pub struct IdpConfig {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub engagement_id: uuid::Uuid,
    pub kind: PortalIdpKind,
    pub entity_id: String,
    pub signing_cert: openssl::x509::X509,  // KMS-decrypted at load
    pub signing_cert_thumbprint: String,
    pub enforcement: Enforcement,
    pub allow_idp_initiated: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Enforcement { Optional, Required }

#[derive(Debug)]
pub struct JitClaims {
    pub external_id: String,    // SAML NameID or OIDC sub
    pub email: String,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub trace_id: String,
}
```

### 3.3 REST endpoints

```text
# SAML SP
POST   /saml/v2/{engagement_slug}/acs               (IdP→SP Assertion Consumer)
GET    /saml/v2/{engagement_slug}/metadata           (public SP metadata)

# OIDC RP
GET    /oidc/v1/{engagement_slug}/.well-known/openid-configuration  (public)
POST   /oidc/v1/{engagement_slug}/callback           (OIDC return)

# Discovery
GET    /v1/portal/sign-in                            (email-domain hint)

# Admin (tenant_admin only)
POST   /v1/admin/engagements/{id}/idp                (create IdP config)
PATCH  /v1/admin/engagements/{id}/idp                (rotate signing cert)
POST   /v1/admin/engagements/{id}/scim-token/rotate  (quarterly rotation)
POST   /v1/admin/engagements/{id}/idp/groups-map     (configure claim → role)

# SCIM 2.0 (bearer token)
{POST,GET,PATCH,PUT,DELETE} /scim/v2/{engagement_slug}/Users[/{id}]
{POST,GET,PATCH,PUT,DELETE} /scim/v2/{engagement_slug}/Groups[/{id}]
```

---

## §4 — Acceptance criteria

1. **SAML happy path** — POST signed SAML Response → JIT subject created with claim-derived role → JWT minted with 8h TTL → `portal.idp_sign_in` memory row emitted.
2. **OIDC happy path** — OIDC callback with valid ID token → JIT subject created → role from `groups` claim → 8h JWT.
3. **SAML replay window** — Response with `NotOnOrAfter` 6 min ago → 401 + `portal.idp_sign_in_failed` with reason='replay_window_exceeded'.
4. **OIDC iat skew** — ID token with `iat` 70 s in future → 401 + reason='iat_skew_exceeded'.
5. **Unsigned attribute dropped** — Response with mixed signed + unsigned attributes → only signed in JIT claims; `portal.signed_attr_trust_violation` emitted; sign-in still succeeds.
6. **SCIM externalId idempotency** — second SCIM CREATE with same externalId returns 409 + existing user id; no duplicate subject.
7. **portal_idp_kind cardinality** — `idp_kind_enum_cardinality_test` asserts enum = exactly `{saml, oidc}`.
8. **Per-Engagement isolation** — Engagement A's IdP config invisible to Engagement B; cross-Engagement lookup returns 404.
9. **Required enforcement disables password** — user in `required`-enforcement Engagement attempts TASK-AUTH-004 password login → 403 + `sso_required` + IdP redirect URL.
10. **8h re-auth trigger** — subject with `last_sso_at` > 8h ago hitting sensitive endpoint → redirect to IdP.
11. **Email-domain discovery hint** — `/v1/portal/sign-in?email=alice@acme.com` where `acme.com` matches `email_domain_hint` → redirect_url returned.
12. **SCIM token rotation 60s overlap** — both old + new token accepted for 60 s post-rotation; `portal.scim_token_rotation` emitted.
13. **Claim → role many-to-one** — IdP groups `["Sales","Marketing"]` both mapped to `client_viewer` → JIT user gets `client_viewer` (single role, no conflict).
14. **Unrecognised group ignored** — IdP claim group `Random` not in `portal_idp_groups_map` → default `client_viewer` role assigned (no elevation).
15. **tenant_admin role gate on IdP config** — engagement_admin POSTing `/v1/admin/engagements/{id}/idp` → 403; tenant_admin → 201.
16. **PII scrubbed in audit rows** — `portal.idp_sign_in` row carries `email_hash16` not raw email.
17. **W3C trace_id threaded** — single trace_id present in SAML ACS span → JIT span → SCIM audit log row → memory row.
18. **IdP-initiated SAML allowed only when opted in** — Response lacking InResponseTo with `allow_idp_initiated=false` → 401; with `true` → accepted.
19. **SCIM token in URL rejected** — `?token=xxx` query param ignored; Authorization header required.
20. **IdP signing cert KMS-wrapped** — `portal_idp_configs.idp_signing_cert_kms_blob` decryptable only via the configured KMS key; raw cert never written to disk.

---

## §5 — Verification

### 5.1 `saml_happy_test.rs`

```rust
#[tokio::test]
async fn saml_signed_response_jit_provisions_subject() {
    let ctx = TestContext::with_engagement_idp("eng-acme", IdpKind::Saml).await;
    let saml_response = ctx.fake_idp.signed_response("alice@acme.com",
        vec![("groups", vec!["Engineering"])]).await;
    let r = ctx.post(&format!("/saml/v2/eng-acme/acs")).body(saml_response).send().await.unwrap();
    assert_eq!(r.status(), 302);  // redirect with JWT cookie set

    let subj = ctx.repo.subjects.find_by_external_id(ctx.tenant_id, "alice@acme.com").await.unwrap();
    assert_eq!(subj.auth_method, "external_idp");
    assert_eq!(subj.role, "client_viewer");  // default; no group mapping yet

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.idp_sign_in"));
    assert!(audit.iter().any(|r| r.kind == "portal.scim_user_created"));
}
```

### 5.2 `saml_unsigned_attr_dropped_test.rs`

```rust
#[tokio::test]
async fn unsigned_attribute_dropped_and_audited() {
    let ctx = TestContext::with_engagement_idp("eng-acme", IdpKind::Saml).await;
    let saml_response = ctx.fake_idp.signed_response_with_unsigned_attrs("alice@acme.com",
        signed=vec![("groups", vec!["Engineering"])],
        unsigned=vec![("role_override", vec!["super_admin"])]).await;

    let r = ctx.post(&format!("/saml/v2/eng-acme/acs")).body(saml_response).send().await.unwrap();
    assert_eq!(r.status(), 302);  // sign-in still succeeds

    let subj = ctx.repo.subjects.find_by_external_id(ctx.tenant_id, "alice@acme.com").await.unwrap();
    assert_eq!(subj.role, "client_viewer");  // role_override IGNORED

    let audit = ctx.memory_rows().await;
    let violation = audit.iter().find(|r| r.kind == "portal.signed_attr_trust_violation").unwrap();
    assert_eq!(violation.payload["attribute_name"], "role_override");
}
```

### 5.3 `oidc_iat_skew_test.rs`

```rust
#[tokio::test]
async fn oidc_iat_in_future_rejected() {
    let ctx = TestContext::with_engagement_idp("eng-acme", IdpKind::Oidc).await;
    let id_token = ctx.fake_idp.id_token_with_iat_offset("alice@acme.com",
        chrono::Duration::seconds(70)).await;
    let r = ctx.post(&format!("/oidc/v1/eng-acme/callback"))
        .json(&json!({"id_token": id_token, "state": ctx.valid_state()})).send().await.unwrap();
    assert_eq!(r.status(), 401);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "iat_skew_exceeded");
}
```

### 5.4 `scim_create_idempotency_test.rs`

```rust
#[tokio::test]
async fn scim_create_idempotent_on_external_id() {
    let ctx = TestContext::with_scim_token("eng-acme").await;
    let body = json!({"externalId": "okta-12345", "userName": "alice",
                       "emails": [{"value": "alice@acme.com", "primary": true}]});

    let r1 = ctx.scim_post("/scim/v2/eng-acme/Users", &body).await;
    assert_eq!(r1.status(), 201);
    let id1: String = r1.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().into();

    let r2 = ctx.scim_post("/scim/v2/eng-acme/Users", &body).await;
    assert_eq!(r2.status(), 409);
    let body2: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(body2["existing_id"], id1);
}
```

### 5.5 `claim_to_role_mapping_test.rs`

```rust
#[tokio::test]
async fn many_to_one_group_mapping() {
    let ctx = TestContext::with_engagement_idp("eng-acme", IdpKind::Saml).await;
    ctx.configure_group_map(vec![
        ("Sales", "client_viewer"),
        ("Marketing", "client_viewer"),
        ("Engineering", "client_editor"),
    ]).await;

    let saml = ctx.fake_idp.signed_response("alice@acme.com",
        vec![("groups", vec!["Sales","Marketing"])]).await;
    ctx.post(&format!("/saml/v2/eng-acme/acs")).body(saml).send().await.unwrap();
    let subj = ctx.repo.subjects.find_by_external_id(ctx.tenant_id, "alice@acme.com").await.unwrap();
    assert_eq!(subj.role, "client_viewer");  // both groups map same role

    let bob_saml = ctx.fake_idp.signed_response("bob@acme.com",
        vec![("groups", vec!["Sales","Engineering"])]).await;
    ctx.post(&format!("/saml/v2/eng-acme/acs")).body(bob_saml).send().await.unwrap();
    let bob = ctx.repo.subjects.find_by_external_id(ctx.tenant_id, "bob@acme.com").await.unwrap();
    assert_eq!(bob.role, "client_editor");  // highest-privilege wins
}
```

### 5.6 `sso_enforcement_required_test.rs`

```rust
#[tokio::test]
async fn required_enforcement_blocks_password_login() {
    let ctx = TestContext::with_engagement_idp_enforcement("eng-acme", Enforcement::Required).await;
    let subj = ctx.jit_provision_via_saml("alice@acme.com").await;

    let r = ctx.post("/v1/auth/login").json(&json!({
        "email": "alice@acme.com", "password": "any-password",
    })).send().await.unwrap();
    assert_eq!(r.status(), 403);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "sso_required");
    assert!(body["idp_redirect_url"].as_str().unwrap().contains("/sso/eng-acme"));
}
```

### 5.7 `8h_re_auth_test.rs`

```rust
#[tokio::test]
async fn last_sso_older_than_8h_triggers_re_auth() {
    let ctx = TestContext::with_engagement_idp_enforcement("eng-acme", Enforcement::Required).await;
    let subj = ctx.jit_provision_via_saml("alice@acme.com").await;
    sqlx::query("UPDATE subjects SET last_sso_at = now() - interval '9 hours' WHERE id=$1")
        .bind(subj.id).execute(&ctx.pool).await.unwrap();

    let jwt = ctx.mint_jwt_for(subj.id).await;
    let r = ctx.get("/v1/portal/sensitive-resource").bearer_auth(jwt).send().await.unwrap();
    assert_eq!(r.status(), 401);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "sso_re_auth_required");
}
```

### 5.8 `scim_token_rotation_test.rs`

```rust
#[tokio::test]
async fn scim_token_rotation_60s_overlap() {
    let ctx = TestContext::with_scim_token("eng-acme").await;
    let old_token = ctx.current_scim_token().await;

    let new_token = ctx.rotate_scim_token("eng-acme").await;
    for token in [&old_token, &new_token] {
        let r = ctx.scim_post_with_token("/scim/v2/eng-acme/Users",
            &json!({"externalId": format!("u-{}", uuid::Uuid::new_v4()), "userName": "x"}), token).await;
        assert!(r.status().is_success(), "token {token} failed during overlap");
    }

    tokio::time::sleep(Duration::from_secs(61)).await;
    let r_old = ctx.scim_post_with_token("/scim/v2/eng-acme/Users",
        &json!({"externalId": format!("u-{}", uuid::Uuid::new_v4()), "userName": "y"}), &old_token).await;
    assert_eq!(r_old.status(), 401);
}
```

### 5.9 `per_engagement_isolation_test.rs`

```rust
#[tokio::test]
async fn engagement_a_idp_invisible_to_engagement_b() {
    let ctx = TestContext::with_engagements(vec!["eng-a", "eng-b"]).await;
    ctx.configure_idp("eng-a", IdpKind::Saml).await;

    let r = ctx.get("/saml/v2/eng-b/metadata").send().await.unwrap();
    assert_eq!(r.status(), 404);
}
```

### 5.10 `idp_kind_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn portal_idp_kind_has_exactly_two_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::portal_idp_kind))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels;
    labels.sort();
    assert_eq!(labels, vec!["oidc".to_string(), "saml".to_string()]);
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton. Additional notes below.)

### 6.1 SAML SP Assertion Consumer (`services/portal/src/idp/saml.rs`)

```rust
pub async fn acs(ctx: AppCtx, eng_slug: String, body: Bytes) -> Result<Response, PortalError> {
    let config = ctx.repo.idp_configs.find_active_by_engagement_slug(&eng_slug).await?
        .ok_or(PortalError::NoActiveIdpConfig)?;
    if config.kind != PortalIdpKind::Saml { return Err(PortalError::WrongIdpKind); }

    let response_xml = saml::parse_xml(&body)?;
    saml::verify_response_signature(&response_xml, &config.signing_cert)?;
    let in_response_to = saml::extract_in_response_to(&response_xml);
    if !config.allow_idp_initiated && in_response_to.is_none() {
        return Err(PortalError::IdpInitiatedNotAllowed);
    }
    if let Some(req_id) = &in_response_to {
        ctx.repo.saml_pending_requests.consume_one_shot(req_id).await?;
    }
    saml::verify_replay_window(&response_xml, Duration::minutes(5))?;

    let signed_attrs = signed_attr::extract_signed_attributes_only(&response_xml, &config.signing_cert);
    let claims = JitClaims::from_saml_signed_attrs(&signed_attrs)?;

    let subject_id = jit_provision_or_update(&ctx, &config, &claims).await?;
    let jwt = ctx.auth.mint_jwt_for_external_idp(subject_id, config.tenant_id, &claims).await?;
    emit_audit(&ctx, "portal.idp_sign_in", json!({
        "engagement_id": config.engagement_id, "subject_id": subject_id,
        "external_id_hash16": hash16(&claims.external_id), "idp_kind": "saml",
    })).await;

    Ok(redirect_response(format!("https://portal.cyberos.world/onboarding?token={jwt}")))
}
```

### 6.2 SCIM bearer-token middleware (`services/portal/src/scim/token_auth.rs`)

```rust
pub async fn scim_bearer_auth<B>(
    ctx: AppCtx, eng_slug: String, req: Request<B>, next: Next<B>
) -> Result<Response, StatusCode> {
    let token = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token_sha256 = sha256_hex(token.as_bytes());
    let valid = ctx.repo.scim_tokens.find_by_sha256_active_or_overlap(&token_sha256, &eng_slug).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !valid { return Err(StatusCode::UNAUTHORIZED); }
    Ok(next.run(req).await)
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-AUTH-103** SAML SSO — SAML protocol primitives (Response parse, Assertion verify, cert chain).
- **TASK-AUTH-104** OIDC SSO — OIDC RP primitives (ID token verify, JWKS rotation, PKCE).

**Cross-module (related_tasks):**
- **TASK-AUTH-002** Subject create — JIT calls into this.
- **TASK-AUTH-004** JWT mint — first-login token after SSO.
- **TASK-AUTH-101** RBAC — Engagement membership grants.
- **TASK-PORTAL-001** Scoped read-only views — depends on subjects existing.
- **TASK-PORTAL-002** Per-tenant brand pack — sign-in page brand applied.
- **TASK-PORTAL-004** SCIM deprovision — full session-invalidation on SCIM DELETE.
- **TASK-PORTAL-005** Branded Genie chat — uses external-IdP scope_grants.
- **TASK-TEN-101** Self-serve signup — root-admin can configure IdP post-signup.
- **TASK-AI-003** memory audit-row bridge — 7 + 1 supporting kinds register.
- **TASK-MEMORY-111** PII scrubbing — email/name/external_id hashes.
- **TASK-OBS-007** Auto-runbook — sev-1 IdP-config events trigger CHAT alerts.

**Downstream (blocks):**
- **TASK-PORTAL-004** SCIM deprovision — needs SCIM endpoints + session linkage.
- **TASK-PORTAL-005** Branded Genie — needs IdP-auth scope_grants.

---

## §8 — Example payloads

### 8.1 `portal.idp_sign_in` memory row

```json
{
  "kind": "portal.idp_sign_in",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "system.portal.idp",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "engagement_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
    "subject_id": "7c4e9a2b-1d3f-4e6a-bccc-000000000042",
    "idp_kind": "saml",
    "external_id_hash16": "f8a1b2c3d4e5f607",
    "email_hash16": "9c4e7a8b6d2f1e3a",
    "groups_count": 2,
    "role_assigned": "client_viewer"
  }
}
```

### 8.2 SCIM Create request/response

```json
// POST /scim/v2/eng-acme/Users
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "externalId": "okta-12345",
  "userName": "alice",
  "emails": [{"value": "alice@acme.com", "primary": true}],
  "name": {"givenName": "Alice", "familyName": "Smith"},
  "groups": [{"value": "Engineering"}]
}

// 201 response
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "id": "7c4e9a2b-1d3f-4e6a-bccc-000000000042",
  "externalId": "okta-12345",
  "userName": "alice",
  "emails": [{"value": "alice@acme.com", "primary": true}],
  "meta": {
    "resourceType": "User",
    "created": "2026-05-17T09:14:32.847Z",
    "location": "https://portal.cyberos.world/scim/v2/eng-acme/Users/7c4e9a2b-..."
  }
}
```

### 8.3 IdP config admin POST

```json
{
  "idp_kind": "saml",
  "idp_name": "Acme Corp Okta",
  "idp_entity_id": "https://acme.okta.com/saml2/acme",
  "idp_metadata_url": "https://acme.okta.com/app/abc123/sso/saml/metadata",
  "enforcement": "required",
  "email_domain_hint": "acme.com",
  "allow_idp_initiated": false
}
```

### 8.4 `portal.signed_attr_trust_violation` memory row

```json
{
  "kind": "portal.signed_attr_trust_violation",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "system.portal.idp",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.901Z",
  "payload": {
    "engagement_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
    "idp_entity_id": "https://acme.okta.com/saml2/acme",
    "attribute_name": "role_override",
    "value_hash16": "a1b2c3d4e5f607c8"
  }
}
```

---

## §9 — Open questions

All resolved for slice 1. Deferred:

- **Deferred:** SCIM 2.0 Group sync DELETE → cascade subject-membership revoke — slice 2, TASK-PORTAL-004 path.
- **Deferred:** eIDAS QES (Qualified Electronic Signature) integration for EU regulated tenants — slice 3, task-AUTH-2xx (placeholder).
- **Deferred:** Multi-IdP per Engagement (e.g., Okta primary + Google fallback) — slice 2.
- **Deferred:** SCIM bulk operations (RFC 7644 §3.7) — slice 2 (single-user CRUD only at slice 1).
- **Deferred:** Custom claim mappers for non-`groups` claim sources (e.g., role inferred from `department`) — slice 2.
- **Deferred:** WebAuthn step-up at sensitive ops post-SSO (vs full re-auth) — slice 2, task-AUTH-2xx.
- **Deferred:** Per-Engagement custom domain for SP endpoint (`sso.acme.com/saml2/acs`) — slice 2 + TASK-PORTAL-002 brand pack integration.
- **Deferred:** SAML SLO (Single Log-Out) — slice 2; slice 1 has SCIM DELETE deprovision via TASK-PORTAL-004.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| SAML Response signature invalid | xmldsig verify returns false | 401 + `signature_invalid` + `portal.idp_sign_in_failed` sev-2 | IdP cert rotated without our config update — admin re-imports IdP metadata |
| SAML Response InResponseTo missing (SP-initiated only flow) | absence of `InResponseTo` element | 401 + `idp_initiated_not_allowed` unless `allow_idp_initiated=true` | Admin opts in via config update OR user retries SP-initiated flow |
| SAML Response replay window exceeded | `NotOnOrAfter < now - 5min` | 401 + `replay_window_exceeded` + `portal.idp_sign_in_failed` | User retries; clock-skew investigation if recurring |
| SAML Assertion unsigned attribute present | signed_attr.rs walk finds attr outside Signature scope | Attribute dropped silently; `portal.signed_attr_trust_violation` sev-1 emitted; sign-in still proceeds with signed-only attrs | Operator alerted; IdP-side admin reconfigures to sign all attrs |
| OIDC ID token signature invalid | JWKS verify fails | 401 + `id_token_invalid` + `portal.idp_sign_in_failed` | JWKS rotation drift — cache refresh triggered (24h max stale per TASK-AUTH-104) |
| OIDC ID token iat skew exceeded | `|now - iat| > 60s` | 401 + `iat_skew_exceeded` | User retries; if recurring, NTP investigation |
| OIDC state nonce mismatch | server-side `state` lookup miss | 401 + `state_nonce_invalid` | User restarts sign-in (CSRF protection working as intended) |
| OIDC PKCE code_verifier mismatch | hash compare fails | 401 + `pkce_verify_failed` | User restarts sign-in |
| SCIM bearer token invalid/expired | token_sha256 not in active or overlap set | 401 + `WWW-Authenticate: Bearer error="invalid_token"` | IdP-side SCIM admin updates token; rotation flow used if mid-rotation |
| SCIM externalId collision | unique constraint hit on `subjects(tenant_id, engagement_id, external_id)` | 409 + `{detail, existing_id}` | IdP-side admin investigates duplicate-user state |
| SCIM bulk DELETE attempted (not supported slice 1) | endpoint match | 405 + `bulk_not_supported_slice_1` | Use individual DELETE per RFC 7644 §3.6 |
| IdP signing cert expired (validation fails) | X.509 NotAfter check | 401 + `idp_cert_expired` + sev-1 alert `portal.idp_cert_expired` | Tenant admin invokes IdP config rotation (`PATCH /v1/admin/engagements/{id}/idp`) |
| IdP metadata URL unreachable (during config create) | fetch timeout 10 s | 503 + `idp_metadata_fetch_failed` | Admin uploads cert manually instead OR retries when IdP-side network recovers |
| Per-Engagement IdP config inactive (status=revoked) | lookup filter `WHERE status='active'` returns nothing | 404 + `no_active_idp_config` | Admin re-activates or re-creates config |
| `enforcement=required` user attempts password login | login handler check | 403 + `sso_required` + IdP redirect URL | User follows redirect to IdP |
| 8h re-auth required post-sensitive-op | last_sso_at check at handler entry | 401 + `sso_re_auth_required` + IdP redirect URL | User completes IdP re-auth + retries |
| SCIM token rotation overlap expired before old IdP-side updated | old token used past 60s overlap | 401 + `token_rotation_expired` | Tenant admin re-rotates with longer notice OR rolls back to old token (revoked status) |
| Claim → role map missing for group present in IdP claim | lookup returns no match | Group silently ignored; default `client_viewer` role assigned | Admin configures `portal_idp_groups_map` entry |
| Cross-Engagement RLS bypass attempt | RLS policy USING clause rejects | 404 (no rows returned; appears as not-found) | Inherent — RLS prevents enumeration |
| SAML XML signature wrap attack (XSW) | XML canonical form check + signature reference validation | 401 + `xsw_detected` + sev-1 alert | Inherent — proper SAML lib (libxmlsec) prevents; logged for forensic |
| OIDC ID token aud mismatch (token for different RP) | aud claim check | 401 + `aud_mismatch` | User signs in via correct portal URL |
| Unrecognised IdP entity_id (Response from different IdP than configured) | issuer check | 401 + `unexpected_issuer` | Admin verifies IdP config; possible cert reuse across IdPs (bad practice) |
| KMS unavailable when loading idp_signing_cert | KMS decrypt timeout | 503 + sev-2 alert `portal.kms_unavailable`; sign-in retries | KMS recovers; admin verifies IAM permissions |

---

## §11 — Implementation notes

**§11.1** SAML SP implementation uses `samael` crate (active maintenance, libxmlsec bindings). OIDC RP uses `openidconnect` crate.

**§11.2** SCIM 2.0 conformance: implement Levels 1-2 per RFC 7644 (Users + Groups CRUD + idempotency); Level 3 (bulk, filter expressions) deferred to slice 2.

**§11.3** SCIM Filter expressions (RFC 7644 §3.4.2.2) for GET supported subset: `eq, ne, sw, ew, co, pr, and, or` — sufficient for most Okta/Azure use cases. Full grammar at slice 2.

**§11.4** IdP signing cert thumbprint = SHA-256 of DER-encoded cert (uppercase hex with colon separators, common convention).

**§11.5** Per-Engagement metadata URL endpoint is unauthenticated (per SAML/OIDC convention — metadata is public).

**§11.6** SCIM `meta.location` URL points to the resource's canonical URL; required by RFC 7644 §3.1.

**§11.7** Bearer-token format for SCIM is opaque (not JWT) — 32 random bytes base64url-encoded. Length 43 chars. Easy to rotate.

**§11.8** The `subjects.last_sso_at` column requires a migration to TASK-AUTH-002's table — added via `services/auth/migrations/0XXX_subjects_last_sso_at.sql` referenced by this task's modified_files.

**§11.9** `signed_attr.rs` is the security-critical core. Test coverage MUST include: signed-only attrs (happy), mixed signed+unsigned (drop unsigned), nested Assertions, attributes outside enclosed Signature scope. Library: `xmlsec-rs` with libxmlsec bindings.

**§11.10** SCIM `PATCH` uses JSON Patch (RFC 6902-ish) per RFC 7644 §3.5.2; partial updates avoid full-replace race conditions.

**§11.11** Per DEC-885, role ordinal: `client_viewer < client_editor < client_admin`. Resolver picks max.

**§11.12** The SP-metadata endpoint serves XML; Content-Type `application/samlmetadata+xml` per OASIS.

**§11.13** Per-Engagement OIDC metadata at `/oidc/v1/{eng}/.well-known/openid-configuration` differs from per-Tenant OIDC of TASK-AUTH-104 — PORTAL is the RP-facing-OUTWARD endpoint (IdP-side configures against this), while TASK-AUTH-104 is the AUTH server-as-RP for internal SSO.

**§11.14** `portal_scim_tokens` query path uses `idx_scim_token_sha256` for O(log n) lookup; bearer-auth middleware caches accepted-token results for 5 s to avoid hot-path DB hit (cache invalidated on rotation events).

**§11.15** Force re-auth (§1 #17) is implemented as middleware on routes tagged with `requires_fresh_sso` attribute; the JWT bearer middleware reads `subject.last_sso_at` and decides.

**§11.16** IdP metadata URL fetch (during config create) uses a 10s timeout + content-length cap of 1 MiB to defend against slow-loris + size-amplification attacks.

**§11.17** PII hashing: `email_hash16 = encode(substring(digest(coalesce(lower(email), '') || global_salt, 'sha256') from 1 for 8), 'hex')` — consistent with TASK-TEN-101's hash function. Global salt rotates yearly; old hashes retained for forensic comparison.

**§11.18** SAML `NameID` formats supported: `emailAddress`, `persistent`, `transient` (per OASIS Section 8.3). The `external_id` field stores the NameID value AS-IS; format is informational.

**§11.19** OIDC `sub` claim stability: per OpenID Connect Core 1.0 §5.5, `sub` is stable per (issuer, audience). We treat `sub` as the canonical external_id and ignore claim instability.

**§11.20** The `cyberos_app` UPDATE grants on `portal_idp_configs` (rotated_at, status) and `portal_scim_tokens` (rotated_at, status) are intentionally narrow — only the rotation handler can update these columns.

**§11.21** Per-Engagement test fixtures use `wiremock` to simulate IdP endpoints; integration tests against real Okta/Azure dev tenants run nightly (not on every PR) to validate protocol-level conformance.

**§11.22** The 7-kind core list (§1 #20) + 1 supporting (`portal.scim_token_rotation`) totals 8 memory audit kinds; TASK-AI-003 closed-set extension adds all 8.

**§11.23** Trace_id in memory rows uses the OTel TraceId Display form (32-char lower-hex) per task-audit skill rule 24; never Debug form.

**§11.24** Slice-2 enhancement: emit OTel histogram per-IdP `portal_idp_sign_in_duration_seconds_by_idp_entity_id` for per-customer latency analysis.

**§11.25** SAML SP-metadata is regenerated on every request (not cached) because per-Engagement SP cert may rotate; cache invalidation complexity > generation cost.

**§11.26** OIDC nonce + state are stored in Redis with 5-min TTL; one-shot consumption + auto-eviction.

---

*End of TASK-PORTAL-003 spec.*
