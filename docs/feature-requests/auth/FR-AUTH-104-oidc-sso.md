---
id: FR-AUTH-104
title: "AUTH OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + per-tenant IdP config + PKCE + JIT subject provisioning + claim → role mapping"
module: AUTH
priority: MUST
status: ready_to_test
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-AUTH-002, FR-AUTH-004, FR-AUTH-101, FR-AI-003, FR-MEMORY-101, FR-AUTH-103, FR-TEN-101, FR-PORTAL-003]
depends_on: [FR-AUTH-004, FR-AUTH-101]
blocks: [FR-TEN-101, FR-PORTAL-003]

source_pages:
  - website/docs/modules/auth.html#sso
source_decisions:
  - DEC-400 (OIDC discovery via RFC 8414 well-known endpoint — `/.well-known/openid-configuration`; cached per tenant with 1h TTL + signature-key cache 24h)
  - DEC-401 (Authorisation Code flow + PKCE (RFC 7636) mandatory; implicit flow + hybrid flow forbidden)
  - DEC-402 (JWKS fetched per RFC 7517; cached 24h with kid-based rotation — old kids accepted for 24h overlap; new kids accepted immediately)
  - DEC-403 (JIT (Just-In-Time) subject provisioning — first OIDC login auto-creates AUTH subject via FR-AUTH-002; claim → role mapping per per-tenant config)
  - DEC-404 (closed claim-mapping ruleset: `groups | roles | role | scope` claim names supported; mapping rules per-tenant YAML; unknown claim → log + ignore)
  - DEC-405 (per-tenant IdP config stored in `auth_oidc_idp_configs` table; secrets KMS-encrypted)
  - DEC-406 (memory audit kinds: auth.oidc_login_succeeded, auth.oidc_login_failed, auth.oidc_jit_provisioned, auth.oidc_jwks_rotated, auth.oidc_idp_config_changed, auth.oidc_signature_invalid)
  - DEC-407 (REVOKE UPDATE, DELETE on auth_oidc_login_history from cyberos_app — append-only at SQL grant)
  - DEC-408 (state + nonce verification mandatory — replay attacks rejected; state TTL 10 minutes)
  - DEC-409 (login flow timeout 5 minutes total — initiation → callback; expired → reject with `oidc_flow_expired`)
  - DEC-410 (sub claim uniqueness per (tenant_id, idp_id) — same sub from different IdPs are different subjects; cross-IdP linking is opt-in per slice 3)
  - DEC-411 (id_token signature MUST validate against JWKS; aud + iss + exp + nbf all checked; clock skew tolerance 60s)
  - DEC-412 (per-tenant max 3 active IdP configs at slice 1; ADR required to raise — keeps the consent + discovery UX manageable)
  - RFC 6749 (OAuth 2.0); RFC 7636 (PKCE); RFC 7515/7517/7518 (JWT/JWS/JWA + JWKS); RFC 8414 (OAuth metadata); OIDC Core 1.0 spec
  - PDPL Art. 13 (data minimisation — claim payload PII-scrubbed in memory chain)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0010_oidc_idp_configs.sql
  - services/auth/migrations/0011_oidc_login_history.sql
  - services/auth/migrations/0012_oidc_subject_link.sql
  - services/auth/src/oidc/mod.rs
  - services/auth/src/oidc/discovery.rs                    # RFC 8414 well-known fetch + cache
  - services/auth/src/oidc/jwks.rs                          # RFC 7517 JWKS fetch + rotation + 24h overlap
  - services/auth/src/oidc/flow.rs                          # initiate + callback (PKCE Authorisation Code)
  - services/auth/src/oidc/state.rs                         # state + nonce generation + TTL store
  - services/auth/src/oidc/id_token.rs                      # signature verify + claim validation
  - services/auth/src/oidc/claim_mapper.rs                  # claim → AUTH role mapping per tenant config
  - services/auth/src/oidc/jit_provision.rs                 # first-login JIT subject creation
  - services/auth/src/oidc/repo.rs                          # idp_configs + login_history + subject_link CRUD
  - services/auth/src/oidc/audit.rs                         # 6 memory row builders
  - services/auth/src/oidc/errors.rs                        # closed error enum
  - services/auth/src/handlers/oidc.rs                      # GET /v1/auth/oidc/initiate + GET /v1/auth/oidc/callback + POST /v1/auth/oidc/idp-configs
  - services/auth/tests/middleware_test.rs
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/geoip_test.rs
  - services/auth/tests/rbac_catalogue_test.rs
  - services/auth/tests/rbac_adr_gate_test.rs
  - services/auth/tests/oidc_claim_mapping_test.rs
  - services/auth/tests/oidc_jit_provision_test.rs
  - services/auth/tests/geoip_test.rs
  - services/auth/tests/oidc_implicit_forbidden_test.rs
  - services/auth/tests/oidc_audit_emission_test.rs
  - services/auth/tests/oidc_replay_attack_test.rs
  - services/auth/tests/geoip_test.rs
modified_files:
  - services/auth/src/lib.rs                                # pub mod oidc

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test oidc

disallowed_tools:
  - implement implicit flow or hybrid flow (per DEC-401)
  - skip state + nonce verification (per DEC-408)
  - skip PKCE (per DEC-401)
  - skip id_token signature verification (per DEC-411)
  - allow > 3 IdP configs per tenant without ADR (per DEC-412)
  - allow cross-IdP subject linking at slice 1 (per DEC-410)
  - log raw id_token to file (chain holds scrubbed)

