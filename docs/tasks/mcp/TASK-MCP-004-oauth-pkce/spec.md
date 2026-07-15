---
id: TASK-MCP-004
title: "OAuth 2.1 PKCE authorization-code flow with audience-bound tokens for MCP servers"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: MCP
priority: p0
status: done
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P1
milestone: P1 · mcp-substrate
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-004, TASK-MCP-001, TASK-MCP-005, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008]
depends_on: [TASK-AUTH-004, TASK-MCP-001]
blocks: [TASK-MCP-005, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008]

source_pages:
  - website/docs/modules/mcp.html#oauth
source_decisions:
  - DEC-800 2026-05-16 — Follow OAuth 2.1 draft + MCP 2025-11-25 auth profile — authorization code + PKCE only, no implicit, no resource-owner-password
  - DEC-801 2026-05-16 — PKCE code_challenge_method = S256 only — plain rejected per OAuth 2.1 §7.6
  - DEC-802 2026-05-16 — Audience binding: every issued token carries `aud` = MCP resource server's canonical URL; resource server rejects mismatched aud (RFC 8707)
  - DEC-803 2026-05-16 — Public clients (CLIs, desktop apps) MUST use PKCE; confidential clients (server-side) MAY use PKCE + MUST authenticate
  - DEC-804 2026-05-16 — Client registration via RFC 7591 Dynamic Client Registration; tenant-admin role required for confidential client; public clients self-registered
  - DEC-805 2026-05-16 — Token format: signed JWT (RS256 or ES256) per TASK-AUTH-004 JWKS; access_token TTL = 1 hour, refresh_token TTL = 30 days
  - DEC-806 2026-05-16 — Refresh-token rotation MUST replace the prior refresh_token on every refresh (sender-constrained); reused old token marks the family as compromised + invalidates all descendants (OAuth 2.1 §6.1)
  - DEC-807 2026-05-16 — Closed `oauth_grant_type` enum (authorization_code, refresh_token) — no client_credentials at this slice (deferred to TASK-MCP-007), no implicit, no password
  - DEC-808 2026-05-16 — Closed `client_type` enum (public, confidential)
  - DEC-809 2026-05-16 — Authorization request `redirect_uri` MUST be pre-registered + exact-match comparison (no substring/regex) — per OAuth 2.1 §3.1.2.4
  - DEC-810 2026-05-16 — `state` parameter REQUIRED on every authorization request; CSRF defense
  - DEC-811 2026-05-16 — Authorization code TTL = 30 seconds (OAuth 2.1 recommends ≤10 min; tighter is safer)
  - DEC-812 2026-05-16 — Authorization code one-time-use; replayed code marks the client + binding token family as compromised + emits sev-2 audit
  - DEC-813 2026-05-16 — Scopes defined via TASK-MCP-001 `tools/list` registry; closed set per MCP server
  - DEC-814 2026-05-16 — JWT signed with TASK-AUTH-004 JWKS rotating keys; aud + iss + sub + scope + nonce + iat + exp + jti claims
  - DEC-815 2026-05-16 — All token issuance and refusal emit memory audit rows with PII scrubbing via TASK-MEMORY-111
  - DEC-816 2026-05-16 — Per-tenant `mcp_oauth_allowlist_redirect_hosts` cap on which redirect_uri hosts may be registered (defense-in-depth against open-redirect attack)
  - DEC-817 2026-05-16 — Token revocation endpoint RFC 7009; revocations propagate via TASK-AUTH-004 JWKS short-lived signing keys
  - DEC-818 2026-05-16 — Introspection endpoint RFC 7662 for resource servers (closed-network only; not exposed to public clients)
  - DEC-819 2026-05-16 — Discovery via RFC 8414 `/.well-known/oauth-authorization-server` — separate from TASK-MCP-005 Protected Resource Metadata (PRM)
  - DEC-820 2026-05-16 — Closed 6-value error enum (invalid_request, invalid_client, invalid_grant, unauthorized_client, unsupported_grant_type, invalid_scope) — per RFC 6749 §5.2 + MCP profile

