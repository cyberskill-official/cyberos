---
id: NFR-MCP-003
title: "MCP audience-bound tokens — OAuth tokens MUST be aud-scoped to the MCP server"
module: MCP
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of MCP-bound tokens carry aud=mcp:<server-id>; cross-audience use rejected"
owner: CTO
created: 2026-05-18
related_frs: [FR-MCP-004, FR-MCP-005]
---

## §1 — Statement (BCP-14 normative)

1. Tokens issued for use against an MCP server **MUST** carry the JWT `aud` claim equal to `mcp:<server-id>` (e.g., `mcp:cyberos-projects`).
2. The MCP server **MUST** reject any token whose `aud` does not match its own server-id with HTTP 401 + `WWW-Authenticate: Bearer error="invalid_token"`.
3. Tokens **MUST NOT** be usable across MCP servers (no shared audience). A token for `mcp:cyberos-projects` is invalid at `mcp:cyberos-okr`.
4. The token issuer (AUTH service) **MUST** require an explicit `audience` parameter on the `/v1/auth/token` request — there is no implicit default.
5. The MCP server **MUST** publish its required `aud` value in its protected-resource metadata document (per FR-MCP-005).

## §2 — Why this constraint

Audience-bound tokens prevent token-theft pivot attacks: even if a token leaks from MCP server A, the attacker can't use it against MCP server B. Without aud-scoping, a compromised tool would have full platform reach. The explicit-audience-required rule prevents the AUTH service from accidentally issuing wildcarded tokens. The published metadata lets clients discover required audience values without trial-and-error.

## §3 — Measurement

- Counter `mcp_token_aud_mismatch_total{server_id, token_aud}`.
- Counter `mcp_token_implicit_audience_request_total` — must be 0.
- Audit row for every rejected token.

## §4 — Verification

- Unit test (T) — issue token with audience A, call server B, assert 401.
- Integration test (T) — proper audience accepted; missing audience refused at issue time.
- CI gate — every MCP server config declares its server-id.

## §5 — Failure handling

- Single aud-mismatch → 401, audit row; expected baseline.
- Burst of mismatches (> 10/min) → sev-3; possible credential leak or misconfigured client.
- AUTH service issuing implicit-audience tokens → sev-2; immediate config fix.

---

*End of NFR-MCP-003.*