effort_hours: 10
sub_tasks:
  - "0.5h: 0010_oidc_idp_configs.sql — per-tenant IdP configs + KMS-encrypted secrets"
  - "0.4h: 0011_oidc_login_history.sql — append-only login history"
  - "0.4h: 0012_oidc_subject_link.sql — sub → subject_id mapping"
  - "0.5h: discovery.rs — RFC 8414 fetch + 1h TTL"
  - "1.0h: jwks.rs — RFC 7517 + 24h cache + rotation overlap"
  - "1.2h: flow.rs — initiate + callback (PKCE Auth Code)"
  - "0.5h: state.rs — state + nonce generation"
  - "1.0h: id_token.rs — signature verify + claim validation"
  - "0.7h: claim_mapper.rs — per-tenant YAML mapping"
  - "0.8h: jit_provision.rs — first-login JIT subject creation"
  - "0.4h: repo.rs"
  - "0.4h: audit.rs — 6 row builders"
  - "0.6h: handlers/oidc.rs — REST surface"
  - "2.6h: tests — 12 test files"

risk_if_skipped: "Without OIDC SSO, every tenant employee provisions an AUTH subject manually (password reset friction + onboarding delay). FR-TEN-101's self-serve signup needs OIDC for federated identity at signup. FR-PORTAL-003's external IdP integration with JIT user provisioning is the canonical enterprise-tenant entry pattern. Without DEC-401's PKCE requirement, the authorization-code interception attack is open; without DEC-402's JWKS rotation, IdP key rotation breaks login until manual rebuild; without DEC-408's state + nonce verification, CSRF + replay attacks succeed; without DEC-411's id_token signature verification, attacker-crafted tokens grant access. The 10h effort lands the standards-compliant SSO with all the security guarantees the spec mandates."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship OIDC SSO with RFC 8414 discovery + RFC 7517 JWKS rotation + PKCE Authorisation Code flow + per-tenant IdP config + JIT subject provisioning + claim → role mapping. Each requirement:

1. **MUST** define `auth_oidc_idp_configs` table: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, name TEXT NOT NULL, issuer_url TEXT NOT NULL, client_id TEXT NOT NULL, client_secret_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, redirect_uri TEXT NOT NULL, claim_mapping_yaml TEXT NOT NULL, is_active BOOLEAN NOT NULL DEFAULT true, created_at TIMESTAMPTZ, created_by_subject_id UUID NOT NULL)`. UNIQUE `(tenant_id, name)`; partial unique `(tenant_id, issuer_url) WHERE is_active=true`. Max 3 active per tenant (handler check per DEC-412).

2. **MUST** define `auth_oidc_login_history` table: `(id BIGSERIAL, tenant_id UUID, idp_id UUID, subject_id UUID (nullable until JIT), sub_claim TEXT NOT NULL, outcome TEXT NOT NULL CHECK (outcome IN ('succeeded','failed','jit_provisioned')), failure_reason TEXT, source_ip_hash16 TEXT, ts TIMESTAMPTZ)`. `REVOKE UPDATE, DELETE FROM cyberos_app`.

3. **MUST** define `auth_oidc_subject_link` table: `(idp_id UUID, sub_claim TEXT, tenant_id UUID, subject_id UUID NOT NULL REFERENCES auth.subjects(id), linked_at TIMESTAMPTZ, PRIMARY KEY (idp_id, sub_claim))`. Per DEC-410, sub uniqueness is per (idp_id, tenant_id); cross-IdP linking is opt-in at slice 3.

4. **MUST** enforce RLS with both `USING` and `WITH CHECK` on all 3 OIDC tables. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`. Discovery + JWKS fetches at gateway are tenant-scoped via the IdP config lookup.

5. **MUST** implement RFC 8414 discovery (per DEC-400). Fetch `<issuer>/.well-known/openid-configuration` once per tenant + cache 1h. Parse `authorization_endpoint`, `token_endpoint`, `jwks_uri`, `issuer`, `response_types_supported`, `grant_types_supported`, `code_challenge_methods_supported`. Validate: `code` in response_types, `authorization_code` in grant_types, `S256` in code_challenge_methods. Missing → reject IdP config with `oidc_pkce_unsupported`.

6. **MUST** implement RFC 7517 JWKS fetch + rotation (per DEC-402). Fetch `jwks_uri` once per kid + cache 24h. On id_token with unknown kid → refetch JWKS; if still unknown → 401 `unknown_kid`. Old kids accepted for 24h overlap (graceful rotation); new kids accepted immediately.

7. **MUST** implement the Authorisation Code flow with PKCE (RFC 7636 + DEC-401):
   - `GET /v1/auth/oidc/initiate?idp_id=<>&tenant_slug=<>` — generates `state` (32 random bytes hex) + `nonce` (32 random bytes hex) + `code_verifier` (43-128 char base64url) + `code_challenge = SHA-256(code_verifier)`; stores `(state, nonce, code_verifier, idp_id, tenant_id, expires_at = now + 10min)` in Redis; redirects to `authorization_endpoint?response_type=code&client_id=<>&redirect_uri=<>&scope=openid profile email&state=<>&nonce=<>&code_challenge=<>&code_challenge_method=S256`.
   - `GET /v1/auth/oidc/callback?code=<>&state=<>` — looks up `state` in Redis; expired or missing → 401 `oidc_flow_expired`; validates `state` + recovers `code_verifier`; POSTs to `token_endpoint` with `grant_type=authorization_code&code=<>&client_id=<>&client_secret=<>&redirect_uri=<>&code_verifier=<>`; receives `id_token`; verifies per §1 #8; provisions JIT subject per §1 #9; issues FR-AUTH-004 JWT; redirects to tenant SPA.

