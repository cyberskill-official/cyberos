---
id: FR-AUTH-110
title: "AUTH OIDC Provider - first-party authorization server (OIDC Core + RFC 8414 discovery, authorize with SSO-session / upstream-IdP brokering, token + id_token, userinfo, first-party RP client registry, PKCE S256, JWKS reuse, revoke-gated authorize)"
module: AUTH
priority: MUST
status: done
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-06-29
memory_chain_hash: null
related_frs: [FR-AUTH-004, FR-AUTH-005, FR-AUTH-101, FR-AUTH-104, FR-MCP-004, FR-PORTAL-003, FR-PORTAL-004]
depends_on: [FR-AUTH-004, FR-AUTH-104, FR-AUTH-005, FR-AUTH-101]
blocks: [FR-PORTAL-003]

source_pages:
  - website/docs/modules/auth.html#provider
source_decisions:
  - DEC-2480 (AUTH is the central first-party identity provider; CHAT/Mattermost, PORTAL, and other first-party apps federate to AUTH via OIDC Authorization Code + PKCE - the inverse of FR-AUTH-104, where AUTH is the client of Google)
  - DEC-2481 (reuse FR-AUTH-004 `auth_signing_keys` + `/.well-known/jwks.json` for id_token + access_token signing; the provider mints NO second key system - one JWKS for the whole platform)
  - DEC-2482 (the authorize endpoint authenticates the human via an AUTH SSO session cookie when present (silent SSO); when absent it brokers to the tenant's primary upstream IdP via FR-AUTH-104 (Google), and on return resumes the authorize and issues the downstream code - so Google stays the ultimate authenticator and AUTH is the broker)
  - DEC-2483 (first-party relying parties are admin-registered confidential clients in `auth_oidc_rp_clients` with locked redirect_uris + KMS-encrypted client_secret; open Dynamic Client Registration RFC 7591 is OUT at slice 1 - first-party RPs are few and operator-managed)
  - DEC-2484 (Authorization Code flow + PKCE S256 mandatory; implicit + hybrid flow forbidden, mirroring FR-AUTH-104 DEC-401)
  - DEC-2485 (consent screen is SKIPPED for first-party clients - they are internal trusted apps; per-login consent + third-party consent UI deferred to slice 3)
  - DEC-2486 (id_token carries: sub = CyberOS subject_id, email, name/preferred_username, tenant_id, roles per FR-AUTH-101; aud = the RP client_id; iss = the AUTH issuer; signed RS256 against auth_signing_keys)
  - DEC-2487 (userinfo endpoint returns the same identity claims for the access_token bearer; access_token is the FR-AUTH-004 RS256 JWT scoped `openid`)
  - DEC-2488 (authorize is revoke-gated: a subject revoked via FR-AUTH-005 (deny-list) CANNOT obtain a new code - the kick blocks re-authentication into every first-party app; existing downstream sessions are killed by the slice-2 back-channel-logout follow-up, NOT this slice)
  - DEC-2489 (AUTH SSO browser session: `auth_sso_sessions` table + `__Host-cyberos_sso` cookie, Secure + HttpOnly + SameSite=Lax, TTL 8h sliding max 24h absolute, revocable, bound to subject_id + tenant_id - this is the "log in once, every first-party app trusts it" linchpin)
  - DEC-2490 (auth codes are single-use, 60-second TTL, PKCE-bound, stored in `auth_oidc_auth_codes`; replay or expiry → `invalid_grant`; reuse of a consumed code also revokes any token already issued from it, per RFC 6749 §4.1.2 security note)
  - DEC-2491 (RP redirect_uri is exact-match against the registered set - no wildcard, no substring; mismatch → reject WITHOUT redirecting (open-redirect defense), render an error instead)
  - DEC-2492 (RFC 8414 + OIDC discovery: `GET /.well-known/openid-configuration` advertises issuer + authorize/token/userinfo/jwks/end_session endpoints + `code` response type + `authorization_code` grant + `S256`; pins the safe OIDC profile)
  - DEC-2493 (reuse the proven OAuth-server substrate already shipped in `services/mcp-gateway/src/oauth/` - pkce, jwt minting, code/token store shape, discovery shape, error enum - extracted to a shared crate OR re-implemented in auth; the OIDC layer (id_token, userinfo, openid-configuration, human-auth brokering, first-party RP registry, SSO cookie) is the net-new surface)
  - DEC-2494 (memory audit kinds: auth.op_authorize_issued, auth.op_authorize_denied, auth.op_token_issued, auth.op_userinfo_served, auth.op_rp_client_changed, auth.op_sso_session_started, auth.op_sso_session_revoked)
  - DEC-2495 (REVOKE UPDATE, DELETE on auth_oidc_auth_codes + auth_op_login_history from cyberos_app - codes are consumed not edited; history is append-only forensic record)
  - DEC-2496 (state parameter is the RP's CSRF token and is echoed back verbatim, never inspected; nonce from the RP is bound into the id_token per OIDC Core)
  - DEC-2497 (only tenant-admin per FR-AUTH-101 may register/modify RP clients; client_secret never returned after creation, only a one-time reveal at create)
  - DEC-2498 (the AUTH issuer URL is a single configured canonical value; id_token `iss` and discovery `issuer` MUST match it exactly so downstream RPs validate consistently)
  - DEC-2499 (first-party RPs MAY receive a refresh_token (offline_access) reusing FR-AUTH-004 rotation; for CHAT/Mattermost it is optional since Mattermost mints its own session - default OFF per RP, opt-in flag on the client record)
  - RFC 6749 (OAuth 2.0); RFC 7636 (PKCE); RFC 7515/7517/7518 (JWT/JWS/JWA + JWKS); RFC 8414 (OAuth metadata); RFC 8707 (resource indicators); OIDC Core 1.0; OIDC Discovery 1.0; OIDC RP-Initiated + Back-Channel Logout (slice 2)
  - PDPL Art. 13 (data minimisation - id_token carries only the claims the RP needs; audit chain PII-scrubbed)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0027_oidc_rp_clients.sql      # renumbered off existing 0013-0026
  - services/auth/migrations/0028_oidc_auth_codes.sql      # + auth_oidc_code_consumptions single-use guard (ADR OPEN-001 #1)
  - services/auth/migrations/0029_sso_sessions.sql
  - services/auth/migrations/0030_op_login_history.sql
  - services/auth/src/op/mod.rs                            # [slice 1a DONE] module wiring + slice/ADR notes
  - services/auth/src/op/discovery.rs                      # [slice 1a DONE] OIDC + RFC 8414 well-known document
  - services/auth/src/op/pkce.rs                           # [slice 1a DONE] PKCE S256 verify (RFC 7636)
  - services/auth/src/op/redirect.rs                       # [slice 1a DONE] exact redirect_uri match (DEC-2491)
  - services/auth/src/op/authorize.rs                      # [slice 1b] /authorize: SSO-session or upstream-IdP broker → code
  - services/auth/src/op/token.rs                          # /token: code → id_token + access_token (+ optional refresh)
  - services/auth/src/op/userinfo.rs                       # /userinfo: bearer access_token → identity claims
  - services/auth/src/op/id_token.rs                       # [slice 1a DONE] id_token builder (claims + RS256 sign via auth_signing_keys)
  - services/auth/src/op/rp_client.rs                      # first-party RP client registry CRUD + redirect_uri match
  - services/auth/src/op/code_store.rs                     # single-use, 60s TTL, PKCE-bound auth codes
  - services/auth/src/op/sso_session.rs                    # AUTH SSO browser session (cookie + table) for silent SSO
  - services/auth/src/op/audit.rs                          # 7 memory row builders
  - services/auth/src/op/errors.rs                         # [slice 1a DONE] closed RFC 6749/OIDC error enum
  - services/auth/src/handlers/op.rs                       # the provider REST surface (authorize/token/userinfo/discovery/rp-clients)
  - services/auth/tests/op_authorize_pkce_test.rs
  - services/auth/tests/op_authorize_revoke_gate_test.rs
  - services/auth/tests/op_token_exchange_test.rs
  - services/auth/tests/op_id_token_claims_test.rs
  - services/auth/tests/op_userinfo_test.rs
  - services/auth/tests/op_redirect_uri_exact_match_test.rs
  - services/auth/tests/op_code_single_use_test.rs
  - services/auth/tests/op_sso_silent_test.rs
  - services/auth/tests/op_discovery_test.rs
  - services/auth/tests/op_rp_client_admin_test.rs
  - services/auth/tests/op_audit_emission_test.rs
modified_files:
  - services/auth/src/lib.rs                               # pub mod op
  - services/auth/src/handlers.rs                          # mount the provider routes

allowed_tools:
  - file_read: services/auth/**
  - file_read: services/mcp-gateway/src/oauth/**          # reuse reference
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test op

disallowed_tools:
  - implement implicit flow or hybrid flow (per DEC-2484)
  - skip PKCE S256 (per DEC-2484)
  - allow wildcard or substring redirect_uri match (per DEC-2491)
  - issue a code or token to a revoked subject (per DEC-2488)
  - return client_secret after creation (per DEC-2497)
  - mint a second signing-key system instead of reusing auth_signing_keys (per DEC-2481)
  - open Dynamic Client Registration at slice 1 (per DEC-2483)
  - log raw id_token / access_token to file (chain holds scrubbed claims)

effort_hours: 12
sub_tasks:
  - "0.6h: 0013_oidc_rp_clients.sql - first-party RP registry + KMS-encrypted secret + locked redirect_uris"
  - "0.4h: 0014_oidc_auth_codes.sql - single-use, 60s TTL, PKCE-bound"
  - "0.5h: 0015_sso_sessions.sql - AUTH SSO browser session"
  - "0.3h: 0016_op_login_history.sql - append-only provider login history"
  - "0.5h: discovery.rs - OIDC + RFC 8414 well-known"
  - "1.6h: authorize.rs - SSO-session check + upstream-IdP broker + revoke gate + code issue"
  - "1.2h: token.rs - code exchange → id_token + access_token (+ optional refresh)"
  - "0.6h: userinfo.rs - bearer → identity claims"
  - "0.8h: id_token.rs - claim builder + RS256 sign via auth_signing_keys"
  - "0.9h: rp_client.rs - registry CRUD + exact redirect_uri match"
  - "0.6h: code_store.rs - single-use + TTL + PKCE bind"
  - "0.9h: sso_session.rs - cookie + table + silent SSO"
  - "0.4h: audit.rs - 7 row builders"
  - "0.7h: handlers/op.rs - REST surface"
  - "2.0h: tests - 11 test files"

risk_if_skipped: "Without AUTH as an OIDC provider, CHAT (Mattermost) cannot federate to one CyberOS identity - it would either run its own password store (no kick-by-revoke) or trust Google directly (chat identity not unified with the platform, no shared roles, no central revoke). The unified path Stephen chose requires AUTH to be the broker: one login, every first-party app trusts the same subject, and a single FR-AUTH-005 revoke locks the person out of all of them. Without DEC-2484's PKCE the code-interception attack is open; without DEC-2491's exact redirect_uri match the open-redirect attack is open; without DEC-2488's revoke gate a fired employee keeps re-authenticating into chat; without DEC-2481's key reuse the platform fragments into two JWKS that drift. The 12h effort lands the standards-compliant first-party IdP that the whole multi-app identity story stands on - and it directly reuses the OAuth-server substrate already proven in FR-MCP-004, so the net-new surface is the OIDC layer + human-auth brokering + the SSO cookie."
---

## §1 - Description (BCP-14 normative)

The AUTH service **MUST** ship a first-party OIDC provider (authorization server) so that CyberOS first-party apps - CHAT (Mattermost), PORTAL, and future ones - authenticate users against one CyberOS identity via OIDC Authorization Code + PKCE. This is the inverse of FR-AUTH-104: there AUTH is the OIDC client of Google; here AUTH is the OIDC provider that downstream apps consume. Each requirement:

1. **MUST** define `auth_oidc_rp_clients`: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, name TEXT NOT NULL, client_id TEXT NOT NULL UNIQUE, client_secret_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, redirect_uris TEXT[] NOT NULL, post_logout_redirect_uris TEXT[] NOT NULL DEFAULT '{}', allow_refresh BOOLEAN NOT NULL DEFAULT false, is_active BOOLEAN NOT NULL DEFAULT true, created_at TIMESTAMPTZ, created_by_subject_id UUID NOT NULL)`. First-party relying parties are admin-registered confidential clients (DEC-2483). UNIQUE `(tenant_id, name)`.

2. **MUST** define `auth_oidc_auth_codes`: `(code_hash TEXT PRIMARY KEY, tenant_id UUID, rp_client_id TEXT NOT NULL, subject_id UUID NOT NULL, redirect_uri TEXT NOT NULL, code_challenge TEXT NOT NULL, nonce TEXT, scope TEXT NOT NULL, sso_session_id UUID NOT NULL, issued_at TIMESTAMPTZ, expires_at TIMESTAMPTZ NOT NULL, consumed_at TIMESTAMPTZ)`. The stored value is a hash of the code, not the code (DB-dump safety). Single-use, 60-second TTL (DEC-2490).

3. **MUST** define `auth_sso_sessions`: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL, created_at TIMESTAMPTZ, last_seen_at TIMESTAMPTZ, absolute_expiry TIMESTAMPTZ NOT NULL, revoked_at TIMESTAMPTZ)`. The browser holds a `__Host-cyberos_sso` cookie carrying the session id; the table is the source of truth so the session is revocable (DEC-2489).

4. **MUST** define `auth_op_login_history`: `(id BIGSERIAL, tenant_id UUID, rp_client_id TEXT, subject_id UUID, outcome TEXT NOT NULL CHECK (outcome IN ('authorize_issued','token_issued','denied')), reason TEXT, source_ip_hash16 TEXT, ts TIMESTAMPTZ)`. `REVOKE UPDATE, DELETE FROM cyberos_app` (DEC-2495).

5. **MUST** enforce RLS with both `USING` and `WITH CHECK` on all four tables: `tenant_id = current_setting('app.current_tenant_id')::uuid`. (Note: AUTH sets the GUC via `SELECT set_config('app.current_tenant_id', $1, true)` inside a transaction - the bind form `SET LOCAL ... = $1` is invalid Postgres and was the class of bug fixed across the SSO path on 2026-06-29.)

6. **MUST** serve OIDC + RFC 8414 discovery at `GET /.well-known/openid-configuration` (DEC-2492) advertising: `issuer` (the configured canonical AUTH URL, DEC-2498), `authorization_endpoint` (`/v1/auth/op/authorize`), `token_endpoint` (`/v1/auth/op/token`), `userinfo_endpoint` (`/v1/auth/op/userinfo`), `jwks_uri` (the existing `/.well-known/jwks.json`), `end_session_endpoint` (slice 2, advertised as present), `response_types_supported: ["code"]`, `grant_types_supported: ["authorization_code"]` (plus `refresh_token` when any active RP has `allow_refresh`), `code_challenge_methods_supported: ["S256"]`, `scopes_supported: ["openid","profile","email","offline_access"]`, `id_token_signing_alg_values_supported: ["RS256"]`, `subject_types_supported: ["public"]`.

7. **MUST** implement `GET /v1/auth/op/authorize` (DEC-2482, DEC-2484):
   - Validate `client_id` is an active registered RP; unknown → render error, do NOT redirect.
   - Validate `redirect_uri` is an exact match of a registered `redirect_uris` entry (DEC-2491); mismatch → render error, do NOT redirect (open-redirect defense).
   - Require `response_type=code`, `scope` containing `openid`, `code_challenge` + `code_challenge_method=S256`, `state` (echoed), optional `nonce`. Missing PKCE → redirect back with `error=invalid_request`.
   - Establish the human: if a valid `__Host-cyberos_sso` cookie maps to a non-revoked `auth_sso_sessions` row, use its `subject_id` (silent SSO). Else broker to the tenant's primary upstream IdP via FR-AUTH-104 `initiate`, preserving the original authorize request; on the FR-AUTH-104 callback, create an SSO session (§1 #9) and resume this authorize.
   - Revoke gate (DEC-2488): if the resolved `subject_id` is revoked per FR-AUTH-005 (deny-list / `subjects.status='revoked'`), deny with `access_denied` + emit `auth.op_authorize_denied`; the person cannot obtain a code.
   - Issue a single-use code: 32 random bytes base64url; store `code_hash`, bind `rp_client_id`, `subject_id`, `redirect_uri`, `code_challenge`, `nonce`, `scope`, `sso_session_id`, `expires_at = now + 60s`; redirect to `redirect_uri?code=<>&state=<>`.

8. **MUST** implement `POST /v1/auth/op/token` (DEC-2490, DEC-2486, DEC-2487):
   - Authenticate the RP (`client_secret_basic` or POST body `client_id`+`client_secret`); bad secret → 401 `invalid_client`.
   - `grant_type=authorization_code`: look up `code_hash`; missing/expired/consumed → `invalid_grant`. On a consumed-code replay, ALSO revoke any token already minted from it (RFC 6749 §4.1.2). Verify `redirect_uri` equals the stored value and `code_verifier` satisfies the stored `code_challenge` (S256). Re-check the revoke gate (DEC-2488).
   - Mint: `access_token` = FR-AUTH-004 RS256 JWT, scope `openid` + granted, `aud` includes the RP; `id_token` = RS256 JWT per §1 #10 with `nonce` echoed; `token_type=Bearer`, `expires_in`. Mark the code consumed.
   - `grant_type=refresh_token`: only if the RP has `allow_refresh` (DEC-2499); rotate per FR-AUTH-004.
   - Emit `auth.op_token_issued`.

9. **MUST** manage the AUTH SSO browser session (DEC-2489, §1 #3): after any successful human authentication at authorize (silent reuse OR fresh upstream login), ensure an `auth_sso_sessions` row + set `__Host-cyberos_sso` (Secure, HttpOnly, SameSite=Lax, Path=/, no Domain - the `__Host-` prefix forbids Domain and requires Secure+Path=/). Sliding TTL 8h on `last_seen_at`, absolute 24h. This is what makes the second app a one-click login. Emit `auth.op_sso_session_started` on creation.

10. **MUST** build the id_token (DEC-2486): claims `iss` (= configured issuer, DEC-2498), `sub` (= CyberOS `subject_id`), `aud` (= RP `client_id`), `exp` (+1h), `iat`, `auth_time`, `nonce` (echoed if supplied), `email`, `email_verified`, `name`, `preferred_username`, `tenant_id`, `roles` (FR-AUTH-101 array). Signed RS256 against `auth_signing_keys` with the current `kid` (DEC-2481). NO second key system.

11. **MUST** implement `GET /v1/auth/op/userinfo` (DEC-2487): validate the bearer access_token (RS256 against the same JWKS, `aud` + `exp` + revoke gate); return `sub`, `email`, `email_verified`, `name`, `preferred_username`, `tenant_id`, `roles`. Emit `auth.op_userinfo_served`. Revoked subject → 401.

12. **MUST** enforce exact redirect_uri matching (DEC-2491) at BOTH authorize (against the registered set) and token (against the code-bound value). No wildcard, no path-prefix, no query-param laxness. The comparison is byte-exact after WHATWG URL normalization of the registered values at write time.

13. **MUST** revoke-gate every credential issuance (DEC-2488): authorize, token, and userinfo all re-check that `subject_id` is not revoked. A revoke (FR-AUTH-005) therefore blocks re-authentication into every first-party app immediately. (Killing already-live downstream sessions in real time is the slice-2 back-channel-logout follow-up, §9.)

14. **MUST** skip consent for first-party clients (DEC-2485): registered RPs are trusted internal apps; authorize issues the code without a consent prompt. A `first_party` flag is implicit (all slice-1 RPs are first-party). Third-party consent UI is slice 3.

15. **MUST** restrict RP registration to tenant-admin (DEC-2497): `POST /v1/auth/op/rp-clients` (create), `PATCH .../{id}`, `DELETE .../{id}` (soft-delete `is_active=false`) require role `tenant-admin` per FR-AUTH-101. The plaintext `client_secret` is returned exactly once at create; never again. Emit `auth.op_rp_client_changed` (excluding the secret value).

16. **MUST** store auth codes hashed + single-use + 60s TTL (DEC-2490): the table key is `sha256(code)`; lookups hash the presented code; first successful exchange sets `consumed_at`; any later use → `invalid_grant`.

17. **MUST** treat `state` as opaque RP CSRF (DEC-2496): echo it back verbatim on the redirect, never parse or store it beyond the round trip. `nonce` is bound into the id_token.

18. **MUST** PII-scrub `email`, `subject_id`, `reason` via the memory PII layer before chain commit; audit rows carry hash16 forms (mirrors FR-AUTH-104 §1 #15).

19. **MUST** complete the token exchange in ≤ 200 ms p95 (no upstream network call on the token path - JWKS + signing keys are local; the upstream-IdP latency lives only on the cold authorize path). `op_token_perf_test`.

20. **MUST** emit OTel spans `auth.op.{authorize,token,userinfo,rp_client_change,sso_session}` with `outcome` attribute (issued | denied_revoked | unknown_client | redirect_mismatch | invalid_grant | pkce_fail | silent_sso | upstream_broker).

21. **MUST** emit OTel metrics: `auth_op_authorize_total{tenant_id, rp_client_id, outcome}`, `auth_op_token_total{outcome}`, `auth_op_active_sso_sessions` (gauge), `auth_op_authorize_denied_revoked_total` (counter - a spike means many revoked users still trying, expected right after a layoff), `auth_op_token_latency_ms` (histogram, SLO p95 < 200ms).

22. **MUST** emit memory audit rows for 7 kinds (DEC-2494): `auth.op_authorize_issued`, `auth.op_authorize_denied`, `auth.op_token_issued`, `auth.op_userinfo_served`, `auth.op_rp_client_changed`, `auth.op_sso_session_started`, `auth.op_sso_session_revoked`.

23. **MUST** reject implicit + hybrid flow (DEC-2484): authorize hard-requires `response_type=code`; any other value → `unsupported_response_type`.

24. **MUST** pin the issuer (DEC-2498): a single configured canonical AUTH URL is the `iss` in every id_token AND the `issuer` in discovery; they MUST be byte-identical so RPs validate consistently. Misconfiguration (mismatch) → startup refuses to boot the provider routes.

25. **MUST** make refresh tokens opt-in per RP (DEC-2499): `allow_refresh=false` by default; CHAT/Mattermost does not need it (Mattermost mints its own session post-login). When true, `offline_access` scope grants a refresh_token rotated per FR-AUTH-004.

26. **MUST** revoke SSO sessions on subject revoke: when FR-AUTH-005 revokes a subject, all that subject's `auth_sso_sessions` rows are marked `revoked_at` (so silent SSO stops too, not only new logins). Emit `auth.op_sso_session_revoked`. (This is the slice-1 reach of the kick into the broker; the reach into already-issued downstream app sessions is slice 2.)

---

## §2 - Why this design (rationale for humans)

Why AUTH becomes the provider, not Mattermost trusting Google directly (DEC-2480). Stephen chose the unified path: one CyberOS identity behind every surface. If Mattermost trusted Google directly, chat would have its own identity island - no shared roles, no central revoke, and the AuthBridge plugin we inspected cannot bridge it (a Mattermost plugin cannot replace the login route, and the shipped one is a simulation: in-memory users, a fabricated session string, no Mattermost SDK). Making AUTH the broker means every first-party app federates to the same subject, and a single revoke locks the person out of all of them.

Why reuse the FR-AUTH-004 keys and JWKS (DEC-2481). The platform already signs RS256 and publishes one JWKS. A second key system for the provider would fragment verification and drift on rotation. One issuer, one key set, one place to rotate.

Why broker through the existing session or Google, not a new password prompt (DEC-2482). The human already proves who they are to CyberOS via Google (FR-AUTH-104). The provider should not re-ask. If the user has an AUTH SSO session, authorize is silent. If not, it sends them through the same Google flow that already works, then issues the downstream code. Google stays the root authenticator; AUTH is the broker that turns that into a CyberOS-shaped identity every app understands.

Why first-party clients are admin-registered, not self-registering (DEC-2483). The relying parties are a small, known set we operate (chat, portal). Open Dynamic Client Registration is for ecosystems of third-party apps - it is the MCP gateway's job for agents, not this. Admin registration with locked redirect_uris keeps the trust boundary tight.

Why PKCE mandatory and implicit forbidden (DEC-2484). Same reasoning as FR-AUTH-104: authorization-code interception is the classic attack, PKCE binds the code to the initiator, and implicit/hybrid leak tokens through the URL. OAuth 2.1 deprecates them. We only ship the safe profile.

Why skip consent for first-party apps (DEC-2485). A consent screen exists to protect a user from a third-party app over-asking. For our own chat and portal there is nothing to consent to - the apps are us. Forcing a consent click on every internal login is friction with no security gain. Third-party consent comes when we actually have third-party RPs (slice 3).

Why the id_token carries roles and tenant (DEC-2486). Mattermost and portal need more than "who" - they need "which tenant" and "what role" to place the user correctly. Putting roles in the id_token lets the RP map a CyberOS admin to a Mattermost admin without a second call. userinfo returns the same for RPs that prefer the call (DEC-2487).

Why the authorize endpoint is revoke-gated (DEC-2488). This is the heart of kick-by-revoke. The moment a subject is revoked, authorize refuses to mint a code, so they cannot get a fresh session in any first-party app. We are honest about the limit: an already-open Mattermost session is not killed by this gate alone, because Mattermost holds its own session after login. Killing live sessions in real time needs back-channel logout, which is the named slice-2 follow-up. Slice 1 guarantees they cannot get back in; slice 2 throws them out instantly.

Why an AUTH SSO session cookie (DEC-2489). Without it, every app would re-run the full Google round trip - the opposite of unified. The cookie is the "logged into CyberOS" fact that all first-party apps borrow. It is a server-side session (the table is truth) so it is revocable, with a sliding 8h / absolute 24h window. The `__Host-` prefix is the strict cookie form: Secure, Path=/, no Domain - the hardest to fixate or scope-leak.

Why codes are hashed, single-use, 60 seconds (DEC-2490). A code is a one-time bearer of a login. Storing only its hash means a DB dump does not leak live codes. Single-use plus the RFC 6749 rule (replay of a consumed code revokes its tokens) closes the replay window. 60 seconds is ample for an immediate redirect-and-exchange and short enough to bound theft.

Why exact redirect_uri matching, reject-without-redirect (DEC-2491). Loose redirect matching is how open-redirect and token-leak attacks happen. Exact match against the registered set, and when it fails we render an error rather than redirecting anywhere - because redirecting to an unverified URI is itself the vulnerability.

Why pin the issuer (DEC-2498). Downstream validators compare the id_token `iss` to the discovery `issuer`. If those differ by even a trailing slash, every RP rejects every token. One configured canonical value, checked at boot, removes a whole class of "it works on my machine" failures.

Why refresh tokens are opt-in per RP (DEC-2499). Mattermost mints its own session after the OIDC handshake, so it never needs our refresh token; handing one out anyway is needless standing credential. Portal or a SPA might want offline_access. Default off, opt-in per client.

Why reuse the MCP OAuth substrate (DEC-2493). FR-MCP-004 already built and proved authorize, token, PKCE, JWT-minting against `auth_signing_keys`, the code/token store, and the RFC 8414 discovery shape. The honest move is to lift that substrate (extract a shared crate or re-implement the thin parts in auth) and spend the budget on what is genuinely new here: the OIDC layer (id_token, userinfo, openid-configuration), the human-auth brokering, and the SSO cookie. The audit should decide extract-vs-reimplement.

---

## §3 - API contract

### 3.1 - Migration 0013 - rp_clients

```sql
-- services/auth/migrations/0013_oidc_rp_clients.sql
BEGIN;

CREATE TABLE auth_oidc_rp_clients (
    id                         UUID        PRIMARY KEY,
    tenant_id                  UUID        NOT NULL,
    name                       TEXT        NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    client_id                  TEXT        NOT NULL UNIQUE,
    client_secret_kms_blob     BYTEA       NOT NULL,
    kms_key_id                 TEXT        NOT NULL,
    redirect_uris              TEXT[]      NOT NULL CHECK (cardinality(redirect_uris) BETWEEN 1 AND 10),
    post_logout_redirect_uris  TEXT[]      NOT NULL DEFAULT '{}',
    allow_refresh              BOOLEAN     NOT NULL DEFAULT false,
    is_active                  BOOLEAN     NOT NULL DEFAULT true,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by_subject_id      UUID        NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX uniq_rp_client_name ON auth_oidc_rp_clients (tenant_id, name);

ALTER TABLE auth_oidc_rp_clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_oidc_rp_clients FORCE ROW LEVEL SECURITY;
CREATE POLICY rp_clients_tenant_iso ON auth_oidc_rp_clients
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::uuid);

COMMIT;
```

### 3.2 - Migration 0014 - auth_codes

```sql
-- services/auth/migrations/0014_oidc_auth_codes.sql
BEGIN;

CREATE TABLE auth_oidc_auth_codes (
    code_hash       TEXT         PRIMARY KEY,            -- sha256(code), never the code
    tenant_id       UUID         NOT NULL,
    rp_client_id    TEXT         NOT NULL,
    subject_id      UUID         NOT NULL REFERENCES auth.subjects(id),
    redirect_uri    TEXT         NOT NULL,
    code_challenge  TEXT         NOT NULL,               -- S256
    nonce           TEXT,
    scope           TEXT         NOT NULL,
    sso_session_id  UUID         NOT NULL,
    issued_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ  NOT NULL,               -- issued_at + 60s
    consumed_at     TIMESTAMPTZ
);

CREATE INDEX auth_codes_expiry_idx ON auth_oidc_auth_codes (expires_at);

ALTER TABLE auth_oidc_auth_codes ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_oidc_auth_codes FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_codes_tenant_iso ON auth_oidc_auth_codes
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::uuid);

REVOKE UPDATE, DELETE ON auth_oidc_auth_codes FROM cyberos_app;  -- consume via consumed_at set in one UPDATE-as-INSERT-guard; see note

COMMIT;
```

Note on consume: because UPDATE is revoked, "consume" is a `... WHERE consumed_at IS NULL` guarded update is not available; instead the consume is modeled as an insert into a sibling `auth_oidc_code_consumptions(code_hash PK)` - the first insert wins, a second raises a unique violation = replay. The audit should confirm this pattern vs granting a narrow UPDATE.

### 3.3 - Migration 0015 - sso_sessions

```sql
-- services/auth/migrations/0015_sso_sessions.sql
BEGIN;

CREATE TABLE auth_sso_sessions (
    id                UUID         PRIMARY KEY,
    tenant_id         UUID         NOT NULL,
    subject_id        UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE CASCADE,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    last_seen_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    absolute_expiry   TIMESTAMPTZ  NOT NULL,              -- created_at + 24h
    revoked_at        TIMESTAMPTZ
);

CREATE INDEX sso_sessions_subject_idx ON auth_sso_sessions (subject_id) WHERE revoked_at IS NULL;

ALTER TABLE auth_sso_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_sso_sessions FORCE ROW LEVEL SECURITY;
CREATE POLICY sso_sessions_tenant_iso ON auth_sso_sessions
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::uuid);

COMMIT;
```

### 3.4 - Discovery document

```rust
// services/auth/src/op/discovery.rs
use serde_json::{json, Value};

/// OIDC + RFC 8414 provider metadata. `issuer` is the single configured canonical AUTH URL
/// (DEC-2498); `jwks_uri` is the existing FR-AUTH-004 /.well-known/jwks.json (DEC-2481).
pub fn openid_configuration(issuer: &str, jwks_uri: &str, grant_types: &[&str]) -> Value {
    let base = issuer.trim_end_matches('/');
    json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/v1/auth/op/authorize"),
        "token_endpoint":         format!("{base}/v1/auth/op/token"),
        "userinfo_endpoint":      format!("{base}/v1/auth/op/userinfo"),
        "end_session_endpoint":   format!("{base}/v1/auth/op/logout"),   // slice 2
        "jwks_uri": jwks_uri,
        "response_types_supported": ["code"],
        "grant_types_supported": grant_types,                            // ["authorization_code"] (+ "refresh_token")
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "offline_access"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
        "claims_supported": ["sub","email","email_verified","name","preferred_username","tenant_id","roles"],
    })
}
```

### 3.5 - id_token builder

```rust
// services/auth/src/op/id_token.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct IdTokenClaims {
    pub iss: String,                 // configured issuer (DEC-2498)
    pub sub: String,                 // CyberOS subject_id
    pub aud: String,                 // RP client_id
    pub exp: i64,
    pub iat: i64,
    pub auth_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
}

/// Signs with the current FR-AUTH-004 signing key (RS256). The `kid` header MUST match a key
/// published in /.well-known/jwks.json so the RP can verify. No second key system (DEC-2481).
pub fn sign_id_token(claims: &IdTokenClaims, signer: &cyberos_auth::signing::ActiveKey)
    -> Result<String, super::errors::OpError> { /* jsonwebtoken encode with header.kid = signer.kid */ unimplemented!() }
```

### 3.6 - authorize (control flow, normative skeleton)

```rust
// services/auth/src/op/authorize.rs  (shape, not full body)
// 1. parse + validate client_id (active RP)            -> else render_error("unknown_client")  [no redirect]
// 2. exact-match redirect_uri against rp.redirect_uris  -> else render_error("redirect_mismatch") [no redirect]  (DEC-2491)
// 3. require response_type=code, scope∋openid, S256 code_challenge, state; nonce optional
//      -> else redirect(redirect_uri, error=invalid_request|unsupported_response_type, state)
// 4. resolve subject:
//      if valid __Host-cyberos_sso cookie -> sso row (not revoked) -> subject_id        (silent_sso)
//      else -> stash this authorize request, 302 into FR-AUTH-104 initiate (upstream broker)
//              on FR-AUTH-104 callback: create sso session (DEC-2489), resume here        (upstream_broker)
// 5. REVOKE GATE (DEC-2488): subject revoked? -> redirect(error=access_denied) + audit op_authorize_denied
// 6. mint single-use code (60s, PKCE-bound, sso_session_id) -> redirect(redirect_uri, code, state)
//    + audit op_authorize_issued
```

---

## §4 - Acceptance criteria

1. **Discovery served** - `/.well-known/openid-configuration` returns the pinned OIDC profile; `issuer` equals the configured value; `jwks_uri` is the existing FR-AUTH-004 JWKS.
2. **Unknown client rejected without redirect** - bad `client_id` → error page, no Location header.
3. **redirect_uri exact match** - registered `https://chat.cyberos.world/signin/oidc/complete` accepted; any variant (extra slash, query, subpath) → error page, no redirect.
4. **PKCE required** - authorize without `code_challenge` → redirect `error=invalid_request`.
5. **Implicit/hybrid rejected** - `response_type=token` or `code id_token` → `unsupported_response_type`.
6. **Silent SSO** - with a valid `__Host-cyberos_sso` cookie, authorize issues a code with no upstream round trip (`outcome=silent_sso`).
7. **Upstream broker** - with no cookie, authorize drives FR-AUTH-104 Google login, creates an SSO session, then issues the code.
8. **Revoke gate at authorize** - a revoked subject → `access_denied`, no code, `auth.op_authorize_denied` row.
9. **Code single-use** - first `/token` exchange succeeds; replay of the same code → `invalid_grant` AND any token from it is revoked.
10. **Code TTL** - exchange after 60s → `invalid_grant`.
11. **PKCE verifier checked** - wrong `code_verifier` → `invalid_grant`.
12. **id_token claims** - `iss`/`sub`/`aud`/`exp`/`nonce`/`email`/`name`/`tenant_id`/`roles` present; signed RS256; `kid` resolves in JWKS; nonce echoed.
13. **access_token + userinfo** - `/userinfo` with the access_token returns the same identity; revoked subject → 401.
14. **RP auth** - `/token` with wrong `client_secret` → 401 `invalid_client`.
15. **RP registry admin-only** - non-admin create → 403; `client_secret` returned once at create, never on read/update.
16. **SSO revoke cascade** - revoking the subject marks `auth_sso_sessions.revoked_at`; subsequent silent SSO fails → re-auth required.
17. **Issuer pinned** - booting with discovery `issuer` != id_token `iss` config → provider routes refuse to start.
18. **Append-only login history** - UPDATE/DELETE on `auth_op_login_history` from `cyberos_app` → permission denied.
19. **Token exchange p95 < 200 ms** - `op_token_perf_test`.
20. **OTel span `auth.op.token`** - `outcome` attribute present.
21. **7 audit kinds emitted** - one test asserts each row shape.

---

## §5 - Verification

```rust
// services/auth/tests/op_authorize_revoke_gate_test.rs
#[tokio::test]
async fn revoked_subject_cannot_get_code(ctx: TestCtx) {
    let rp  = ctx.register_rp("chat", &["https://chat.cyberos.world/signin/oidc/complete"]).await;
    let sub = ctx.seed_subject_with_sso_session("[email protected]").await;
    ctx.revoke_subject(sub.id).await;                       // FR-AUTH-005
    let resp = ctx.authorize(&rp, &sub.sso_cookie, "S256-challenge", "state123").await;
    assert_eq!(resp.status(), 302);
    let loc = resp.headers()["Location"].to_str().unwrap();
    assert!(loc.contains("error=access_denied"));
    assert!(!loc.contains("code="));
    assert_eq!(ctx.memory_rows("auth.op_authorize_denied").await.len(), 1);
}
```

```rust
// services/auth/tests/op_code_single_use_test.rs
#[tokio::test]
async fn code_replay_revokes_issued_tokens(ctx: TestCtx) {
    let (rp, code, verifier) = ctx.happy_authorize().await;
    let first = ctx.token_exchange(&rp, &code, &verifier).await.unwrap();      // 200
    assert!(first.id_token.split('.').count() == 3);
    let replay = ctx.token_exchange(&rp, &code, &verifier).await.unwrap_err();
    assert!(format!("{replay:?}").contains("invalid_grant"));
    assert!(ctx.access_token_is_revoked(&first.access_token).await);          // RFC 6749 §4.1.2
}
```

```rust
// services/auth/tests/op_redirect_uri_exact_match_test.rs
#[tokio::test]
async fn redirect_uri_must_match_exactly(ctx: TestCtx) {
    let rp = ctx.register_rp("chat", &["https://chat.cyberos.world/signin/oidc/complete"]).await;
    for bad in ["https://chat.cyberos.world/signin/oidc/complete/",
                "https://chat.cyberos.world/signin/oidc/complete?x=1",
                "https://evil.example/signin/oidc/complete"] {
        let resp = ctx.authorize_with_redirect(&rp, bad).await;
        assert!(resp.is_error_page());        // rendered, NOT redirected
        assert!(resp.headers().get("Location").is_none());
    }
}
```

```rust
// services/auth/tests/op_id_token_claims_test.rs
#[tokio::test]
async fn id_token_carries_roles_and_tenant_signed_by_jwks(ctx: TestCtx) {
    let (rp, code, verifier) = ctx.happy_authorize_with_roles(&["tenant-admin"]).await;
    let tok = ctx.token_exchange(&rp, &code, &verifier).await.unwrap();
    let claims = ctx.verify_against_jwks(&tok.id_token).await.unwrap();   // verifies kid ∈ JWKS
    assert_eq!(claims["iss"], ctx.configured_issuer());
    assert_eq!(claims["aud"], rp.client_id);
    assert_eq!(claims["roles"], serde_json::json!(["tenant-admin"]));
    assert!(claims["tenant_id"].is_string());
}
```

---

## §6 - Implementation skeleton

§3 is the skeleton. The provider reuses the FR-MCP-004 `oauth` substrate (pkce, jwt-minting against `auth_signing_keys`, code/token store shape, discovery shape, error enum) per DEC-2493; the net-new is the OIDC layer (id_token, userinfo, openid-configuration), the authorize human-auth brokering into FR-AUTH-104, and the SSO cookie/session. The 7 memory row builders follow the canonical pattern.

---

## §7 - Dependencies

Upstream:
- FR-AUTH-004 - signing keys + JWKS (id_token + access_token signed against these; one key system).
- FR-AUTH-104 - OIDC client (the upstream Google login that authorize brokers to when there is no SSO session).
- FR-AUTH-005 - revoke / deny-list (the kick the authorize/token/userinfo gates enforce).
- FR-AUTH-101 - role enum (roles claim in the id_token).
- FR-MCP-004 - the OAuth-server substrate reused (DEC-2493).

Downstream:
- CHAT SSO (the Mattermost OIDC connector) consumes this provider - the reachable unified-path login (the AuthBridge plugin is a non-working simulation and is dropped; Mattermost's native OIDC connector points at this provider's discovery URL). The exact CHAT FR id is reconciled in the backlog.
- FR-PORTAL-003 - external IdP / portal login federates here too.
- FR-PORTAL-004 - SCIM auto-deprovision complements the revoke gate for instant cross-app deprovisioning.

Cross-module:
- FR-AUTH-002 - subject lookup for claims.
- FR-AI-003 / memory - audit bridge.
- FR-OBS-007 - alarm on a denied-revoked spike (expected post-layoff; sustained from one IP may be probing).

---

## §8 - Example payloads

### 8.1 - POST /v1/auth/op/rp-clients (create, admin-only)

```json
{
  "name": "CyberOS Chat",
  "redirect_uris": ["https://chat.cyberos.world/signin/oidc/complete"],
  "post_logout_redirect_uris": ["https://chat.cyberos.world/login"],
  "allow_refresh": false
}
```

Response (the only time the secret is shown):

```json
{
  "client_id": "cyberos-chat",
  "client_secret": "ONE_TIME_REVEAL_VALUE",
  "redirect_uris": ["https://chat.cyberos.world/signin/oidc/complete"]
}
```

### 8.2 - auth.op_token_issued memory row

```json
{
  "kind": "auth.op_token_issued",
  "tenant_id": "5e8f1d2a-...",
  "rp_client_id": "cyberos-chat",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "scope": "openid profile email",
  "ts_ns": 1751155200000000000
}
```

### 8.3 - auth.op_authorize_denied memory row (revoke gate)

```json
{
  "kind": "auth.op_authorize_denied",
  "tenant_id": "5e8f1d2a-...",
  "rp_client_id": "cyberos-chat",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "reason": "subject_revoked",
  "ts_ns": 1751155200000000000
}
```

---

## §9 - Open questions

Deferred (named slices, not gaps):
- **Back-channel + RP-initiated logout** (OIDC logout specs) - slice 2. This is what kills a live Mattermost session the instant a subject is revoked; slice 1 already blocks re-authentication and revokes the SSO session.
- **Refresh-token rotation for RPs** - slice 2 (the FR-AUTH-004 rotation already exists; this wires it per RP).
- **Third-party consent UI + RFC 7591 Dynamic Client Registration** - slice 3 (first-party RPs are admin-registered).
- **SCIM auto-deprovision** - FR-PORTAL-004 (instant cross-app removal beyond the revoke gate).
- **Extract-vs-reimplement the MCP oauth substrate** - the audit decides whether to lift `services/mcp-gateway/src/oauth/` into a shared crate or re-implement the thin parts in auth (DEC-2493).

---

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown client_id | registry lookup | error page, no redirect | Register RP |
| redirect_uri mismatch | exact match | error page, no redirect | Fix RP config |
| Missing PKCE | param check | redirect error=invalid_request | RP sends S256 |
| Implicit/hybrid response_type | param check | unsupported_response_type | Use code flow |
| No SSO session, no cookie | cookie absent | broker to FR-AUTH-104 | Designed |
| Revoked subject at authorize | revoke gate | access_denied + audit | Designed (the kick) |
| Revoked subject at token | revoke gate | invalid_grant | Designed |
| Revoked subject at userinfo | revoke gate | 401 | Designed |
| Code expired (>60s) | expires_at | invalid_grant | Re-initiate |
| Code replay | consume guard | invalid_grant + token revoke | Designed |
| Wrong code_verifier | S256 check | invalid_grant | Designed |
| Wrong client_secret | client auth | 401 invalid_client | Fix secret |
| issuer mismatch (config) | boot check | provider refuses to start | Fix config |
| JWKS/signing key missing | FR-AUTH-004 | 500 + sev-1 | Key health |
| SSO cookie fixation | __Host- prefix + server table | rejected | Designed |
| SSO session past absolute expiry | absolute_expiry | re-auth | Designed |
| Append-only history UPDATE/DELETE | SQL grant | permission denied | Designed |
| RLS bypass | USING + FORCE | 0 rows | Designed |
| client_secret leaked in response | one-time reveal only | never re-exposed | Designed |
| Live downstream session after revoke | (slice 2) | not killed in slice 1 | Back-channel logout (slice 2) |

---

## §11 - Implementation notes

- One issuer, one JWKS - reuse FR-AUTH-004; never a second key system.
- authorize brokers: silent via the SSO cookie, else through FR-AUTH-104 Google, then resume.
- PKCE S256 mandatory; implicit/hybrid forbidden.
- redirect_uri exact match; on mismatch render an error, never redirect.
- Codes hashed + single-use + 60s; replay revokes issued tokens.
- Revoke gate on authorize + token + userinfo = kick-by-revoke into every first-party app.
- SSO session is server-side truth (revocable); `__Host-` cookie is the strict form.
- id_token carries sub + email + name + tenant_id + roles; signed RS256 with a JWKS kid.
- First-party RPs admin-registered, confidential, locked redirect_uris, secret one-time reveal.
- Consent skipped for first-party; third-party consent is slice 3.
- Refresh tokens opt-in per RP (off for Mattermost).
- Reuse the FR-MCP-004 oauth substrate; spend the budget on the OIDC layer + brokering + SSO cookie.
- Honest kick boundary: slice 1 blocks re-auth + revokes SSO; slice 2 back-channel logout kills live app sessions.
- 7 memory audit kinds; PII-scrubbed; append-only history.

---

*End of FR-AUTH-110.*