build_envelope:
  language: rust 1.81
  service: cyberos/services/mcp-gateway/
  new_files:
    - services/mcp-gateway/src/oauth/mod.rs
    - services/mcp-gateway/src/oauth/authorize.rs
    - services/mcp-gateway/src/oauth/token.rs
    - services/mcp-gateway/src/oauth/pkce.rs
    - services/mcp-gateway/src/oauth/refresh.rs
    - services/mcp-gateway/src/oauth/dcr.rs
    - services/mcp-gateway/src/oauth/revoke.rs
    - services/mcp-gateway/src/oauth/introspect.rs
    - services/mcp-gateway/src/oauth/discovery.rs
    - services/mcp-gateway/src/oauth/audience.rs
    - services/mcp-gateway/src/oauth/scope.rs
    - services/mcp-gateway/src/oauth/enums.rs
    - services/mcp-gateway/src/oauth/error.rs
    # migrations renumbered 0010-0012 -> 0013-0015 (0010-0012 reserved for TASK-MCP-007/008 per BACKLOG)
    - services/mcp-gateway/migrations/0013_oauth_clients.sql
    - services/mcp-gateway/migrations/0014_oauth_codes.sql
    - services/mcp-gateway/migrations/0015_oauth_refresh_families.sql
    - services/mcp-gateway/tests/oauth_authorize_test.rs
    - services/mcp-gateway/tests/oauth_token_test.rs
    - services/mcp-gateway/tests/oauth_refresh_rotation_test.rs
    - services/mcp-gateway/tests/oauth_pkce_test.rs
    - services/mcp-gateway/tests/oauth_audience_test.rs
    - services/mcp-gateway/tests/oauth_dcr_test.rs
    - services/mcp-gateway/tests/oauth_revoke_test.rs
    - services/mcp-gateway/tests/oauth_discovery_test.rs
  modified_files:
    - services/mcp-gateway/src/handlers/tools_call.rs (assert audience-bound token before tool dispatch)
  allowed_tools:
    - file_read: services/mcp-gateway/**
    - file_write: services/mcp-gateway/{src,tests,migrations}/**
    - bash: cargo test -p cyberos-mcp-gateway
  disallowed_tools:
    - implement implicit flow (OAuth 2.1 §10.5 prohibits)
    - implement resource-owner-password grant
    - accept code_challenge_method=plain
    - issue tokens without aud claim
    - bypass PKCE on public clients

effort_hours: 10
subtasks:
  - "1.0h: oauth_clients table + DCR endpoint (POST /register)"
  - "1.0h: oauth_codes table + authorize endpoint (GET /authorize → 302 redirect)"
  - "1.5h: PKCE S256 verifier (code_challenge + code_verifier hashing)"
  - "1.5h: token endpoint (POST /token) — authorization_code + refresh_token grants"
  - "1.0h: refresh-token rotation + family compromise detection"
  - "0.5h: revocation endpoint (POST /revoke)"
  - "0.5h: introspection endpoint (POST /introspect)"
  - "0.5h: discovery endpoint (GET /.well-known/oauth-authorization-server)"
  - "1.0h: audience-bound token verification at MCP resource server hot path"
  - "0.5h: per-tenant redirect_uri allowlist + DCR validation"
  - "0.5h: memory audit emission for 8 audit kinds + TASK-MEMORY-111 scrubbing"
  - "0.5h: closed enums + CI cardinality tests"
risk_if_skipped: "Without OAuth 2.1 + PKCE, MCP clients can't authenticate to MCP servers per the 2025-11-25 protocol. Every MCP-005/006/007/008 implementation would have to re-invent auth. Without audience binding, an access token issued for `server-A` would be valid at `server-B` — full cross-server token replay. Both compliance and security are at stake."
---

## §1 — Description (BCP-14 normative)

The MCP Gateway service **MUST** implement OAuth 2.1 + PKCE per the MCP 2025-11-25 auth profile, issuing audience-bound JWT access tokens that MCP resource servers verify before dispatching tool calls.

1. **MUST** implement exactly two grant types: `authorization_code` and `refresh_token` (DEC-800 + DEC-807). Implicit, resource-owner-password, and client_credentials are NOT in scope for this slice (client_credentials is deferred to TASK-MCP-007). Unsupported grant_type returns `400 unsupported_grant_type` per RFC 6749 §5.2.

2. **MUST** require PKCE on every authorization request from a `public` client (DEC-803). The authorization request MUST include `code_challenge` (43-128 char base64url) and `code_challenge_method` (S256 only per DEC-801). Missing PKCE on a public client returns `400 invalid_request` with `error_description: "pkce_required_for_public_client"`. Plain code_challenge_method returns `400 invalid_request` with `error_description: "pkce_method_must_be_s256"`.

3. **MUST** verify PKCE at the token endpoint: SHA-256(code_verifier) base64url-encoded MUST equal the stored code_challenge. Mismatch returns `400 invalid_grant`. The code_verifier MUST be 43-128 chars per RFC 7636.

4. **MUST** define a closed 2-value `client_type` Postgres enum (`public`, `confidential`) per DEC-808. Public clients have no client_secret and rely on PKCE for proof. Confidential clients MUST authenticate at the token endpoint via Basic Auth (client_id:client_secret) per RFC 6749 §2.3.1 OR via private_key_jwt (RFC 7523). Both MUST also use PKCE if exposed to a browser context.

5. **MUST** define a closed 2-value `oauth_grant_type` Postgres enum (`authorization_code`, `refresh_token`). CI cardinality test asserts exactly 2 (DEC-807).

6. **MUST** define a closed 6-value `oauth_error_code` Postgres enum (`invalid_request`, `invalid_client`, `invalid_grant`, `unauthorized_client`, `unsupported_grant_type`, `invalid_scope`) per DEC-820 + RFC 6749 §5.2. CI cardinality test asserts exactly 6.

7. **MUST** issue access tokens as signed JWTs using the TASK-AUTH-004 JWKS keys (RS256 or ES256). The token claims MUST include:
   - `iss`: the MCP-gateway URL.
   - `aud`: the canonical URL of the MCP resource server (DEC-802 + RFC 8707).
   - `sub`: the authorizing subject's UUID (TASK-AUTH-002).
   - `scope`: space-separated scope strings from the client's authorization grant (DEC-813).
   - `nonce`: 16-byte random hex.
   - `iat`, `exp`: standard timestamps; exp = iat + 3600 (DEC-805).
   - `jti`: UUIDv4 — used for revocation list lookup (DEC-817).
   - `client_id`: the requesting client's UUID.
   - `tenant_id`: the tenant binding (per TASK-AUTH-108 Lumi pattern).

8. **MUST** issue refresh tokens as opaque 256-bit random base64url strings (NOT JWTs). The refresh token is stored hashed (Argon2 or SHA-256) in `oauth_refresh_families` table with `(family_id, token_hash, issued_at, expires_at, parent_token_hash)`. TTL = 30 days (DEC-805).

9. **MUST** enforce refresh-token rotation (DEC-806). On `grant_type=refresh_token`, the server:
   - Verifies the presented refresh_token matches an active row in `oauth_refresh_families`.
   - Marks the presented row as `state = used`.
   - Issues a new refresh_token in the same family with `parent_token_hash` pointing to the just-used row.
   - Returns the new access + refresh pair.
   If the presented refresh_token is found but `state = used` already, this is a REUSE — mark the entire family as `state = compromised`, invalidate all descendants, refuse issuance, emit a sev-1 memory audit row `mcp.oauth_refresh_reuse_detected`.

10. **MUST** verify `redirect_uri` via exact-match comparison against the pre-registered URI (DEC-809). The authorization request's `redirect_uri` MUST match one of `oauth_clients.redirect_uris` entries character-for-character. No substring, no regex, no trailing-slash normalization. Mismatch returns `400 invalid_request` with `error_description: "redirect_uri_mismatch"`.

11. **MUST** validate `redirect_uri` host on registration (DEC-816). The per-tenant policy `mcp_oauth_allowlist_redirect_hosts` (JSON array of hostnames or wildcards like `*.example.com`) restricts which hosts may appear in registered redirect URIs. Default policy: allow any HTTPS host (no host restriction). Tenants opt into stricter policies; tenant_admin role required for policy mutation + sev-2 memory audit.

12. **MUST** require `https` scheme on all redirect URIs in production. The only exception is `http://localhost[:port]/...` and `http://127.0.0.1[:port]/...` for native CLI clients (OAuth 2.1 §10.3.3). HTTP redirect to a non-loopback host returns `400 invalid_request` at registration.

13. **MUST** require the `state` parameter on every authorization request (DEC-810). Missing `state` returns `400 invalid_request` with `error_description: "state_required"`. The state is opaque to the gateway and returned unchanged on the redirect (CSRF defense).

14. **MUST** enforce authorization code TTL = 30 seconds (DEC-811). The `oauth_codes` row carries `expires_at = issued_at + INTERVAL '30 seconds'`. An expired code at token exchange returns `400 invalid_grant` with `error_description: "code_expired"`.

15. **MUST** enforce authorization code one-time-use (DEC-812). On token exchange, the handler:
    - SELECTs the row FOR UPDATE.
    - Verifies `consumed_at IS NULL`.
    - UPDATEs `consumed_at = now()`.
    - If `consumed_at IS NOT NULL` (replay), marks the client's most recent refresh family as compromised + emits sev-2 memory audit `mcp.oauth_code_reuse_detected`.

16. **MUST** define a closed 3-value `oauth_code_state` enum (`active`, `consumed`, `expired`). CI cardinality test asserts exactly 3.

17. **MUST** define a closed 3-value `oauth_refresh_state` enum (`active`, `used`, `compromised`). CI cardinality test asserts exactly 3.

18. **MUST** implement RFC 7591 Dynamic Client Registration at `POST /register`. Request body:
    - `client_type` (required, public | confidential)
    - `redirect_uris` (required, array of URIs; max 5 per client)
    - `client_name` (optional, ≤ 64 chars)
    - `scope` (required, space-separated list of MCP scopes from the registry)
    Response body:
    - `client_id`
    - `client_secret` (only for confidential; cryptographically random 256-bit base64url)
    - `client_id_issued_at` (unix ts)
    Public client registration is open (no caller auth); confidential client registration requires `tenant_admin` role per TASK-AUTH-101.

19. **MUST** scope confidential client registration to the requesting subject's tenant. The client row carries `tenant_id` and the issued access tokens carry the same tenant_id. Cross-tenant client lookup returns 404.

20. **MUST** implement RFC 8414 Discovery at `GET /.well-known/oauth-authorization-server`. Response includes:
    ```json
    {
      "issuer": "https://mcp.cyberos.world",
      "authorization_endpoint": "https://mcp.cyberos.world/authorize",
      "token_endpoint": "https://mcp.cyberos.world/token",
      "registration_endpoint": "https://mcp.cyberos.world/register",
      "revocation_endpoint": "https://mcp.cyberos.world/revoke",
      "introspection_endpoint": "https://mcp.cyberos.world/introspect",
      "jwks_uri": "https://mcp.cyberos.world/.well-known/jwks.json",
      "response_types_supported": ["code"],
      "grant_types_supported": ["authorization_code", "refresh_token"],
      "code_challenge_methods_supported": ["S256"],
      "token_endpoint_auth_methods_supported": ["none", "client_secret_basic", "private_key_jwt"],
      "scopes_supported": [<from TASK-MCP-001 registry>]
    }
    ```

21. **MUST** implement RFC 7009 token revocation at `POST /revoke`. Body: `token=<access_or_refresh>&token_type_hint=...`. The handler:
    - For access_token: adds the JWT `jti` to a revocation list cached for 1 hour (matches access_token TTL).
    - For refresh_token: marks the row + family `state = compromised`.
    - Always returns 200 OK regardless of whether token was active (RFC 7009 §2.2 — prevents probing).

22. **MUST** implement RFC 7662 introspection at `POST /introspect` for resource servers only (DEC-818). The endpoint requires the caller to authenticate as a registered confidential client with the `mcp_introspect` scope. Public clients receive 401. Response shape per RFC 7662 §2.2. Introspection enables resource servers to verify the token's audience, scope, subject, tenant — without re-implementing JWT validation.

23. **MUST** verify audience binding at every MCP resource server hot path (DEC-802). The MCP gateway's `tools/call` handler MUST:
    - Decode the bearer JWT via TASK-AUTH-004 JWKS.
    - Assert `aud` exactly equals this resource server's canonical URL.
    - Mismatched aud returns `401 invalid_token` with `WWW-Authenticate: Bearer error="invalid_token", error_description="audience_mismatch"`.

24. **MUST** verify the JWT signature, exp, and revocation list on every request. Cache the revocation list in-process with 60s TTL; misses fetch from Postgres. Defense-in-depth: signature + exp + jti revocation check + audience check.

25. **MUST** emit 8 closed memory audit kinds:
    - `mcp.oauth_authorize_started` (sev-3, per /authorize redirect issued)
    - `mcp.oauth_token_issued` (sev-3, per access_token issued)
    - `mcp.oauth_token_refreshed` (sev-3, per refresh)
    - `mcp.oauth_token_revoked` (sev-2, per revocation)
    - `mcp.oauth_refresh_reuse_detected` (sev-1, family compromise)
    - `mcp.oauth_code_reuse_detected` (sev-2, code replay)
    - `mcp.oauth_audience_mismatch` (sev-2, per /tools/call rejection)
    - `mcp.oauth_client_registered` (sev-2, per DCR call)

26. **MUST** route all reason-bearing audit text (error_description, client_name in registration audit) through TASK-MEMORY-111 PII scrubbing.

27. **MUST** persist authorization codes in the `oauth_codes` table with `(code, client_id, subject_id, redirect_uri, code_challenge, scope, nonce, state, issued_at, expires_at, consumed_at)`. The table is append-only via SQL grant + UPDATE limited to `consumed_at = now()` only via a privileged role `oauth_code_consumer`.

28. **MUST** support `prompt=none` parameter at the authorize endpoint for silent re-authorization when the subject is already logged in (existing AUTH session). On success, the gateway redirects to redirect_uri with a fresh code immediately. On failure (subject not logged in OR consent not previously granted), redirect to redirect_uri with `error=login_required` per OIDC §3.1.2.6.

29. **MUST** support a `consent_screen` flow for the first time a subject grants scopes to a client. The consent record is persisted to `oauth_consents (subject_id, client_id, scopes, granted_at)`. Subsequent authorizations with the same or subset scopes skip the consent screen. New scope set requires re-consent.

30. **MUST** validate `scope` against the MCP-server's registered scope set (TASK-MCP-001 `tools/list` registry — DEC-813). Unknown scopes return `400 invalid_scope`. Scope strings are case-sensitive and conform to RFC 6749 §3.3 syntax (visible printable, no whitespace beyond the space separator).

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why OAuth 2.1 and not OAuth 2.0.** DEC-800. OAuth 2.1 is the consolidation IETF draft that removes the unsafe parts of 2.0 (implicit grant, password grant) and codifies the safe practices (PKCE everywhere, exact-match redirect_uri, refresh rotation). The MCP 2025-11-25 auth profile explicitly cites 2.1. Implementing 2.0 with all its options would multiply attack surface; 2.1 is the right baseline.

**§2.2  Why PKCE on confidential clients too.** Clause #4. OAuth 2.1 §4.1.3.2: "MUST use PKCE if the authorization code flow is initiated in a browser context." A confidential client running a browser-based authorize step is functionally a public client during that step; PKCE protects against the authorization code interception attack regardless of how the client authenticates later.

**§2.3  Why S256 only.** DEC-801. The S256 code_challenge_method uses SHA-256 to bind code_verifier to the request; the plain method just sends the verifier in clear. Plain offers no protection against an attacker who intercepts the authorization request. OAuth 2.1 §4.1.4: "the plain method MUST NOT be used".

**§2.4  Why audience-bound tokens.** DEC-802 + RFC 8707. Without audience binding, an access token issued for `mcp-a.cyberos.world` could be presented at `mcp-b.cyberos.world` — full cross-server replay. The `aud` claim + the resource server's exact-match check eliminates this; an attacker who steals a token for one MCP server cannot use it elsewhere.

**§2.5  Why refresh-token rotation.** DEC-806. A long-lived refresh token is the highest-value credential in OAuth — stolen, it gives the attacker indefinite access. Rotation means every refresh produces a new token and invalidates the old one; an attacker who steals an old refresh token is detected the moment the legitimate client refreshes (the old token's REUSE triggers family compromise). This is the standard "reuse detection" pattern from OAuth 2.1 §6.1.

**§2.6  Why we mark the entire family on reuse, not just one token.** Clause #9 (refresh) + clause #15 (code). If an attacker has stolen one refresh token, they may have a chain of subsequent refresh tokens too. Invalidating just the reused one leaves the rest of the chain valid. Family compromise revokes the entire descendant chain in one operation.

**§2.7  Why 30-second authorization code TTL.** DEC-811. The code is meant to be exchanged immediately at the token endpoint (typically ≤ 1 second in practice). OAuth 2.1 recommends ≤10 minutes; we tighten to 30 seconds. This narrows the window where a stolen authorization code could be exchanged. Legitimate clients exchange within sub-second; the 30-second buffer accommodates network jitter.

**§2.8  Why one-time-use authorization codes.** DEC-812 + clause #15. A code that can be exchanged twice would let an attacker who intercepts the redirect (e.g., via URL-history snooping) silently issue themselves a token while the legitimate user's session also succeeds. One-time-use + reuse-detection makes intercepts visible immediately.

**§2.9  Why exact-match redirect_uri.** DEC-809 + clause #10. Substring matching enables open-redirect attacks (e.g., registered `https://app.example.com` matches attacker-controlled `https://app.example.com.attacker.com`). Regex matching has similar pitfalls. Exact match is the only safe comparison — the cost is operator discipline (register every variant explicitly), the benefit is no class of redirect-attack works.

**§2.10  Why per-tenant redirect_uri host allowlist.** DEC-816 + clause #11. Even with exact-match, a misconfigured DCR endpoint could allow registration of arbitrary HTTPS URLs. Per-tenant allowlists let tenant_admins lock down what hosts are registrable. The default (no restriction) keeps the gateway open for development; tenants opt in to strictness for production.

**§2.11  Why JWT access tokens and opaque refresh tokens.** Clauses #7 + #8. JWTs let resource servers verify access tokens without an introspection round-trip (fast hot path). Refresh tokens are presented only at the token endpoint; opaqueness lets us rotate and revoke without leaking structure to clients. The hybrid is standard practice.

**§2.12  Why TTL = 1h for access, 30d for refresh.** DEC-805. Access TTL of 1h means a stolen token's blast radius is bounded to 1h max (or until JWKS rotation, whichever first). Refresh TTL of 30d covers the common "I open my IDE every couple weeks" pattern without forcing weekly re-auth. Tighter access TTLs increase refresh traffic; looser refresh TTLs increase stolen-credential risk. 1h + 30d is the standard balance.

**§2.13  Why DCR for public clients with no caller auth.** DEC-804 + clause #18. Public clients (CLIs, desktops) have no pre-existing tenant binding when they first appear; requiring caller auth would create a chicken-and-egg. Open DCR for public clients is safe because (a) public clients can only use the authorization_code flow which requires user interaction, (b) they hold no client_secret, (c) PKCE binds the registered client to the specific authorization request.

**§2.14  Why confidential clients require tenant_admin.** DEC-804 + clause #18. A confidential client gets a client_secret that authenticates to the token endpoint without user interaction (in client_credentials, deferred to TASK-MCP-007). Issuing such credentials needs tenant accountability; tenant_admin is the right role.

**§2.15  Why scopes from the MCP server registry.** DEC-813 + clause #30. Scopes are not free-form; they map to specific tool capabilities the MCP server exposes. The TASK-MCP-001 `tools/list` registry IS the scope authority; the OAuth server validates against it. This keeps the scope vocabulary closed and machine-checkable.

**§2.16  Why discovery via RFC 8414.** DEC-819 + clause #20. Discovery lets clients learn endpoints + capabilities without hardcoding. RFC 8414 is the standard; MCP profile cites it. The endpoint is at `/.well-known/oauth-authorization-server` (separate from TASK-MCP-005's Protected Resource Metadata at `/.well-known/oauth-protected-resource`).

**§2.17  Why introspection is closed-network only.** DEC-818 + clause #22. Introspection lets a caller learn token validity + claims. Exposing it publicly lets attackers probe valid token shapes. Restricting to authenticated resource servers (with `mcp_introspect` scope) eliminates the probing surface.

**§2.18  Why we revoke via short-lived JWKS keys + revocation list.** Clauses #21 + #24 + DEC-817. JWTs are inherently bearer credentials — once issued, they're valid until exp. To revoke before exp, we need either (a) check a revocation list on every verify (small cost; gives true revocation), or (b) rotate JWKS keys so all issued JWTs invalidate together (cheap; coarse). We do both: per-token revocation via jti list + key rotation as a periodic refresh. Revocation list is cached 60s for hot-path performance.

**§2.19  Why prompt=none for silent re-auth.** Clause #28. CLIs and IDEs that re-authorize on token expiry shouldn't bounce the user through a browser prompt every hour. If the subject has a valid AUTH session and previous consent for this client, prompt=none redirects with a fresh code immediately. This matches OIDC §3.1.2.6 semantics.

**§2.20  Why consent screen on first authorization.** Clause #29. The subject should knowingly grant scopes to a client. The first authorization shows the scope list + the client name; subsequent authorizations within the same scope set skip. This is the GDPR-aligned consent model adapted to OAuth.

**§2.21  Why 8 closed audit kinds.** Clause #25. Each kind is a distinct operational signal: authorize-started (intent), token-issued (success), refreshed (renewal), revoked (deliberate end), refresh-reuse (compromise detection), code-reuse (compromise detection), audience-mismatch (cross-server probe), client-registered (DCR). Operators query on kind; collapsing would force free-text parsing.

**§2.22  Why we don't support DPoP at this slice.** DPoP (RFC 9449) sender-constrains access tokens via per-request signatures. It's the next step after audience binding. We defer it because (a) MCP 2025-11-25 doesn't mandate DPoP, (b) audience + JWKS rotation + revocation list cover the bulk of the threat model, (c) DPoP requires every client to manage signing keys. Revisit in P3 if audit findings demand it.

---

## §3 — API & schema

### §3.1 — Migration 0010: oauth_clients + closed enums

```sql
-- services/mcp-gateway/migrations/0010_oauth_clients.sql

CREATE TYPE client_type AS ENUM ('public', 'confidential');
CREATE TYPE oauth_grant_type AS ENUM ('authorization_code', 'refresh_token');
CREATE TYPE oauth_error_code AS ENUM (
    'invalid_request',
    'invalid_client',
    'invalid_grant',
    'unauthorized_client',
    'unsupported_grant_type',
    'invalid_scope'
);

CREATE TABLE oauth_clients (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            UUID REFERENCES tenants(id),   -- NULL for public CLI clients
    client_type          client_type NOT NULL,
    client_secret_hash   TEXT,                          -- NULL for public; Argon2 for confidential
    redirect_uris        JSONB NOT NULL,
    client_name          TEXT CHECK (client_name IS NULL OR length(client_name) <= 64),
    scope                TEXT NOT NULL CHECK (length(scope) BETWEEN 1 AND 1024),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at           TIMESTAMPTZ,
    CONSTRAINT confidential_has_secret CHECK (
        (client_type = 'confidential' AND client_secret_hash IS NOT NULL)
     OR (client_type = 'public' AND client_secret_hash IS NULL)
    ),
    CONSTRAINT redirect_uris_max_5 CHECK (jsonb_array_length(redirect_uris) BETWEEN 1 AND 5),
    CONSTRAINT confidential_has_tenant CHECK (
        client_type = 'public' OR tenant_id IS NOT NULL
    )
);

CREATE INDEX oauth_clients_tenant ON oauth_clients (tenant_id) WHERE revoked_at IS NULL;

REVOKE UPDATE, DELETE ON oauth_clients FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(revoked_at) ON oauth_clients TO oauth_writer;
GRANT SELECT ON oauth_clients TO oauth_reader;
```

### §3.2 — Migration 0011: oauth_codes

```sql
-- services/mcp-gateway/migrations/0011_oauth_codes.sql

CREATE TYPE oauth_code_state AS ENUM ('active', 'consumed', 'expired');

CREATE TABLE oauth_codes (
    code                TEXT PRIMARY KEY CHECK (length(code) = 43),  -- 256-bit base64url
    client_id           UUID NOT NULL REFERENCES oauth_clients(id),
    subject_id          UUID NOT NULL REFERENCES subjects(id),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    redirect_uri        TEXT NOT NULL,
    code_challenge      TEXT NOT NULL CHECK (length(code_challenge) BETWEEN 43 AND 128),
    code_challenge_method TEXT NOT NULL CHECK (code_challenge_method = 'S256'),
    scope               TEXT NOT NULL,
    audience            TEXT NOT NULL,  -- target resource server URL
    nonce               TEXT NOT NULL,
    state               TEXT NOT NULL,  -- client-supplied CSRF state
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL,
    consumed_at         TIMESTAMPTZ,
    memory_chain_hash    CHAR(64) NOT NULL
);

CREATE INDEX oauth_codes_expires ON oauth_codes (expires_at);

REVOKE UPDATE, DELETE ON oauth_codes FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(consumed_at) ON oauth_codes TO oauth_code_consumer;
```

### §3.3 — Migration 0012: oauth_refresh_families + revocation_list

```sql
-- services/mcp-gateway/migrations/0012_oauth_refresh_families.sql

CREATE TYPE oauth_refresh_state AS ENUM ('active', 'used', 'compromised');

CREATE TABLE oauth_refresh_families (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    family_id              UUID NOT NULL,
    client_id              UUID NOT NULL REFERENCES oauth_clients(id),
    subject_id             UUID NOT NULL REFERENCES subjects(id),
    tenant_id              UUID NOT NULL REFERENCES tenants(id),
    audience               TEXT NOT NULL,
    scope                  TEXT NOT NULL,
    token_hash             CHAR(64) NOT NULL,  -- SHA-256 hex
    parent_token_hash      CHAR(64),
    issued_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at             TIMESTAMPTZ NOT NULL,
    state                  oauth_refresh_state NOT NULL DEFAULT 'active',
    state_changed_at       TIMESTAMPTZ,
    memory_chain_hash       CHAR(64) NOT NULL
);

CREATE UNIQUE INDEX oauth_refresh_token_hash ON oauth_refresh_families (token_hash);
CREATE INDEX oauth_refresh_family_active ON oauth_refresh_families (family_id) WHERE state = 'active';

REVOKE UPDATE, DELETE ON oauth_refresh_families FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(state, state_changed_at) ON oauth_refresh_families TO oauth_refresh_writer;

CREATE TABLE oauth_revocation_list (
    jti           UUID PRIMARY KEY,
    revoked_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at    TIMESTAMPTZ NOT NULL,  -- matches the JWT exp; row TTL-eviction
    reason        TEXT
);

CREATE INDEX oauth_revocation_expires ON oauth_revocation_list (expires_at);

REVOKE UPDATE, DELETE ON oauth_revocation_list FROM cyberos_app;
GRANT INSERT, SELECT ON oauth_revocation_list TO oauth_writer;

CREATE TABLE oauth_consents (
    subject_id   UUID NOT NULL REFERENCES subjects(id),
    client_id    UUID NOT NULL REFERENCES oauth_clients(id),
    scopes       TEXT NOT NULL,
    granted_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_id, client_id)
);
```

### §3.4 — PKCE verification

```rust
// services/mcp-gateway/src/oauth/pkce.rs

use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

pub fn verify_pkce(code_verifier: &str, stored_code_challenge: &str) -> bool {
    if code_verifier.len() < 43 || code_verifier.len() > 128 {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    constant_time_eq::constant_time_eq(computed.as_bytes(), stored_code_challenge.as_bytes())
}
```

### §3.5 — Token endpoint (handle authorization_code grant)

```rust
// services/mcp-gateway/src/oauth/token.rs

pub async fn token_handler(
    pool: &PgPool,
    body: TokenRequest,
    client_authn: Option<ClientAuth>,
) -> Result<TokenResponse, OAuthError> {
    let client: OAuthClient = load_client(pool, &body.client_id).await?;
    if client.is_confidential() {
        verify_client_authn(&client, &client_authn)?;
    }
    match body.grant_type {
        OAuthGrantType::AuthorizationCode => handle_authz_code(pool, &client, body).await,
        OAuthGrantType::RefreshToken => handle_refresh(pool, &client, body).await,
    }
}

async fn handle_authz_code(pool: &PgPool, client: &OAuthClient, body: TokenRequest)
    -> Result<TokenResponse, OAuthError>
{
    let mut tx = pool.begin().await?;

    // §1 #15 SELECT FOR UPDATE + one-time-use
    let code_row: OAuthCodeRow = sqlx::query_as!(
        OAuthCodeRow,
        r#"SELECT code, client_id, subject_id, tenant_id, redirect_uri,
                  code_challenge, scope, audience, nonce, state,
                  issued_at, expires_at, consumed_at
           FROM oauth_codes WHERE code = $1 FOR UPDATE"#,
        body.code
    ).fetch_optional(&mut *tx).await?.ok_or(OAuthError::InvalidGrant("code_not_found"))?;

    if code_row.client_id != client.id {
        return Err(OAuthError::InvalidGrant("code_client_mismatch"));
    }
    if code_row.expires_at < Utc::now() {
        return Err(OAuthError::InvalidGrant("code_expired"));
    }
    if code_row.consumed_at.is_some() {
        // §1 #15 REUSE detected — mark refresh family compromised
        mark_family_compromised(pool, &client.id, &code_row.subject_id, "code_reuse").await?;
        emit_memory_audit(MemoryKind::OauthCodeReuseDetected, ...).await?;
        return Err(OAuthError::InvalidGrant("code_already_used"));
    }
    if code_row.redirect_uri != body.redirect_uri {
        return Err(OAuthError::InvalidGrant("redirect_uri_mismatch"));
    }

    // PKCE
    if !pkce::verify_pkce(&body.code_verifier, &code_row.code_challenge) {
        return Err(OAuthError::InvalidGrant("pkce_verification_failed"));
    }

    sqlx::query!(
        "UPDATE oauth_codes SET consumed_at = now() WHERE code = $1",
        body.code
    ).execute(&mut *tx).await?;

    // Issue access + refresh
    let access = issue_access_jwt(
        &code_row.subject_id, &code_row.tenant_id, &client.id,
        &code_row.audience, &code_row.scope
    ).await?;
    let refresh = issue_refresh(pool, &client.id, &code_row.subject_id, &code_row.tenant_id,
                                &code_row.audience, &code_row.scope, None).await?;

    emit_memory_audit(MemoryKind::OauthTokenIssued, ...).await?;
    tx.commit().await?;

    Ok(TokenResponse {
        access_token: access.token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: Some(refresh.token),
        scope: code_row.scope,
    })
}
```

### §3.6 — Refresh rotation with reuse-detection

```rust
async fn handle_refresh(pool: &PgPool, client: &OAuthClient, body: TokenRequest)
    -> Result<TokenResponse, OAuthError>
{
    let mut tx = pool.begin().await?;
    let token_hash = sha256_hex(&body.refresh_token);

    let row: OAuthRefreshRow = sqlx::query_as!(
        OAuthRefreshRow,
        r#"SELECT id, family_id, client_id, subject_id, tenant_id,
                  audience, scope, token_hash, state AS "state: OauthRefreshState", expires_at
           FROM oauth_refresh_families WHERE token_hash = $1 FOR UPDATE"#,
        &token_hash
    ).fetch_optional(&mut *tx).await?.ok_or(OAuthError::InvalidGrant("refresh_not_found"))?;

    if row.client_id != client.id {
        return Err(OAuthError::InvalidGrant("refresh_client_mismatch"));
    }
    if row.expires_at < Utc::now() {
        return Err(OAuthError::InvalidGrant("refresh_expired"));
    }

    match row.state {
        OauthRefreshState::Active => {
            // §1 #9 mark this row as used; issue child
            sqlx::query!(
                "UPDATE oauth_refresh_families SET state = 'used', state_changed_at = now() WHERE id = $1",
                row.id
            ).execute(&mut *tx).await?;

            let access = issue_access_jwt(&row.subject_id, &row.tenant_id, &client.id,
                                          &row.audience, &row.scope).await?;
            let new_refresh = issue_refresh(pool, &client.id, &row.subject_id, &row.tenant_id,
                                           &row.audience, &row.scope, Some(&token_hash)).await?;
            emit_memory_audit(MemoryKind::OauthTokenRefreshed, ...).await?;
            tx.commit().await?;

            Ok(TokenResponse {
                access_token: access.token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                refresh_token: Some(new_refresh.token),
                scope: row.scope,
            })
        }
        OauthRefreshState::Used | OauthRefreshState::Compromised => {
            // §1 #9 REUSE — mark entire family compromised
            sqlx::query!(
                r#"UPDATE oauth_refresh_families
                      SET state = 'compromised', state_changed_at = now()
                      WHERE family_id = $1 AND state != 'compromised'"#,
                row.family_id
            ).execute(&mut *tx).await?;
            emit_memory_audit(MemoryKind::OauthRefreshReuseDetected, ...).await?;
            tx.commit().await?;
            Err(OAuthError::InvalidGrant("refresh_reuse_detected"))
        }
    }
}
```

### §3.7 — Audience verification at MCP resource server

```rust
// services/mcp-gateway/src/handlers/tools_call.rs (modified)

pub async fn tools_call(
    auth_header: HeaderValue,
    body: ToolsCallRequest,
) -> Result<ToolsCallResponse, McpError> {
    let bearer = parse_bearer(auth_header)?;
    let claims = jwt::verify(&bearer, &JWKS).await?;

    let expected_aud = std::env::var("MCP_RESOURCE_SERVER_URL")?;
    if claims.aud != expected_aud {
        emit_memory_audit(MemoryKind::OauthAudienceMismatch, &claims, &expected_aud).await?;
        return Err(McpError::InvalidToken("audience_mismatch"));
    }

    if is_revoked(&claims.jti).await? {
        return Err(McpError::InvalidToken("token_revoked"));
    }

    // Tenant + scope authorized — dispatch
    dispatch_tool(&body, &claims).await
}
```

---

## §4 — Acceptance criteria

1. CI cardinality tests assert `client_type` (2), `oauth_grant_type` (2), `oauth_error_code` (6), `oauth_code_state` (3), `oauth_refresh_state` (3).
2. Public client DCR: POST /register with `client_type=public` succeeds without caller auth.
3. Confidential client DCR: requires `tenant_admin` role.
4. DCR registers redirect_uris (1-5 URIs); 6th rejected with 400.
5. Authorization request without `code_challenge` from public client → 400 `pkce_required_for_public_client`.
6. Authorization request with `code_challenge_method=plain` → 400 `pkce_method_must_be_s256`.
7. Authorization request without `state` → 400 `state_required`.
8. Authorization request with substring-matching `redirect_uri` (not exact) → 400 `redirect_uri_mismatch`.
9. Authorization request with HTTP non-loopback redirect → 400 at registration (not at authorize — caught earlier).
10. Authorization code TTL = 30s; exchange at +31s returns `invalid_grant code_expired`.
11. Authorization code one-time-use: second exchange returns `invalid_grant code_already_used` + marks refresh family compromised + sev-2 audit.
12. PKCE verification: code_verifier mismatch returns `invalid_grant pkce_verification_failed`.
13. Token endpoint issues JWT with claims iss + aud + sub + scope + nonce + iat + exp + jti + client_id + tenant_id.
14. Access token TTL = 3600s (1h); refresh TTL = 30 days.
15. Refresh rotation: each refresh returns new refresh_token; old marked `used`.
16. Refresh reuse: presenting `used` refresh_token marks family compromised + invalidates all descendants + sev-1 audit `mcp.oauth_refresh_reuse_detected`.
17. Audience mismatch at MCP resource server: returns 401 `invalid_token audience_mismatch` + sev-2 audit.
18. Revoked JWT (jti in revocation_list) at /tools/call returns 401 `token_revoked`.
19. Revocation list cached in-process with 60s TTL.
20. Revocation endpoint: POST /revoke with valid token returns 200; same for invalid token (no probing surface).
21. Introspection endpoint requires `mcp_introspect` scope; public client request returns 401.
22. Discovery endpoint (GET /.well-known/oauth-authorization-server) returns RFC 8414 JSON with all required fields.
23. prompt=none with logged-in subject + previous consent redirects with fresh code immediately.
24. prompt=none without logged-in subject redirects with `error=login_required`.
25. First-time scope grant shows consent screen; subsequent same-scope grants skip.
26. Scope outside TASK-MCP-001 registry returns `400 invalid_scope`.
27. JWT signed with TASK-AUTH-004 JWKS (RS256 or ES256); signature verified on every request.
28. oauth_codes / oauth_refresh_families REVOKE UPDATE/DELETE from cyberos_app confirmed; only privileged roles can mutate specific columns.
29. Per-tenant redirect_uri host allowlist enforced at registration; mutation requires tenant_admin + sev-2 audit.
30. memory audit row emitted per 8 closed kinds; all reason text scrubbed via TASK-MEMORY-111.
31. Confidential client authentication: client_secret_basic AND private_key_jwt both supported.
32. Cross-tenant client lookup returns 404 (tenant isolation at confidential client level).

---

## §5 — Verification (CI tests)

- `cardinality_test_all` — 5 enums.
- `dcr_public_test` — POST /register without auth → success.
- `dcr_confidential_acl_test` — non-tenant_admin → 403.
- `dcr_redirect_uri_max_test` — 6 URIs → 400.
- `authorize_pkce_required_test` — public client w/o PKCE → 400.
- `authorize_pkce_plain_test` — plain method → 400.
- `authorize_state_required_test` — missing state → 400.
- `authorize_redirect_substring_test` — substring not match → 400.
- `register_http_non_loopback_test` — `http://example.com/cb` → 400.
- `register_http_loopback_test` — `http://localhost:8080/cb` → succeed.
- `code_ttl_test` — exchange at +31s → invalid_grant.
- `code_one_time_test` — second exchange → family compromised + sev-2.
- `pkce_verification_test` — wrong verifier → 400.
- `jwt_claims_test` — assert iss + aud + sub + scope + nonce + iat + exp + jti + client_id + tenant_id present.
- `access_ttl_test` — exp - iat = 3600.
- `refresh_ttl_test` — expires_at - issued_at = 30 days.
- `refresh_rotation_test` — old marked used; new active.
- `refresh_reuse_test` — present used refresh → compromised + sev-1.
- `audience_mismatch_test` — JWT aud != server aud → 401.
- `revoked_jti_test` — revocation_list hit → 401.
- `revocation_cache_test` — 60s TTL.
- `revoke_endpoint_test` — POST /revoke valid + invalid both return 200.
- `introspect_acl_test` — public client → 401.
- `discovery_test` — JSON matches RFC 8414 fields.
- `prompt_none_logged_in_test` — fresh code immediately.
- `prompt_none_logged_out_test` — error=login_required.
- `consent_first_test` — first auth shows consent.
- `consent_skip_test` — subsequent same scope skips.
- `scope_unknown_test` — not in registry → 400.
- `jwt_signature_test` — JWKS verify.
- `append_only_test` — REVOKE inspection.
- `redirect_host_allowlist_test` — host not in tenant policy → 400 at registration.
- `confidential_basic_test` — Basic auth → success.
- `confidential_private_key_jwt_test` — private_key_jwt → success.
- `cross_tenant_client_404_test` — wrong tenant returns 404.

---

## §6 — File skeleton

```
services/mcp-gateway/
├── src/
│   ├── oauth/
│   │   ├── mod.rs          # pub re-exports
│   │   ├── authorize.rs    # GET /authorize handler
│   │   ├── token.rs        # POST /token (§3.5 + §3.6)
│   │   ├── pkce.rs         # §3.4 verify
│   │   ├── refresh.rs      # rotation + family-compromise
│   │   ├── dcr.rs          # POST /register
│   │   ├── revoke.rs       # POST /revoke
│   │   ├── introspect.rs   # POST /introspect
│   │   ├── discovery.rs    # GET /.well-known/oauth-authorization-server
│   │   ├── audience.rs     # audience verification helper
│   │   ├── scope.rs        # scope validation against TASK-MCP-001 registry
│   │   ├── jwt.rs          # access JWT mint/verify (uses TASK-AUTH-004 JWKS)
│   │   ├── consent.rs      # consent screen handler + persisted record
│   │   ├── audit.rs        # 8 memory audit kinds
│   │   └── error.rs        # OAuthError + RFC 6749 §5.2 mapping
│   └── handlers/
│       └── tools_call.rs   # MODIFIED: verify audience + revocation list
├── migrations/
│   ├── 0010_oauth_clients.sql
│   ├── 0011_oauth_codes.sql
│   └── 0012_oauth_refresh_families.sql
└── tests/
    ├── oauth_authorize_test.rs
    ├── oauth_token_test.rs
    ├── oauth_refresh_rotation_test.rs
    ├── oauth_pkce_test.rs
    ├── oauth_audience_test.rs
    ├── oauth_dcr_test.rs
    ├── oauth_revoke_test.rs
    └── oauth_discovery_test.rs
```

---

## §7 — Dependencies & blast-radius

**Depends on**: TASK-AUTH-004 (JWKS for JWT signing), TASK-MCP-001 (scope registry from tools/list).

**Blocks**: TASK-MCP-005 (Protected Resource Metadata), TASK-MCP-006 (resource indicators), TASK-MCP-007 (client_credentials grant), TASK-MCP-008 (tenant-admin token revocation UI).

**Blast radius if broken**:
- **Audience-binding bypass**: cross-server token replay — any token works at any MCP server.
- **Refresh-reuse not detected**: stolen refresh = indefinite access.
- **PKCE bypass**: authorization code interception attacks viable on public clients.
- **Redirect_uri substring matching**: open-redirect → token leakage.
- **Code TTL too loose**: stolen code window widens.

---

## §8 — Payload examples

### §8.1 — Authorization request

```
GET /authorize?
  response_type=code&
  client_id=cli_abc&
  redirect_uri=http://localhost:8080/cb&
  scope=mcp.tools.list%20mcp.tools.call&
  state=xyz&
  code_challenge=E9Melh...&
  code_challenge_method=S256&
  audience=https://mcp-a.cyberos.world

302 → http://localhost:8080/cb?code=AcDe...&state=xyz
```

### §8.2 — Token request

```
POST /token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=AcDe...&
redirect_uri=http://localhost:8080/cb&
client_id=cli_abc&
code_verifier=dBjftJeZ...

200 OK
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "vF7-...",
  "scope": "mcp.tools.list mcp.tools.call"
}
```

### §8.3 — Refresh request

```
POST /token
grant_type=refresh_token&refresh_token=vF7-...&client_id=cli_abc

200 OK
{
  "access_token": "eyJhbGc... (new)",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "Ka9Lm... (new)",
  "scope": "mcp.tools.list mcp.tools.call"
}
```

### §8.4 — Refresh reuse rejection

```
POST /token
grant_type=refresh_token&refresh_token=vF7-... (already used)

400 Bad Request
{ "error": "invalid_grant", "error_description": "refresh_reuse_detected" }
```

### §8.5 — Audience mismatch at /tools/call

```
POST /tools/call (at mcp-b.cyberos.world)
Authorization: Bearer eyJ... (aud=mcp-a.cyberos.world)

401 Unauthorized
WWW-Authenticate: Bearer error="invalid_token", error_description="audience_mismatch"
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-800): OAuth 2.1 baseline.
- **OQ-2** (closed by DEC-806): refresh rotation mandatory.
- **OQ-3** (open): DPoP support — defer to P3 per §2.22.
- **OQ-4** (open): mTLS client authentication (RFC 8705) as alternative to client_secret_basic — out of scope for this slice.
- **OQ-5** (open): JAR (RFC 9101) signed request objects — defer to P3.

---

## §10 — Failure modes (33 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | PKCE missing on public client | request validation | 3 | 400 + sev-3 |
| 2 | PKCE method = plain | request validation | 3 | 400 + sev-3 |
| 3 | code_challenge length out of [43, 128] | CHECK | 3 | 400 |
| 4 | code_verifier mismatch | PKCE verify | 3 | invalid_grant + sev-3 |
| 5 | Authorization code TTL expired | timestamp | 3 | invalid_grant code_expired |
| 6 | Authorization code reuse | consumed_at != NULL | 2 | invalid_grant + family compromised + sev-2 |
| 7 | Refresh-token reuse | state = used | 1 | family compromised + sev-1 audit |
| 8 | Refresh-token compromised family | state = compromised | 1 | invalid_grant + sev-1 |
| 9 | Redirect_uri substring (non-exact) | string compare | 3 | 400 redirect_uri_mismatch |
| 10 | HTTP non-loopback redirect at registration | URL parser | 3 | 400 invalid_redirect |
| 11 | State parameter missing | request validation | 3 | 400 state_required |
| 12 | Client_id unknown | DB lookup | 3 | 400 invalid_client |
| 13 | Confidential client missing client_secret | authn check | 2 | 401 invalid_client |
| 14 | Audience mismatch at resource server | aud claim compare | 2 | 401 + sev-2 |
| 15 | JWT signature invalid | JWKS verify | 1 | 401 + sev-1 |
| 16 | JWT exp passed | exp claim | 3 | 401 |
| 17 | JWT jti in revocation list | revocation list check | 3 | 401 token_revoked |
| 18 | Revocation list cache stale > 60s | TTL refresh | 3 | Refresh from DB |
| 19 | Scope outside MCP-001 registry | scope validation | 3 | 400 invalid_scope |
| 20 | DCR public with confidential body | constraint violation | 3 | 400 |
| 21 | DCR > 5 redirect_uris | CHECK constraint | 3 | 400 |
| 22 | DCR client_name > 64 chars | CHECK | 3 | 400 |
| 23 | DCR confidential without tenant_admin role | RBAC | 2 | 403 + sev-2 |
| 24 | Redirect host not in tenant allowlist | policy check | 3 | 400 |
| 25 | Tenant allowlist mutation by non-admin | RBAC | 2 | 403 + sev-2 |
| 26 | prompt=none without active session | session check | 3 | redirect with error=login_required |
| 27 | Consent scope wider than previously granted | scope diff | 3 | Show consent screen |
| 28 | Discovery endpoint omits required field | response shape test | 1 | CI blocks |
| 29 | Introspection by public client | RBAC | 2 | 401 + sev-2 |
| 30 | Cross-tenant client lookup | RLS / tenant context | 2 | 404 + sev-2 |
| 31 | JWT issued without aud claim | code review + test | 1 | CI blocks |
| 32 | Audit emission fails | subprocess error | 1 | Retry via WAL; sev-1 if exhausted |
| 33 | Code expired but cleanup job fails | TTL cron | 3 | Periodic cleanup; sev-3 |

---

## §11 — Implementation notes

**§11.1** All OAuth endpoints serve under a single base path; the Discovery doc lists the absolute URLs.

**§11.2** Refresh tokens are stored as SHA-256 hashes (32 bytes), not Argon2 (we need fast lookup; collision-resistance of SHA-256 is sufficient since the token is 256-bit random).

**§11.3** Access JWT signing uses the TASK-AUTH-004 active signing key from JWKS. Key rotation invalidates all in-flight access tokens (acceptable — they re-issue via refresh).

**§11.4** The revocation list is held in-process per gateway as `lru::LruCache<Uuid, Instant>` with 60s TTL. On cache miss, the gateway queries `oauth_revocation_list` and inserts.

**§11.5** Authorization codes are cleaned up by a daily cron that DELETEs rows where `expires_at < now() - INTERVAL '1 day'`. The DELETE goes through `oauth_code_cleaner` privileged role since the table is REVOKE'd from cyberos_app.

**§11.6** The Discovery endpoint is publicly accessible (no auth) — RFC 8414 §3.

**§11.7** The Introspection endpoint requires a confidential client with `mcp_introspect` scope (a system scope reserved for resource servers).

**§11.8** Per-tenant redirect host allowlist supports wildcard `*.example.com` matching the leftmost label; `*` alone is forbidden.

**§11.9** Consent records are scoped to (subject, client) — not (subject, tenant). A subject who consents in one tenant context doesn't automatically consent in another.

**§11.10** The audience verification at /tools/call reads the expected canonical URL from env (`MCP_RESOURCE_SERVER_URL`). The env is set at deploy time per resource-server instance.

**§11.11** Tests use the TASK-AUTH-004 testcontainers JWKS fixture for sign + verify.

**§11.12** The token endpoint supports both `client_secret_basic` (RFC 6749 §2.3.1) and `private_key_jwt` (RFC 7523). private_key_jwt requires the confidential client to have registered a JWKS endpoint at DCR time.

**§11.13** Refresh-token rotation: the new token is issued in the same `family_id`; the chain of `parent_token_hash` forms a linked list per family. Family compromise sweeps all rows with that family_id.

**§11.14** The DCR endpoint emits client credentials in the response once. Lost client_secret cannot be recovered; the tenant must DELETE and re-register.

**§11.15** The closed `oauth_error_code` enum maps to RFC 6749 §5.2's exact strings; the JSON response field is `"error"` (not `"error_code"`).

**§11.16** Authorization codes use 256-bit base64url-no-pad (43 chars); refresh tokens use the same shape. Both random via OS RNG.

**§11.17** The `consent_screen` is HTML rendered by the gateway with the client name + scope descriptions; the descriptions come from the TASK-MCP-001 registry.

**§11.18** Token endpoint failure responses include `Cache-Control: no-store` and `Pragma: no-cache` per RFC 6749 §5.2.

**§11.19** The revocation endpoint returns 200 OK regardless of whether the token existed, to prevent probing.

**§11.20** The audience parameter on /authorize defaults to the MCP server origin (`https://<host>`) when not specified. Explicit specification is required when one client targets multiple resource servers within the same tenant.

**§11.21** PKCE constant-time equality prevents timing-channel leakage of the verifier hash.

**§11.22** The implementation uses the `oauth2` Rust crate as a foundation but layers our audience binding + reuse detection + audit emission on top.

---

*End of TASK-MCP-004 spec.*