7. **MUST** validate id_token signature + claims (per DEC-411 + §1 #8):
    - Signature: HS/RS/ES algorithms per JWKS `alg`; verify against the matching `kid`.
    - `iss` claim MUST equal `idp_config.issuer_url`.
    - `aud` claim MUST contain `client_id`.
    - `exp` claim MUST be in the future (60s skew tolerance).
    - `nbf` claim MUST be in the past (60s skew).
    - `nonce` claim MUST match the stored nonce.
    - Any fail → 401 + emit `auth.oidc_login_failed` + `auth.oidc_signature_invalid` memory rows.

8. **MUST** JIT-provision a subject on first login (per DEC-403). If `auth_oidc_subject_link WHERE idp_id=$1 AND sub_claim=$2` returns nothing:
    - Extract `email` claim (fall back to `preferred_username` if email absent).
    - Call `FR-AUTH-002`'s `create_subject` internal helper with `email`, JIT-generated password (zeroised; subject MUST set password reset on first session OR rely on SSO subsequently), default role per claim mapping (§1 #10).
    - INSERT `auth_oidc_subject_link` row.
    - Emit `auth.oidc_jit_provisioned` memory row.

9. **MUST** apply per-tenant claim → role mapping (per DEC-404). The IdP config's `claim_mapping_yaml` is a closed-shape YAML:
   ```yaml
   default_role: tenant-member
   claim_rules:
     - claim: groups        # one of: groups | roles | role | scope
       contains: "Engineering"
       grant_role: tenant-admin
     - claim: groups
       contains: "Finance"
       grant_role: cfo      # per FR-AUTH-101 enum
   ```
   Rules evaluated top-down; first match wins. Unmatched claim values → use `default_role`. Unknown role in mapping → log + fall back to `default_role`.

10. **MUST** enforce the closed FR-AUTH-101 role enum on JIT provisioning. claim_mapping_yaml referencing a role not in the catalogue → IdP config save fails with `unknown_role: <name>`.

11. **MUST** support state + nonce replay protection (per DEC-408). Redis stores each `state` with 10-minute TTL; reuse → 401 `oidc_state_reused`; expired → 401 `oidc_flow_expired`. Nonce checked against id_token.

12. **MUST** rotate JWKS via 24-hour key-cache overlap (per DEC-402). On id_token with unknown `kid`, refetch JWKS once; emit `auth.oidc_jwks_rotated` memory row when new kid first observed.

13. **MUST** enforce a per-tenant max of 3 active IdP configs (per DEC-412). Attempt to create a 4th → 409 `idp_config_limit_exceeded`. ADR required to raise the cap.

14. **MUST** emit memory audit rows for 6 kinds (per DEC-406):
    - `auth.oidc_login_succeeded` (full login flow → AUTH JWT issued).
    - `auth.oidc_login_failed` (any callback failure).
    - `auth.oidc_jit_provisioned` (new subject created on first login).
    - `auth.oidc_jwks_rotated` (new kid observed in JWKS).
    - `auth.oidc_idp_config_changed` (POST/PATCH/DELETE on idp_configs).
    - `auth.oidc_signature_invalid` (id_token verification failed; sev-2 — may signal attack).

15. **MUST** PII-scrub `sub_claim`, `email`, `failure_reason` via FR-MEMORY-111 before chain commit.

16. **MUST** complete the OIDC callback handler in ≤ 500 ms p95 (most latency is IdP token endpoint + JWKS fetch; PKCE + claim validation < 50 ms server-side). `oidc_perf_test`.

17. **MUST** emit OTel span `auth.oidc.{initiate,callback,jit_provision,idp_config_change}` with `outcome` attribute (success | jit_provisioned | state_reused | flow_expired | signature_invalid | sub_mismatch | unknown_kid | idp_unreachable | claim_mapping_unknown_role).

18. **MUST** emit OTel metrics:
    - `auth_oidc_login_total{tenant_id, idp_id, outcome}` (counter).
    - `auth_oidc_jit_provisioned_total{tenant_id}` (counter).
    - `auth_oidc_jwks_rotations_total{tenant_id, idp_id}` (counter).
    - `auth_oidc_signature_failures_total{tenant_id, idp_id}` (counter — sev-2 if sustained > 5/h).
    - `auth_oidc_flow_latency_ms` (histogram; SLO p95 < 500ms).

19. **MUST** ship `POST /v1/auth/oidc/idp-configs` for tenant-admin IdP config CRUD. Caller MUST have role `tenant-admin` per FR-AUTH-101. Validates claim_mapping_yaml + roles + discovery succeeds before persisting.

20. **MUST** support `PATCH /v1/auth/oidc/idp-configs/{id}` (update non-immutable fields) + `DELETE` (soft-delete via `is_active=false`); deletion of active IdP with linked subjects requires explicit confirm header `X-Confirm-Subject-Deactivate: yes`.

21. **MUST** reject implicit + hybrid flow (per DEC-401). The initiate handler hard-codes `response_type=code`; the IdP config validator rejects discovery payloads missing `code` in `response_types_supported`.

22. **MUST** ensure 60-second clock skew tolerance on exp + nbf claims (per DEC-411 + §1 #8). Wider tolerance → replay risk; tighter → false-reject of legitimate tokens.

23. **MUST** support the standard scopes: `openid` (required), `profile`, `email`, plus per-IdP custom scopes specified in idp_config. Unknown IdP-claimed scopes → ignore (not an error).

24. **MUST** validate sub_claim uniqueness per (idp_id, tenant_id) (per DEC-410). Same sub appearing twice for different subject_id values → 409 `oidc_sub_already_linked`. Cross-IdP same sub → distinct subjects (DEC-410).

25. **MUST** emit `auth.oidc_idp_config_changed` memory row with the diff (which fields changed) BUT NOT the new client_secret value (PII-grade secret).

26. **MUST** alarm at sev-2 on sustained signature failures > 5/h per tenant (per §1 #18 metric) — suggests IdP key rotation issue OR attack attempt.

---

## §2 — Why this design (rationale for humans)

**Why RFC 8414 discovery (DEC-400, §1 #5)?** Discovery is the well-known endpoint that publishes the IdP's metadata (token endpoint, JWKS URI, supported algs). Without it, every IdP integration becomes manual config. RFC 8414 standardises the shape; OIDC providers publish at `/.well-known/openid-configuration`. Caching 1h limits hot-path latency without losing rotation freshness.

**Why PKCE mandatory (DEC-401, §1 #6)?** Authorization-code interception is the classic OAuth attack — attacker intercepts the redirect_uri callback containing `code`, exchanges it for token. PKCE binds the code to the original initiator via `code_verifier` (only the initiator knows the verifier matching the published code_challenge). RFC 7636 mandatory for public clients; we use it for all clients (defense in depth).

**Why implicit + hybrid flow forbidden (DEC-401, §1 #21)?** Implicit flow returns id_token in the URL fragment — leaks via browser history, referrer headers. Hybrid flow has similar leakage. OAuth 2.1 (draft) deprecates both. Standard auth code with PKCE is the canonical secure pattern.

**Why JWKS 24h cache + overlap (DEC-402, §1 #6, §1 #12)?** IdPs rotate signing keys periodically. The 24h cache avoids refetching JWKS on every token verification (cost). Overlap means old kids (still in cache) AND new kids (refetched on unknown-kid) both work — graceful rotation. Without overlap, key rotation = forced re-auth for everyone.

**Why state + nonce mandatory (DEC-408, §1 #11)?** State prevents CSRF (attacker initiates flow + tricks victim to complete callback with attacker's code). Nonce prevents id_token replay (same token can't be reused across login attempts). Both are spec-required; making them ours-required closes the gap if IdP misimplements.

**Why 10-minute state TTL (DEC-409)?** Login flow should complete within seconds; 10 minutes covers slow IdPs + user delay (e.g. MFA prompt). Past 10 minutes the user has abandoned — force re-initiate.

**Why JIT provisioning (DEC-403, §1 #9)?** Enterprise SSO works only if user provisioning is automatic. First login → IdP authenticates → AUTH creates the subject from id_token claims. The alternative (require pre-provisioning via admin) defeats the SSO value prop. JIT keeps user lifecycle synced with IdP.

**Why claim → role mapping per-tenant YAML (DEC-404, §1 #10)?** Different tenants use different IdP claim shapes. Okta uses `groups`; Azure AD uses `roles`; Google Workspace uses scope claims. Per-tenant YAML lets each tenant configure their mapping without code changes. Closed-shape YAML (4 supported claim names) prevents drift; explicit `default_role` covers unmatched cases.

**Why max 3 active IdP configs per tenant (DEC-412, §1 #13)?** Each active IdP shows up in the login selector UI; > 3 becomes a UX problem. Multi-IdP scenarios (employee + contractor + partner) typically fit ≤ 3. ADR-raise forces operator consideration of UX cost.

**Why per-(idp_id, sub_claim) uniqueness, not global sub (DEC-410, §1 #24)?** Same `sub` value from different IdPs are different users (e.g. Google `sub=12345` and Okta `sub=12345` are unrelated). Scoping uniqueness per IdP prevents accidental cross-linking. Cross-IdP linking (same human via two IdPs) is opt-in at slice 3.

**Why append-only login_history (DEC-407)?** Login attempts (success + fail) are forensic events. UPDATE/DELETE blocked at SQL grant prevents post-hoc rewriting of "who logged in when".

**Why 60s clock skew tolerance (DEC-411, §1 #22)?** NTP drift between IdP + our server is typically < 1s but can spike to ~10-30s on misconfigured systems. 60s is generous + below replay window. Tighter would false-reject legitimate tokens.

**Why sub_claim PII-scrubbed in memory chain (§1 #15)?** sub is typically an opaque IdP-assigned id (e.g. UUID), but some IdPs use email-derived subs. Scrubbing prevents inadvertent PII in audit chain.

**Why client_secret KMS-encrypted (§1 #1)?** Client secret authenticates the AUTH service to the IdP. Plaintext at rest = compromise on DB dump. KMS encryption requires KMS key permissions for decrypt — meaningful additional barrier.

**Why redirect_uri in IdP config not request-supplied (§1 #1, §1 #6)?** Redirect_uri is a per-tenant fixed value (e.g. `https://auth.cyberos.world/v1/auth/oidc/callback`). Allowing request-supplied redirects opens open-redirect attacks. Config-locked redirect prevents.

**Why JIT-generated password (subject MUST reset OR rely on SSO) (§1 #9)?** Subject needs a password hash to satisfy FR-AUTH-002's schema. JIT generates a strong random password (zeroised immediately); subject either (a) sets a real password via reset flow if SSO becomes unavailable, OR (b) rely on SSO indefinitely. Common case = (b).

**Why sev-2 alarm on signature failures > 5/h (§1 #26)?** Normal operation produces zero signature failures. A burst means IdP key rotation not propagated OR active attack (forged tokens). Sev-2 = operator investigation.

**Why `claim_mapping_yaml` rather than JSON (§1 #10)?** YAML is human-readable; operators edit by hand at slice 1 (slice 3 may add UI). YAML supports comments — operators can document why a mapping rule exists.

**Why slice 1 doesn't ship cross-IdP linking (DEC-410)?** Cross-IdP same-human resolution requires identity-matching logic (email match? phone match?) — substantial. Slice 1 keeps it simple: same human via two IdPs = two AUTH subjects. Slice 3 ships the link flow.

---

## §3 — API contract

### 3.1 — Migration 0010 — idp_configs

```sql
-- services/auth/migrations/0010_oidc_idp_configs.sql

BEGIN;

CREATE TABLE auth_oidc_idp_configs (
    id                      UUID         PRIMARY KEY,
    tenant_id               UUID         NOT NULL,
    name                    TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    issuer_url              TEXT         NOT NULL,
    client_id               TEXT         NOT NULL,
    client_secret_kms_blob  BYTEA        NOT NULL,
    kms_key_id              TEXT         NOT NULL,
    redirect_uri            TEXT         NOT NULL,
    claim_mapping_yaml      TEXT         NOT NULL,
    is_active               BOOLEAN      NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id   UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX uniq_idp_name ON auth_oidc_idp_configs (tenant_id, name);
CREATE UNIQUE INDEX uniq_active_idp_issuer ON auth_oidc_idp_configs (tenant_id, issuer_url) WHERE is_active = true;

ALTER TABLE auth_oidc_idp_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY auth_oidc_idp_configs_tenant_iso ON auth_oidc_idp_configs
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.2 — Migration 0011 — login_history

```sql
-- services/auth/migrations/0011_oidc_login_history.sql

BEGIN;

CREATE TABLE auth_oidc_login_history (
    id                  BIGSERIAL    PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    idp_id              UUID         NOT NULL REFERENCES auth_oidc_idp_configs(id),
    subject_id          UUID         REFERENCES auth.subjects(id),
    sub_claim           TEXT         NOT NULL,
    outcome             TEXT         NOT NULL CHECK (outcome IN ('succeeded','failed','jit_provisioned')),
    failure_reason      TEXT,
    source_ip_hash16    TEXT,
    ts                  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX login_history_tenant_ts_idx ON auth_oidc_login_history (tenant_id, ts DESC);
CREATE INDEX login_history_subject_idx ON auth_oidc_login_history (subject_id) WHERE subject_id IS NOT NULL;

ALTER TABLE auth_oidc_login_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY login_history_tenant_iso ON auth_oidc_login_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON auth_oidc_login_history FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0012 — subject_link

```sql
-- services/auth/migrations/0012_oidc_subject_link.sql

BEGIN;

CREATE TABLE auth_oidc_subject_link (
    idp_id       UUID         NOT NULL REFERENCES auth_oidc_idp_configs(id) ON DELETE RESTRICT,
    sub_claim    TEXT         NOT NULL,
    tenant_id    UUID         NOT NULL,
    subject_id   UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    linked_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (idp_id, sub_claim)
);

CREATE INDEX subject_link_subject_idx ON auth_oidc_subject_link (tenant_id, subject_id);

ALTER TABLE auth_oidc_subject_link ENABLE ROW LEVEL SECURITY;
CREATE POLICY subject_link_tenant_iso ON auth_oidc_subject_link
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.4 — Discovery + JWKS

```rust
// services/auth/src/oidc/discovery.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    #[serde(default)]
    pub response_types_supported: Vec<String>,
    #[serde(default)]
    pub grant_types_supported: Vec<String>,
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
}

pub async fn fetch_discovery(issuer_url: &str) -> anyhow::Result<OidcDiscovery> {
    let url = format!("{issuer_url}/.well-known/openid-configuration");
    let resp = reqwest::get(&url).await?.error_for_status()?;
    let disco: OidcDiscovery = resp.json().await?;
    validate_pkce_support(&disco)?;
    Ok(disco)
}

fn validate_pkce_support(disco: &OidcDiscovery) -> anyhow::Result<()> {
    if !disco.response_types_supported.iter().any(|t| t == "code") {
        anyhow::bail!("oidc_response_type_code_not_supported");
    }
    if !disco.grant_types_supported.iter().any(|g| g == "authorization_code") {
        anyhow::bail!("oidc_auth_code_grant_not_supported");
    }
    if !disco.code_challenge_methods_supported.iter().any(|m| m == "S256") {
        anyhow::bail!("oidc_pkce_s256_unsupported");
    }
    Ok(())
}
```

```rust
// services/auth/src/oidc/jwks.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    pub alg: String,
    pub n: Option<String>,    // RSA modulus
    pub e: Option<String>,    // RSA exponent
    pub crv: Option<String>,  // EC curve
    pub x: Option<String>,    // EC x
    pub y: Option<String>,    // EC y
    pub k: Option<String>,    // HMAC key (base64url)
}

pub struct JwksCache {
    inner: Arc<ArcSwap<HashMap<String, Jwk>>>,   // kid → Jwk
}

impl JwksCache {
    pub fn new() -> Self {
        Self { inner: Arc::new(ArcSwap::from_pointee(HashMap::new())) }
    }

    pub async fn refetch(&self, jwks_uri: &str) -> anyhow::Result<()> {
        #[derive(Deserialize)] struct JwksDoc { keys: Vec<Jwk> }
        let doc: JwksDoc = reqwest::get(jwks_uri).await?.error_for_status()?.json().await?;
        let map: HashMap<String, Jwk> = doc.keys.into_iter().map(|k| (k.kid.clone(), k)).collect();
        self.inner.store(Arc::new(map));
        Ok(())
    }

    pub fn get(&self, kid: &str) -> Option<Jwk> {
        self.inner.load().get(kid).cloned()
    }
}
```

### 3.5 — Authorisation code + PKCE flow

```rust
// services/auth/src/oidc/flow.rs
use rand::RngCore;
use sha2::{Digest, Sha256};
use base64::{Engine, engine::general_purpose};

#[derive(Debug, Clone)]
pub struct FlowState {
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    pub idp_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub fn generate_flow_state(idp_id: uuid::Uuid, tenant_id: uuid::Uuid) -> FlowState {
    let mut rng = rand::thread_rng();
    let state = random_hex(32, &mut rng);
    let nonce = random_hex(32, &mut rng);
    let mut verifier_bytes = [0u8; 32];
    rng.fill_bytes(&mut verifier_bytes);
    let code_verifier = general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);
    FlowState {
        state, nonce, code_verifier, idp_id, tenant_id,
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(10),
    }
}

pub fn code_challenge_from_verifier(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

fn random_hex(n_bytes: usize, rng: &mut impl RngCore) -> String {
    let mut buf = vec![0u8; n_bytes];
    rng.fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn build_auth_url(disco: &super::discovery::OidcDiscovery, idp: &IdpConfig, state: &FlowState) -> String {
    let challenge = code_challenge_from_verifier(&state.code_verifier);
    format!(
        "{auth}?response_type=code&client_id={cid}&redirect_uri={ru}&scope={scope}&state={s}&nonce={n}&code_challenge={c}&code_challenge_method=S256",
        auth = disco.authorization_endpoint,
        cid = urlencoding::encode(&idp.client_id),
        ru = urlencoding::encode(&idp.redirect_uri),
        scope = urlencoding::encode("openid profile email"),
        s = state.state, n = state.nonce, c = challenge,
    )
}
```

### 3.6 — id_token verification

```rust
// services/auth/src/oidc/id_token.rs
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header, decode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: serde_json::Value,    // string or array
    pub exp: i64,
    pub nbf: Option<i64>,
    pub iat: i64,
    pub nonce: Option<String>,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub groups: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub role: Option<String>,
    pub scope: Option<String>,
}

pub fn verify_id_token(
    token: &str,
    jwks: &super::jwks::JwksCache,
    expected_issuer: &str,
    expected_client_id: &str,
    expected_nonce: &str,
) -> Result<IdTokenClaims, OidcError> {
    let header = decode_header(token).map_err(|_| OidcError::TokenMalformed)?;
    let kid = header.kid.as_deref().ok_or(OidcError::TokenKidMissing)?;
    let jwk = jwks.get(kid).ok_or(OidcError::UnknownKid)?;

    let alg = match jwk.alg.as_str() {
        "RS256" => Algorithm::RS256, "ES256" => Algorithm::ES256,
        _ => return Err(OidcError::AlgorithmUnsupported(jwk.alg)),
    };
    let key = make_decoding_key(&jwk, alg)?;
    let mut validation = Validation::new(alg);
    validation.set_issuer(&[expected_issuer]);
    validation.set_audience(&[expected_client_id]);
    validation.leeway = 60;   // 60-second clock skew per DEC-411

    let decoded = decode::<IdTokenClaims>(token, &key, &validation)
        .map_err(|e| OidcError::IdTokenValidation(format!("{e:?}")))?;

    if decoded.claims.nonce.as_deref() != Some(expected_nonce) {
        return Err(OidcError::NonceMismatch);
    }
    Ok(decoded.claims)
}

fn make_decoding_key(jwk: &super::jwks::Jwk, alg: Algorithm) -> Result<DecodingKey, OidcError> {
    match (jwk.kty.as_str(), alg) {
        ("RSA", Algorithm::RS256) => {
            let n = jwk.n.as_deref().ok_or(OidcError::JwkMalformed)?;
            let e = jwk.e.as_deref().ok_or(OidcError::JwkMalformed)?;
            DecodingKey::from_rsa_components(n, e).map_err(|_| OidcError::JwkMalformed)
        }
        ("EC", Algorithm::ES256) => {
            let x = jwk.x.as_deref().ok_or(OidcError::JwkMalformed)?;
            let y = jwk.y.as_deref().ok_or(OidcError::JwkMalformed)?;
            DecodingKey::from_ec_components(x, y).map_err(|_| OidcError::JwkMalformed)
        }
        _ => Err(OidcError::JwkKtyAlgMismatch),
    }
}
```

### 3.7 — Claim mapper

```rust
// services/auth/src/oidc/claim_mapper.rs
use serde::Deserialize;
use cyberos_auth::rbac::Role;

#[derive(Debug, Deserialize)]
pub struct ClaimMappingConfig {
    pub default_role: String,
    pub claim_rules: Vec<ClaimRule>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimRule {
    pub claim: ClaimName,          // groups | roles | role | scope
    pub contains: String,
    pub grant_role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimName { Groups, Roles, Role, Scope }

pub fn parse_config(yaml: &str) -> anyhow::Result<ClaimMappingConfig> {
    let cfg: ClaimMappingConfig = serde_yaml::from_str(yaml)?;
    // Validate all referenced roles against FR-AUTH-101 closed enum
    cfg.default_role.parse::<Role>()
        .map_err(|_| anyhow::anyhow!("unknown_role: {}", cfg.default_role))?;
    for rule in &cfg.claim_rules {
        rule.grant_role.parse::<Role>()
            .map_err(|_| anyhow::anyhow!("unknown_role: {}", rule.grant_role))?;
    }
    Ok(cfg)
}

pub fn resolve_role(cfg: &ClaimMappingConfig, claims: &super::id_token::IdTokenClaims) -> Role {
    for rule in &cfg.claim_rules {
        let claim_values: Vec<String> = match rule.claim {
            ClaimName::Groups => claims.groups.clone().unwrap_or_default(),
            ClaimName::Roles  => claims.roles.clone().unwrap_or_default(),
            ClaimName::Role   => claims.role.clone().map(|r| vec![r]).unwrap_or_default(),
            ClaimName::Scope  => claims.scope.clone().map(|s| s.split(' ').map(String::from).collect()).unwrap_or_default(),
        };
        if claim_values.iter().any(|v| v == &rule.contains) {
            if let Ok(role) = rule.grant_role.parse::<Role>() {
                return role;
            }
        }
    }
    cfg.default_role.parse::<Role>().expect("validated at parse_config")
}
```

---

## §4 — Acceptance criteria

1. **Discovery fetch + cache** — fresh fetch on first IdP config; cached 1h subsequent.
2. **PKCE S256 challenge generated** — code_verifier 43-128 chars; challenge = base64url(SHA256(verifier)).
3. **Implicit flow IdP config rejected** — discovery missing `code` in response_types → `oidc_response_type_code_not_supported`.
4. **Hybrid flow rejected** — same path.
5. **Authorisation URL contains required params** — response_type, client_id, redirect_uri, scope, state, nonce, code_challenge, code_challenge_method=S256.
6. **State + nonce stored 10-min TTL** — Redis lookup recovers code_verifier within window.
7. **Expired state rejected** — past TTL → 401 `oidc_flow_expired`.
8. **Reused state rejected** — second callback with same state → 401 `oidc_state_reused`.
9. **id_token signature verified** — happy path: 200; invalid sig: 401 + `auth.oidc_signature_invalid` memory row.
10. **Unknown kid triggers JWKS refetch** — JWKS cache updates; `auth.oidc_jwks_rotated` row emitted.
11. **iss mismatch rejected** — 401 `oidc_iss_mismatch`.
12. **aud mismatch rejected** — 401 `oidc_aud_mismatch`.
13. **nonce mismatch rejected** — 401 `oidc_nonce_mismatch`.
14. **exp expired (past + 60s skew)** → 401 `oidc_expired`.
15. **JIT provisioning on first login** — new sub → subject created + link row; `auth.oidc_jit_provisioned` memory row.
16. **JIT skipped on repeat login** — existing link → subject reused.
17. **Claim mapping grants role** — groups: ["Engineering"] → tenant-admin per config.
18. **Default role used on no match** — claim mapping has no rule matching → default_role applied.
19. **Unknown role in claim_mapping_yaml rejected at config save** — `unknown_role: <name>` 400.
20. **Max 3 IdP configs per tenant** — 4th create → 409 `idp_config_limit_exceeded`.
21. **append-only login_history** — UPDATE/DELETE from cyberos_app rejected.
22. **client_secret KMS-encrypted at rest** — DB row contains BYTEA blob, not plaintext.
23. **redirect_uri locked in config** — request-supplied redirect_uri ignored.
24. **OIDC callback p95 < 500 ms** — `oidc_perf_test`.
25. **5/h signature failure alarm** — OBS fires sev-2.
26. **OTel span `auth.oidc.callback` emitted** — outcome attr.
27. **Counter `auth_oidc_login_total{outcome=succeeded}` increments**.

---

## §5 — Verification

```rust
// services/auth/tests/geoip_test.rs
#[tokio::test]
async fn pkce_full_flow(ctx: TestCtx) {
    let idp = ctx.seed_idp_config_with_mock_provider().await;
    // 1. Initiate
    let init_resp = ctx.get(&format!("/v1/auth/oidc/initiate?idp_id={}&tenant_slug=acme", idp.id)).await;
    let auth_url = init_resp.headers()["Location"].to_str().unwrap();
    assert!(auth_url.contains("response_type=code"));
    assert!(auth_url.contains("code_challenge="));
    assert!(auth_url.contains("code_challenge_method=S256"));
    // 2. Simulate IdP redirect with code
    let state_param = extract_query_param(auth_url, "state");
    ctx.mock_idp_provider().expect_token_exchange().return_id_token(/* signed by mock JWKS */);
    let cb_resp = ctx.get(&format!("/v1/auth/oidc/callback?code=mock_code&state={state_param}")).await;
    assert_eq!(cb_resp.status(), 302);
    // 3. Subject created (JIT)
    let subjects = ctx.list_subjects().await;
    assert_eq!(subjects.len(), 1);
    let rows = ctx.memory_audit_rows("auth.oidc_jit_provisioned").await;
    assert_eq!(rows.len(), 1);
}
```

```rust
// services/auth/tests/rbac_catalogue_test.rs
#[tokio::test]
async fn reused_state_rejected(ctx: TestCtx) {
    let idp = ctx.seed_idp_config().await;
    let state = ctx.initiate_and_get_state(idp.id).await;
    let _first = ctx.complete_callback(&state, "code1").await.unwrap();
    let second = ctx.complete_callback(&state, "code2").await.unwrap_err();
    assert!(format!("{second:?}").contains("oidc_state_reused"));
}

#[tokio::test]
async fn expired_state_rejected(ctx: TestCtx) {
    let idp = ctx.seed_idp_config().await;
    let state = ctx.initiate_and_get_state(idp.id).await;
    ctx.advance_clock_minutes(11).await;
    let err = ctx.complete_callback(&state, "code1").await.unwrap_err();
    assert!(format!("{err:?}").contains("oidc_flow_expired"));
}
```

```rust
// services/auth/tests/oidc_claim_mapping_test.rs
#[test]
fn unknown_role_in_yaml_rejected() {
    let yaml = "default_role: tenant-member\nclaim_rules:\n  - claim: groups\n    contains: Eng\n    grant_role: super-admin\n";
    let err = cyberos_auth::oidc::claim_mapper::parse_config(yaml).unwrap_err();
    assert!(format!("{err}").contains("unknown_role: super-admin"));
}

#[test]
fn groups_claim_maps_to_role() {
    let yaml = "default_role: tenant-member\nclaim_rules:\n  - claim: groups\n    contains: Engineering\n    grant_role: tenant-admin\n";
    let cfg = cyberos_auth::oidc::claim_mapper::parse_config(yaml).unwrap();
    let claims = IdTokenClaims { groups: Some(vec!["Engineering".into()]), /* ... */ };
    assert_eq!(cyberos_auth::oidc::claim_mapper::resolve_role(&cfg, &claims), Role::TenantAdmin);
}
```

```rust
// services/auth/tests/rbac_adr_gate_test.rs
#[tokio::test]
async fn unknown_kid_triggers_jwks_refetch(ctx: TestCtx) {
    let idp = ctx.seed_idp_config_with_mock_jwks().await;
    ctx.warmup_jwks(idp.id).await;
    let cached_kids_before = ctx.jwks_cache_kids(idp.id).await;
    // IdP rotates: emits new kid
    ctx.mock_provider().rotate_kid("new-kid-001").await;
    let token = ctx.mock_provider().sign_token_with_kid("new-kid-001").await;
    let resp = ctx.callback_with_token(idp.id, &token).await;
    assert_eq!(resp.status(), 302);
    let cached_kids_after = ctx.jwks_cache_kids(idp.id).await;
    assert!(cached_kids_after.contains(&"new-kid-001".to_string()));
    let rotation_rows = ctx.memory_audit_rows("auth.oidc_jwks_rotated").await;
    assert_eq!(rotation_rows.len(), 1);
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 6 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-004** — JWT issuance + JWKS (used after OIDC callback for our own JWT).
- **FR-AUTH-101** — RBAC role enum (claim mapper validates against).

**Downstream (2 placeholders):**
- **FR-TEN-101** — self-serve signup form (consumes OIDC for federated identity).
- **FR-PORTAL-003** — external IdP integration with JIT user provisioning.

**Cross-module:**
- **FR-AUTH-002** — subject create (JIT provisioning calls internal helper).
- **FR-AI-003** — memory audit bridge.
- **FR-MEMORY-111** — PII scrubbing for sub_claim + email.
- **FR-OBS-007** — sev-2 alarm on signature failures > 5/h.

---

## §8 — Example payloads

### 8.1 — POST /v1/auth/oidc/idp-configs

```json
{
  "name": "Acme Okta",
  "issuer_url": "https://acme.okta.com",
  "client_id": "0oa3xyz...",
  "client_secret": "REDACTED_SECRET",
  "redirect_uri": "https://auth.cyberos.world/v1/auth/oidc/callback",
  "claim_mapping_yaml": "default_role: tenant-member\nclaim_rules:\n  - claim: groups\n    contains: Engineering\n    grant_role: tenant-admin\n  - claim: groups\n    contains: Finance\n    grant_role: cfo\n"
}
```

### 8.2 — auth.oidc_login_succeeded memory row

```json
{
  "kind": "auth.oidc_login_succeeded",
  "tenant_id": "5e8f1d2a-...",
  "idp_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "sub_claim_hash16": "abc123def4567890",
  "granted_role": "tenant-admin",
  "ts_ns": 1747920731000000000
}
```

### 8.3 — auth.oidc_jit_provisioned memory row

```json
{
  "kind": "auth.oidc_jit_provisioned",
  "tenant_id": "5e8f1d2a-...",
  "idp_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "new_subject_id_hash16": "9b1deb4d3b7d4bad",
  "sub_claim_hash16": "abc123def4567890",
  "email_hash16": "def4567890123456",
  "granted_role": "tenant-member",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — auth.oidc_signature_invalid memory row (sev-2)

```json
{
  "kind": "auth.oidc_signature_invalid",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "idp_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "source_ip_hash16": "fed0987654321abc",
  "reason": "unknown_kid",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Cross-IdP subject linking** — slice 3.
- **OIDC dynamic client registration (RFC 7591)** — slice 3.
- **Refresh token rotation** — handled by FR-AUTH-004 separately.
- **SCIM provisioning** — FR-PORTAL-004.
- **SAML 2.0 SSO** — FR-AUTH-103 (sibling).

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| IdP discovery unreachable | reqwest timeout | 500 + sev-3 | IdP health investigation |
| Discovery payload lacks PKCE | validator | IdP config save fails | Use PKCE-supporting IdP |
| Implicit flow IdP config | validator | 400 | Use auth code flow |
| JWKS unreachable | reqwest fail | 401 + sev-3 | IdP key endpoint check |
| Unknown kid (rotation) | refetch + retry | Success or 401 if still unknown | Designed |
| id_token signature invalid | jsonwebtoken | 401 + sev-2 audit | Investigate |
| iss mismatch | validation.set_issuer | 401 | Designed |
| aud mismatch | validation.set_audience | 401 | Designed |
| exp expired | validation + leeway | 401 | Re-auth |
| nbf future | validation + leeway | 401 | Clock sync |
| nonce mismatch | post-decode check | 401 | CSRF defense |
| State reused | Redis lookup | 401 | Designed |
| State expired | TTL | 401 | Restart flow |
| Unknown role in claim_mapping_yaml | parse_config | 400 at config save | Fix YAML |
| Claim mapping no match | resolve_role fallback | default_role applied | Designed |
| 4th IdP config | handler check | 409 | ADR + cap raise |
| Cross-tenant sub conflict | UNIQUE (idp_id, sub_claim) | 409 | Designed |
| KMS decrypt fail (client_secret) | aws-sdk error | 500 + sev-1 | Rotate secret |
| JIT subject create fail | FR-AUTH-002 error | 500 + audit | Investigate |
| Append-only login_history UPDATE | SQL grant | permission denied | Designed |
| Append-only login_history DELETE | SQL grant | permission denied | Designed |
| RLS bypass | USING | 0 rows | Designed |
| OTel span attribute missing | otel_test | CI fails | Fix |
| Redis unreachable (state store) | redis error | 500 | Redis health |
| client_secret leaked in API response | handler omits | Never exposed | Designed |
| Concurrent state insert | Redis SETNX | One wins | Designed |
| Disabled IdP login attempt | is_active check | 401 | Re-enable or use other |
| Claim mapping YAML malformed | serde_yaml error | 400 | Fix YAML |
| > 5/h signature failures | OBS rule | sev-2 | Investigate IdP/attack |

---

## §11 — Implementation notes

- **RFC 8414 discovery + 1h cache** — fast hot path; refresh on cache miss.
- **JWKS 24h cache + per-kid lookup** — old kids accepted during overlap; new kids refetched on unknown.
- **PKCE S256 mandatory** — auth code interception defense.
- **Implicit + hybrid flow forbidden** — modern best practice.
- **State + nonce mandatory** — CSRF + replay defense.
- **10-minute state TTL** — covers slow IdP + user delay.
- **60-second clock skew** — generous + replay-safe.
- **JIT provisioning per first login** — auto-create AUTH subject.
- **Per-tenant claim mapping YAML** — closed shape; default_role fallback.
- **Max 3 IdP configs per tenant** — UX bound; ADR to raise.
- **Per-(idp_id, sub) uniqueness** — different IdPs = different subjects at slice 1.
- **Append-only login_history at SQL grant** — forensic record.
- **client_secret KMS-encrypted** — secret-at-rest hardening.
- **redirect_uri locked in config** — open-redirect defense.
- **6 memory audit kinds** — succeeded/failed/jit/jwks-rotated/config-changed/signature-invalid.
- **PII scrubbing of sub + email** — chain holds hashed forms.
- **Sev-2 alarm on > 5/h signature failures** — operator investigation prompt.
- **claim_mapping_yaml validates roles at save** — closed FR-AUTH-101 enum.
- **`auth.oidc_idp_config_changed` row excludes client_secret** — PII-grade secret never logged.
- **OIDC scopes: openid (required) + profile + email** — standard set.

---

*End of FR-AUTH-104.*
