# TASK-MCP-004 slice 2 - OAuth 2.1 authorization server build plan

Written 2026-06-25. Slice 1 shipped the PKCE S256 verifier and the RFC 6749 error type. This slice
builds the authorization server. The pure, database-free foundation is authored and unit-tested in
`services/mcp-gateway/src/oauth/`; the database-bound endpoints below are the remaining work, built at
the toolchain (they need a live Postgres for the integration tests, not for compilation).

## What is done (this commit) - pure, unit-tested, compiles without a DB

- `oauth/enums.rs` - closed `GrantType` (2), `ClientType` (2), `CodeState` (3), `RefreshState` (3),
  each with a cardinality tripwire test and a wire round-trip (DEC-807/808, clauses #5/#4/#16/#17).
- `oauth/audience.rs` - `bind_audience`, `audience_matches` (exact, case-sensitive, fail-closed), and
  the `WWW-Authenticate` challenge. The cross-server-replay defense (DEC-802, RFC 8707, clauses
  #7/#23), with the substring / trailing-slash / case / empty traps pinned by tests.
- `oauth/scope.rs` - RFC 6749 §3.3 scope-token syntax + closed-set membership against the registry
  (DEC-813, clause #30).
- `oauth/error.rs`, `oauth/pkce.rs` - from slice 1.

## Key architecture decisions (grounded in the codebase, not invented)

1. Token signing reuses TASK-AUTH-004 keys, no new keypair. The auth service stores RS256 signing keys
   in the shared Postgres table `auth_signing_keys (kid, algorithm, public_pem, private_pem, status,
   expires_at, activated_at, retired_at)` and publishes the public halves at `/.well-known/jwks.json`
   (see `services/auth/src/jwt.rs`). The gateway mints OAuth access tokens by loading the active key
   from that same table and signing with `jsonwebtoken` (already a gateway dependency), `kid` in the
   header. Resource servers then verify against the auth JWKS. Algorithm is RS256 only - the auth code
   is RS256-only despite the spec saying "RS256 or ES256", so match it.

2. Runtime-checked sqlx. Every other service uses `sqlx::query(...)` / `query_as(...)` (runtime
   checked), not the `query!` macro, and there is no `.sqlx` offline cache. Follow that: the code
   compiles with no `DATABASE_URL` and no live DB; only the integration tests connect to Postgres.

3. Access token = JWT, refresh token = opaque. Access tokens are signed JWTs with the claim set in
   clause #7 (`iss`, `aud` = resource URL, `sub`, `scope`, `nonce`, `iat`, `exp = iat + 3600`, `jti`,
   `client_id`, `tenant_id`). Refresh tokens are opaque 256-bit base64url strings stored hashed
   (SHA-256) in `oauth_refresh_families` (clauses #7/#8).

## Tables (migrations)

The three OAuth migrations were renumbered off 0010-0012 (the backlog reserves those for TASK-MCP-007
tasks and TASK-MCP-008 elicitation) to 0013-0015. Do the rename as a git move so history follows:

```bash
cd ~/Projects/CyberSkill/cyberos/services/mcp-gateway/migrations
git mv 0010_oauth_clients.sql          0013_oauth_clients.sql
git mv 0011_oauth_codes.sql            0014_oauth_codes.sql
git mv 0012_oauth_refresh_families.sql 0015_oauth_refresh_families.sql
```

Then reconcile each file's columns against the spec before wiring:
- `oauth_clients` - `(client_id uuid pk, client_type, client_secret_hash null, redirect_uris text[],
  client_name, scope text[], tenant_id, created_at)`. confidential carries a secret hash; public does
  not (clause #18/#19).
- `oauth_codes` - `(code, client_id, subject_id, redirect_uri, code_challenge, scope, nonce, state,
  issued_at, expires_at, consumed_at)`; one-time-use via `SELECT ... FOR UPDATE` + `consumed_at`
  (clauses #14/#15/#16/#27).
- `oauth_refresh_families` - `(family_id, token_hash, parent_token_hash null, state, issued_at,
  expires_at)`; rotation + reuse detection (clauses #8/#9/#17).

Three tables the spec needs that are not in the original three - add as new migrations 0016-0018:
- `oauth_consents (subject_id, client_id, scopes text[], granted_at)` (clause #29).
- `oauth_revocations (jti, revoked_at)` - the access-token revocation list, 1h cache (clauses #21/#24).
- per-tenant `mcp_oauth_allowlist_redirect_hosts` policy (clause #11) - a column on a tenant policy
  table or its own table.

## Endpoints (each is one source file under `oauth/`, plus a `tests/oauth_*_test.rs`)

- `dcr.rs` -> `POST /register` (RFC 7591). Validate `client_type`, `redirect_uris` (<=5, https or
  loopback http per clause #12, exact-host allowlist per clause #11), `scope` (use `scope::validate`).
  Public is open; confidential requires `tenant_admin` (reuse the auth RBAC middleware). Emit
  `mcp.oauth_client_registered` (clauses #18/#19).
- `authorize.rs` -> `GET /authorize` -> 302. Require `state` (clause #13), PKCE `code_challenge` +
  `code_challenge_method=S256` for public clients (clauses #2/#3, reject plain), exact-match
  `redirect_uri` (clause #10). Handle `prompt=none` silent re-auth (clause #28) and the consent screen
  (clause #29). Mint a 30s one-time code into `oauth_codes`. Emit `mcp.oauth_authorize_started`.
- `token.rs` -> `POST /token`. `grant_type` via `enums::GrantType::from_wire` (else
  `unsupported_grant_type`). For `authorization_code`: `SELECT ... FOR UPDATE` the code, check
  `consumed_at IS NULL` (else replay -> compromise family + `mcp.oauth_code_reuse_detected`), verify
  PKCE with `pkce::verify_pkce`, check expiry (clause #14), mint the JWT (see `jwt.rs`) + an opaque
  refresh token. Emit `mcp.oauth_token_issued`.
- `refresh.rs` -> the `refresh_token` branch of `/token`. Look up the presented token's hash; if
  `state = used` -> reuse -> mark family `compromised`, refuse, `mcp.oauth_refresh_reuse_detected`
  (sev-1). Else mark it `used`, mint a new pair with `parent_token_hash` set. Emit
  `mcp.oauth_token_refreshed` (clause #9).
- `revoke.rs` -> `POST /revoke` (RFC 7009). access -> add `jti` to `oauth_revocations`; refresh ->
  mark family `compromised`. Always 200. Emit `mcp.oauth_token_revoked` (clause #21).
- `introspect.rs` -> `POST /introspect` (RFC 7662). Confidential client + `mcp_introspect` scope only;
  public -> 401. RFC 7662 §2.2 response shape (clause #22).
- `discovery.rs` -> `GET /.well-known/oauth-authorization-server` (RFC 8414). Static document from
  clause #20 with `scopes_supported` filled from the `tools/list` registry.
- `jwt.rs` (new) - load the active key from `auth_signing_keys` (mirror `auth::jwt::load_active_key`),
  mint RS256 with the clause-#7 claims and `aud = audience::bind_audience(resource)`. A pure
  `build_claims(...)` is unit-testable; the sign step is exercised by the integration test with a key
  from the DB.

## The hot-path security wire (clause #23) - do not skip

In `protocol/tools_call.rs`, before dispatch: decode the bearer JWT against the auth JWKS, then
`if !audience::audience_matches(&claims.aud, this_server_canonical_url) { 401 +
audience::audience_mismatch_challenge(); emit mcp.oauth_audience_mismatch; }`. Also check `exp`,
signature, and the `jti` revocation list (60s in-process cache, clause #24). This is the property the
whole FR exists for - an access token only works at the resource server it was minted for.

## Audit + scrubbing

Emit the 8 closed audit kinds (clause #25) to the memory chain the way obs-router does
(`obs.alert_triaged` etc.) - an async POST to the memory audit endpoint, reason text routed through
TASK-MEMORY-111 scrubbing (clause #26). This is the only async/DB-touching part of the otherwise-pure
audience and scope logic, so keep it at the handler edge, not inside the pure functions.

## Verify

```bash
cd ~/Projects/CyberSkill/cyberos/services
cargo test -p cyberos-mcp-gateway                       # unit tests incl. the new oauth foundation
cargo clippy -p cyberos-mcp-gateway --all-targets -- -D warnings
# integration tests (need Postgres with the auth + oauth migrations applied):
DATABASE_URL=postgres://... cargo test -p cyberos-mcp-gateway --test 'oauth_*'
```

## Out of scope for slice 2

`client_credentials` grant (deferred to TASK-MCP-007). ES256 (auth is RS256-only). TASK-MCP-005 Protected
Resource Metadata is a separate FR that depends on this one.
