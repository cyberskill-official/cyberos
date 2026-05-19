---
fr_id: FR-MCP-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 10
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-MCP-004 ships OAuth 2.1 + PKCE per the MCP 2025-11-25 auth profile with audience-bound JWT access tokens, opaque rotating refresh tokens with reuse-detection, RFC 7591 DCR, RFC 8414 Discovery, RFC 7009 Revocation, RFC 7662 Introspection. Scope: 30 §1 normative clauses covering closed 2-value `oauth_grant_type` (authorization_code, refresh_token — no implicit/no password/no client_credentials at this slice), closed 2-value `client_type` (public, confidential), closed 6-value `oauth_error_code` (RFC 6749 §5.2 vocabulary), closed 3-value `oauth_code_state` (active, consumed, expired), closed 3-value `oauth_refresh_state` (active, used, compromised), PKCE S256-only (plain rejected per OAuth 2.1 §7.6) MANDATORY on public clients + RECOMMENDED on confidential, audience binding via `aud` claim per RFC 8707 + exact-match check at MCP resource server (no cross-server replay), refresh-token rotation MANDATORY with reuse → entire family marked compromised + sev-1 audit, authorization code TTL 30 seconds (tighter than OAuth 2.1's 10-min recommend) + one-time-use enforced via `consumed_at` FOR UPDATE + reuse → family compromised + sev-2 audit, exact-match redirect_uri comparison (no substring/regex/normalization), per-tenant `mcp_oauth_allowlist_redirect_hosts` policy with tenant_admin role gate + sev-2 audit, HTTPS-only redirect URIs except http://localhost or http://127.0.0.1 for native CLI (OAuth 2.1 §10.3.3), `state` parameter REQUIRED on every authorize request (CSRF defense), JWT signed with FR-AUTH-004 JWKS RS256/ES256 with iss + aud + sub + scope + nonce + iat + exp + jti + client_id + tenant_id claims, access TTL 1h + refresh TTL 30d, revocation via jti list cached 60s + JWKS key rotation, RFC 7009 always-200 (no probing surface), RFC 7662 Introspection requires confidential client with `mcp_introspect` scope (no public exposure), RFC 8414 Discovery at /.well-known/oauth-authorization-server (separate from FR-MCP-005 PRM), RFC 7591 DCR with public clients self-registered (no caller auth) + confidential requires tenant_admin + max 5 redirect_uris + 1-1024 char scope, prompt=none for silent re-auth + login_required when no session, consent screen on first scope grant + skip on subsequent same-scope + new scope re-consent, scope validated against FR-MCP-001 `tools/list` registry (closed vocabulary), append-only via SQL grants (oauth_codes UPDATE limited to consumed_at via oauth_code_consumer role, oauth_refresh_families UPDATE limited to state/state_changed_at via oauth_refresh_writer, oauth_revocation_list INSERT/SELECT only), tenant isolation for confidential clients (cross-tenant lookup returns 404), constant-time PKCE equality (timing-channel defense), 8 closed memory audit kinds (oauth_authorize_started sev-3, oauth_token_issued sev-3, oauth_token_refreshed sev-3, oauth_token_revoked sev-2, oauth_refresh_reuse_detected sev-1, oauth_code_reuse_detected sev-2, oauth_audience_mismatch sev-2, oauth_client_registered sev-2), all reason text scrubbed via FR-MEMORY-111. 22 rationale paragraphs. §3 contains: 3 migrations (oauth_clients with all closed enums + CHECK constraints for confidential-has-secret + redirect_uris_max_5 + confidential_has_tenant; oauth_codes with code_challenge length + S256-only CHECK + audience column + grants; oauth_refresh_families with unique token_hash + family_id + state-transition grant + oauth_revocation_list with jti PK + oauth_consents per subject+client), constant-time PKCE verifier, token handler with FOR UPDATE lock + one-time-use enforcement + family-compromise transition + refresh-rotation, audience verification at /tools/call hot path. 32 ACs. 33 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Implicit grant or password grant included (OAuth 2.1 violation)
First-pass included both. Resolved: §1 #1 + DEC-800 + DEC-807 + closed 2-value enum (authorization_code, refresh_token) + 400 unsupported_grant_type for everything else; AC #1 cardinality.

### ISS-002 — PKCE optional or plain method accepted
First-pass had `code_challenge_method = plain` valid. Resolved: §1 #2 + DEC-801 + S256 only + 400 pkce_method_must_be_s256 + DB CHECK constraint; AC #5 + #6.

### ISS-003 — No audience binding (cross-server token replay)
First-pass had no `aud` claim. Resolved: §1 #7 + §1 #23 + DEC-802 + RFC 8707 + aud claim required + exact-match at resource server + 401 audience_mismatch + sev-2 audit; AC #17.

### ISS-004 — Refresh tokens long-lived without rotation (stolen = indefinite access)
Resolved: §1 #9 + DEC-806 + rotation MANDATORY + state machine (active → used → compromised) + reuse detection invalidates entire family + sev-1 audit; AC #15 + #16.

### ISS-005 — Authorization code reuse undetected
Resolved: §1 #15 + DEC-812 + SELECT FOR UPDATE + consumed_at one-time check + reuse marks refresh family compromised + sev-2 audit; AC #11.

### ISS-006 — Redirect_uri substring matching (open-redirect)
Resolved: §1 #10 + DEC-809 + exact-match string compare + AC #8 substring rejected.

### ISS-007 — HTTP non-loopback redirect URIs registered
Resolved: §1 #12 + URL parser check at registration + allow only http://localhost or http://127.0.0.1 per OAuth 2.1 §10.3.3; AC #9.

### ISS-008 — Authorization code TTL too loose (10-min default invites theft)
Resolved: §1 #14 + DEC-811 + 30-second TTL + expires_at CHECK + invalid_grant code_expired; AC #10.

### ISS-009 — Introspection endpoint publicly accessible (token probing)
Resolved: §1 #22 + DEC-818 + requires confidential client + `mcp_introspect` scope + 401 for public clients; AC #21.

### ISS-010 — Open DCR without redirect_uri host restriction
Resolved: §1 #11 + DEC-816 + per-tenant `mcp_oauth_allowlist_redirect_hosts` policy + tenant_admin gate on policy + sev-2 audit on policy mutation + sev-3 on registration rejection; AC #29.

## §3 — Resolution

All 10 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (OAuth 2.1 baseline × authorization_code + refresh_token only × PKCE S256-only mandatory on public × audience binding RFC 8707 × refresh rotation with family-compromise detection × 30-second code TTL × one-time-use code with reuse → family compromise × exact-match redirect_uri × HTTPS-only except loopback × state CSRF param × JWT signed via FR-AUTH-004 JWKS × 1h access + 30d refresh × revocation via jti list cached 60s × RFC 7009 always-200 × RFC 7662 introspection confidential-only × RFC 8414 discovery × RFC 7591 DCR with tenant_admin gate on confidential × max 5 redirect_uris × per-tenant redirect-host allowlist × prompt=none silent re-auth × consent screen first-time × scope from FR-MCP-001 registry × append-only SQL grants × tenant isolation × constant-time PKCE × 8 closed memory audit kinds × FR-MEMORY-111 PII scrubbing), not by line targets.

---

*End of FR-MCP-004 audit.*
