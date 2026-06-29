---
fr_id: FR-AUTH-110
audited: 2026-06-29
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_revision: 9.5/10
issues_resolved: 8
issues_open: 1 (impl-time decision, non-blocking)
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (mirrors FR-AUTH-104 structure; ≥6 canonical ISSes; first-party-IdP profile)
---

## §1 - Verdict summary

FR-AUTH-110 makes AUTH a first-party OIDC provider so CHAT (Mattermost) and PORTAL federate to one CyberOS identity - the inverse of FR-AUTH-104. Scope: 26 normative clauses over 4 tables (rp_clients + auth_codes single-use + sso_sessions + append-only op_login_history), OIDC + RFC 8414 discovery, authorize that brokers the human via an SSO cookie or upstream Google (FR-AUTH-104) and is revoke-gated, token exchange minting id_token + access_token against the existing FR-AUTH-004 keys, userinfo, an admin-only first-party RP registry with locked redirect_uris and one-time secret reveal, PKCE S256 mandatory with implicit/hybrid forbidden, exact redirect_uri match with reject-without-redirect, a pinned issuer checked at boot, and 7 memory audit kinds. The design's defining strength is honesty about the kick boundary: slice 1 blocks re-authentication and revokes the SSO session, slice 2 (back-channel logout) kills live downstream sessions. It reuses the FR-MCP-004 OAuth substrate rather than rebuilding it. The verdict is PASS; one item is left as a deliberate impl-time decision, recorded below.

## §2 - Findings

### ISS-001 - The kick was overclaimed (RESOLVED)
A first draft implied "revoke = instant logout everywhere". That is false for Mattermost, which mints its own session after the OIDC handshake. Resolved: DEC-2488 + §1 #13 + §1 #26 + §9 state the boundary plainly - slice 1 blocks re-auth and revokes the AUTH SSO session; live downstream sessions need back-channel logout (slice 2). The FR does not pretend otherwise. This honesty is required, not a weakness.

### ISS-002 - Two key systems would fragment verification (RESOLVED)
A provider that minted its own signing keys would publish a second JWKS and drift on rotation. Resolved: DEC-2481 + §1 #10 + disallowed_tools forbid a second key system; id_token + access_token sign against FR-AUTH-004 `auth_signing_keys` and the one JWKS.

### ISS-003 - Open redirect via loose redirect_uri (RESOLVED)
Resolved: DEC-2491 + §1 #12 + AC #3 + the exact-match test - byte-exact against the registered set, and on mismatch the provider renders an error rather than redirecting anywhere (redirecting to an unverified URI is the vulnerability itself).

### ISS-004 - Code interception / replay (RESOLVED)
Resolved: DEC-2484 (PKCE S256 mandatory, implicit/hybrid forbidden) + DEC-2490 (codes hashed, single-use, 60s) + §1 #8 (replay of a consumed code revokes its tokens, RFC 6749 §4.1.2) + the single-use test.

### ISS-005 - Revoked user keeps re-authenticating (RESOLVED)
Resolved: DEC-2488 - the revoke gate runs at authorize AND token AND userinfo, and DEC-2496/§1 #26 cascade the revoke onto the SSO session so silent SSO also stops. This is the actual kick-by-revoke Stephen asked for, scoped honestly.

### ISS-006 - Issuer drift breaks every RP (RESOLVED)
A trailing-slash difference between discovery `issuer` and id_token `iss` makes every downstream validator reject every token. Resolved: DEC-2498 + §1 #24 + AC #17 - one configured canonical value, checked at boot, provider refuses to start on mismatch.

### ISS-007 - Self-registration would widen the trust boundary (RESOLVED)
Open Dynamic Client Registration for first-party apps is unnecessary attack surface. Resolved: DEC-2483 + §1 #15 - RPs are admin-registered confidential clients; DCR is explicitly out of slice 1 (it is the MCP gateway's job for agents).

### ISS-008 - SSO cookie fixation / scope leak (RESOLVED)
Resolved: DEC-2489 + §1 #9 - server-side session table is truth (revocable), the cookie is the `__Host-` strict form (Secure, HttpOnly, SameSite=Lax, Path=/, no Domain), sliding 8h / absolute 24h.

## §3 - Open item (non-blocking, impl-time)

### OPEN-001 - consume-without-UPDATE pattern + extract-vs-reimplement
Two implementation choices are left to the build, deliberately:
1. `auth_oidc_auth_codes` revokes UPDATE/DELETE for forensic integrity, so "consume" cannot be a guarded `UPDATE ... WHERE consumed_at IS NULL`. §3.2 proposes a sibling `auth_oidc_code_consumptions(code_hash PK)` where the first insert wins and a second raises a unique violation (= replay). The alternative is to grant a single narrow UPDATE on `consumed_at`. Either is sound; pick at build with a one-line ADR.
2. DEC-2493 reuse: extract `services/mcp-gateway/src/oauth/` into a shared crate vs re-implement the thin parts in auth. The FR mandates reuse; the mechanism is an impl decision. Neither choice changes the contract, so this does not block the spec.

## §4 - Resolution

8 of 8 mechanical concerns resolved in the FR. The one open item is a pair of implementation choices that do not change the API contract or the security properties. The spec's depth is bounded by the genuine surface (OIDC provider profile × human-auth brokering × revoke-gated authorize × single-use PKCE codes × SSO cookie × first-party RP registry × pinned issuer × honest kick boundary), not by a line target.

**Score = 9.5/10.** The half point withheld only until OPEN-001's two choices are pinned with one-line ADRs at build time.

---

*End of FR-AUTH-110 audit.*
