---
id: NFR-MCP-007
title: "MCP OAuth PKCE — authorization code flow MUST require PKCE; no implicit grant"
module: MCP
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of authorization-code requests carry valid PKCE challenge; implicit grant disabled"
owner: CTO
created: 2026-05-18
related_frs: [FR-MCP-004]
---

## §1 — Statement (BCP-14 normative)

1. The MCP-AUTH endpoint **MUST** require PKCE (RFC 7636) for every authorization code request — `code_challenge` + `code_challenge_method=S256` are mandatory.
2. The implicit grant type **MUST NOT** be supported — `/authorize` requests with `response_type=token` are rejected.
3. The `code_verifier` **MUST** be validated on token exchange; mismatches return `invalid_grant`.
4. The `code` itself **MUST** be single-use, ≤ 60s TTL, bound to the redirect_uri.
5. `code_challenge_method=plain` **MUST** be rejected — S256 only.

## §2 — Why this constraint

PKCE eliminates the authorization-code interception attack class, especially in public-client (CLI, desktop, mobile) contexts. Forcing S256 (no plain) closes the downgrade attack. Disabling implicit grant follows OAuth 2.1 best practice — implicit is fundamentally insecure for token issuance. The single-use + short-TTL + redirect-uri-bound code closes the replay window.

## §3 — Measurement

- Counter `mcp_oauth_pkce_missing_total` — must be 0.
- Counter `mcp_oauth_implicit_grant_attempt_total` — must be 0.
- Counter `mcp_oauth_code_replay_total` — must be 0 (replay attempts indicate active attack).

## §4 — Verification

- Integration test `modules/mcp/tests/test_oauth_pkce.py` (T) — missing PKCE → reject; valid PKCE → success.
- Pen test (T, quarterly) — attempt PKCE downgrade + implicit grant; assert blocked.

## §5 — Failure handling

- Missing PKCE → 400 invalid_request; expected; counter should stay near 0 (legit clients always include).
- Implicit grant attempted → 400; counter > 0 may indicate misconfigured legacy client.
- Code replay → 400 invalid_grant; audit + investigate possible token leak.

---

*End of NFR-MCP-007.*
