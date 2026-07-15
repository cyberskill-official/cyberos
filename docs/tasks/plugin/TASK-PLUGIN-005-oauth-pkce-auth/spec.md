---
id: TASK-PLUGIN-005
title: "Plugin OAuth-PKCE authentication — install-time authorize + 24h refresh-token rotation against auth.cyberskill.world"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PLUGIN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-002, TASK-PLUGIN-006, TASK-MCP-004, TASK-AUTH-004, TASK-AUTH-007]
depends_on: [TASK-PLUGIN-001, TASK-AUTH-004]
blocks: [TASK-PLUGIN-007]

source_pages:
  - modules/plugin/INTEROP.md universal constraint 3
  - modules/plugin/manifest.schema.json (auth section)
  - docs/tasks/mcp/TASK-MCP-004-oauth-pkce/spec.md (protocol shape)

source_decisions:
  - DEC-2440 2026-05-19 — Plugin auth is OAuth 2.1 + RFC 7636 PKCE — only oauth-pkce in v1, no api-key or basic auth
  - DEC-2441 2026-05-19 — Access tokens are JWT RS256 audience-bound to "plugin:<plugin_id>"; lifetime 1 hour
  - DEC-2442 2026-05-19 — Refresh tokens are opaque, lifetime 24 hours (configurable via manifest.auth.refresh_interval_seconds, min 600)
  - DEC-2443 2026-05-19 — Scopes follow "cyberos:<resource>:<action>" namespace — list locked in this task
  - DEC-2444 2026-05-19 — Install-time consent screen MUST render declared capabilities translated to scope list — user grants explicit
  - DEC-2445 2026-05-19 — Token storage on host filesystem MUST use the host's secret store (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) — fall back to encrypted file with restrictive perms
  - DEC-2446 2026-05-19 — Token theft mitigation: refresh tokens are rotated on every use; tenant admin MAY revoke at AUTH service; revocation propagates within 60 seconds via cache TTL

build_envelope:
  language: rust 1.81
  service: services/plugin-host/
  new_files:
    - services/plugin-host/src/auth/mod.rs
    - services/plugin-host/src/auth/pkce.rs
    - services/plugin-host/src/auth/token_store.rs
    - services/plugin-host/src/auth/refresh.rs
    - services/plugin-host/src/auth/scope_check.rs
    - services/plugin-host/migrations/0001_plugin_auth_grants.sql
    - services/plugin-host/tests/pkce_handshake_test.rs
    - services/plugin-host/tests/jwt_audience_binding_test.rs
    - services/plugin-host/tests/refresh_token_rotation_test.rs
    - services/plugin-host/tests/scope_denial_test.rs
    - services/plugin-host/tests/revocation_propagation_test.rs

  modified_files:
    - services/plugin-host/src/handlers/tools_call.rs (add scope guards)
    - services/plugin-host/src/transport/http.rs (add /oauth/callback redirect handler)
    - services/auth/src/routes.rs (add /oauth/authorize + /oauth/token endpoints — placeholder until TASK-AUTH-007 lands)

  allowed_tools:
    - file_read: services/plugin-host/**
    - file_write: services/plugin-host/{src,tests,migrations}/**
    - bash: cd services && cargo test -p cyberos-plugin-host auth

  disallowed_tools:
    - api-key or basic auth (per DEC-2440)
    - tokens in plaintext on disk (per DEC-2445)
    - cross-plugin audience reuse (per DEC-2441)

effort_hours: 8
subtasks:
  - "0.6h: auth/mod.rs trait + types"
  - "1.5h: auth/pkce.rs (code_verifier/challenge gen, authorize URL build, callback parser)"
  - "1.0h: auth/token_store.rs (Keychain/Credential Manager wrappers + file fallback)"
  - "1.0h: auth/refresh.rs (rotation policy, retry, persist new refresh)"
  - "0.6h: auth/scope_check.rs (per-tool scope matrix)"
  - "0.5h: 0001_plugin_auth_grants.sql"
  - "0.3h: /oauth/callback HTTP handler"
  - "2.5h: 5 test files"

risk_if_skipped: "Without OAuth-PKCE, plugins ship with embedded long-lived API keys — anyone extracting a bundle gets credentials. Strategy §2 'audit-chained' positioning collapses because credentials are static. Without DEC-2441 audience-bound tokens, a stolen plugin token can call other CyberOS APIs. Without DEC-2442 short refresh lifetime, revocation is meaningless. Without DEC-2445 OS-keychain storage, tokens live in plaintext config files and ssh-readable secrets. Without DEC-2446 rotation+revocation, compromise is permanent."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** implement OAuth 2.1 + RFC 7636 PKCE authentication for every plugin tool call. The bridge (TASK-PLUGIN-002) MUST require a valid access token on every `tools/call` request; tokens MUST be obtained via a one-time install-time authorize flow against `auth.cyberskill.world`.

1. **MUST** require OAuth 2.1 PKCE per DEC-2440. Other auth methods (api-key, basic, bearer-static) MUST be rejected at install time by the host (manifest schema enforces; bridge double-checks). PKCE prevents code-interception attacks for native + desktop clients.

2. **MUST** issue access tokens as JWT RS256 with these required claims per DEC-2441:
   - `iss: "https://auth.cyberskill.world"`
   - `aud: "plugin:<plugin_id>"` (e.g. `"plugin:cyberos"`) — audience-bound prevents cross-plugin token reuse
   - `sub: "<subject_uuid>"` — the human or service identity
   - `tenant_id: "<tenant_uuid>"` — tenant scope
   - `scope: "cyberos:cuo:execute cyberos:memory:read ..."` — space-separated scope list
   - `exp: <epoch>` — 1 hour from issue
   - `iat: <epoch>` — issue time
   - `jti: <uuid>` — JWT id (for revocation cache lookup)

3. **MUST** sign access tokens with the AUTH service's JWKS-published key per TASK-AUTH-004. Bridge verifies via cached JWKS (5-minute TTL).

4. **MUST** issue opaque refresh tokens per DEC-2442:
   - 256 bits of CSPRNG entropy
   - 24-hour default lifetime (configurable per-plugin via manifest `auth.refresh_interval_seconds`, min 600)
   - Rotated on every use per DEC-2446 — old refresh invalidated, new refresh issued alongside new access
   - Stored in `services/plugin-host/migrations/0001_plugin_auth_grants.sql` (RLS-protected by tenant_id)

5. **MUST** define and enforce the scope catalogue per DEC-2443:
   ```
   cyberos:cuo:list           — list_personas, list_workflows
   cyberos:cuo:route          — route
   cyberos:cuo:execute        — execute_workflow
   cyberos:memory:read        — read_audit
   cyberos:memory:write       — append_audit
   cyberos:skill:list         — list_catalog
   cyberos:skill:invoke       — invoke_skill
   ```
   A tool call without the required scope MUST return error class `authz_denied` per TASK-PLUGIN-002 clause 7. Scope-vs-tool matrix lives in `auth/scope_check.rs::REQUIRED_SCOPES`.

6. **MUST** present a consent screen at install time per DEC-2444. Hosts that support consent UI (Claude Code, Cowork) render the manifest's declared capabilities translated to scope list; user grants each capability explicitly. The consent screen MUST be host-rendered, not plugin-rendered (trust boundary).

7. **MUST** store tokens via OS-native secret stores per DEC-2445:
   - macOS: Keychain via `security` framework
   - Windows: Credential Manager via `wincred`
   - Linux: Secret Service via `libsecret` D-Bus
   - Fallback: encrypted file at `~/.config/cyberos-plugin/tokens.enc` with `0600` permissions and AES-256-GCM
   The fallback MUST log a warning to TASK-OBS-001 OTel that secret store is unavailable.

8. **MUST** support token revocation per DEC-2446:
   - AUTH service exposes `POST /v1/oauth/revoke?token=<jti>`
   - Bridge maintains a 60-second-TTL revocation cache populated from JWKS-style endpoint
   - On every JWT verify, bridge checks `jti` against revocation cache
   - Revoked tokens fail with `authz_denied` (hint: "token revoked")

9. **MUST** emit memory audit rows per TASK-PLUGIN-006 for every auth event:
   - `plugin.installed` (post-consent, post-first-token-issue)
   - `plugin.auth_refreshed` (every refresh)
   - `plugin.scope_denied` (every scope check failure)
   - `plugin.uninstalled` (revoke + delete tokens)

10. **MUST** implement the PKCE handshake exactly:
    - Client (plugin/bridge) generates 32-byte random `code_verifier`; computes `code_challenge = BASE64URL(SHA256(code_verifier))`
    - Host opens browser to `${authorize_url}?response_type=code&client_id=plugin:<plugin_id>&redirect_uri=<host_callback>&scope=<requested>&state=<csrf_random>&code_challenge=<challenge>&code_challenge_method=S256`
    - User authenticates on auth.cyberskill.world; consents to scopes; AUTH redirects to host callback with `?code=<authorization_code>&state=<csrf_echo>`
    - Host verifies state matches; sends `code_verifier` + `code` to `${token_url}` via POST; AUTH validates challenge ↔ verifier; returns `{access_token, refresh_token, expires_in: 3600}`

11. **MUST NOT** store access tokens longer than their `exp` claim — `auth/token_store.rs` MUST evict expired tokens within 60 seconds.

12. **MUST NOT** expose refresh tokens outside the bridge process — even error logs and OTel spans MUST redact refresh token bytes.

13. **MUST NOT** accept access tokens whose `aud` does not match the calling plugin's id — prevents cross-plugin token reuse.

14. **MUST NOT** reuse PKCE `code_verifier` values — each authorize flow uses a fresh CSPRNG-generated verifier.

---

## §2 — Why this design

**Why OAuth-PKCE only (DEC-2440)?** Static credentials in distributable bundles are a supply-chain liability. PKCE moves the trust from "the bundle holder" to "the user authenticating with AUTH at install time." Even if a bundle is stolen, the attacker cannot mint tokens without the user's authn factor (password + MFA).

**Why audience-bound JWTs (DEC-2441, clause 2)?** A JWT scoped only by signature is reusable anywhere the signer is trusted. Without `aud: "plugin:<id>"`, a token issued for cyberos plugin could be replayed against an unrelated CyberOS service. Audience binding closes this — tools/call handlers MUST verify aud matches their plugin id.

**Why 1-hour access tokens, 24-hour refresh (DEC-2441/2442)?** Industry default. 1-hour access limits the blast radius of token theft to ≤ 1 hour of unauthorised actions. 24-hour refresh gives user-experience continuity without re-prompting. Operators can shorten via manifest.

**Why rotation on every refresh (DEC-2446)?** Without rotation, a stolen refresh token grants attacker indefinite access. With rotation, the legitimate client and attacker diverge — second-use detection at AUTH service catches the duplicate (a feature of OAuth 2.1).

**Why locked scope catalogue (DEC-2443, clause 5)?** Open scope strings invite naming creep ("cyberos.memory.write_v2", "memory:write", etc.). A locked catalogue with a clear pattern (`cyberos:<resource>:<action>`) ensures consistency across the manifest, the consent UI, and the bridge. Adding scopes requires this task or a successor.

**Why mandatory consent screen (DEC-2444, clause 6)?** Without consent, users grant scopes implicitly by installing the plugin. The consent screen makes the grant explicit and reviewable. Strategy §2 ("audit-chained") requires consent be a separate audit event from install.

**Why OS-keychain storage (DEC-2445, clause 7)?** Tokens in plaintext config files are readable by anyone with shell access. OS keychains use hardware-backed encryption where available (macOS Secure Enclave, Windows TPM, Linux kernel keyring). Falling back to encrypted file is acceptable for unattended servers but MUST be logged as a degradation.

**Why 60-second revocation propagation (DEC-2446, clause 8)?** Hard real-time revocation would require bridge → AUTH on every request (latency tax). 60-second cache TTL is the compromise: tenant admin can revoke within a minute, bridge calls AUTH once per minute regardless of request volume. For sensitive operations the cache MAY be bypassed (clause 5 destructive tools could check live).

**Why audit every auth event (clause 9)?** Strategy §2 "open audit chain" requires every credential operation be traceable. Installation, refresh, denial, revocation are all security-relevant. Without audit, debugging "why did this plugin lose access?" is impossible.

**Why expire-and-evict instead of expire-and-error (clause 11)?** Keeping expired tokens in memory invites use-after-expire bugs in handler code. Active eviction makes the invariant compile-time: a token in the store is valid.

**Why aud check separately from signature check (clause 13)?** Signature proves provenance; aud proves intent. A correctly-signed but wrong-audience token is a misrouted token, not a forged one. Different error message, same denial.

---

## §3 — API contract

### Postgres schema for grants

```sql
-- migrations/0001_plugin_auth_grants.sql
CREATE SCHEMA IF NOT EXISTS plugin_host;

CREATE TABLE plugin_host.grants (
  grant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  plugin_id TEXT NOT NULL,
  refresh_token_hash BYTEA NOT NULL,       -- SHA-256 of refresh token; raw never stored
  refresh_token_issued_at TIMESTAMPTZ NOT NULL,
  refresh_token_expires_at TIMESTAMPTZ NOT NULL,
  scopes TEXT[] NOT NULL,
  jti TEXT,                                 -- current access token jti (for revocation cache)
  revoked_at TIMESTAMPTZ,
  trace_id CHAR(32),
  UNIQUE (tenant_id, subject_id, plugin_id)
);
ALTER TABLE plugin_host.grants ENABLE ROW LEVEL SECURITY;
CREATE POLICY grants_rls ON plugin_host.grants
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
CREATE INDEX ON plugin_host.grants (refresh_token_hash);
CREATE INDEX ON plugin_host.grants (jti) WHERE jti IS NOT NULL;
```

### Authorize request URL

```
https://auth.cyberskill.world/v1/oauth/authorize
  ?response_type=code
  &client_id=plugin:cyberos
  &redirect_uri=http://127.0.0.1:7421/oauth/callback   (host-allocated localhost port)
  &scope=cyberos:cuo:execute%20cyberos:memory:read%20cyberos:skill:list
  &state=<random-32-byte-hex>
  &code_challenge=<base64url-sha256-of-verifier>
  &code_challenge_method=S256
```

### Token request

```http
POST /v1/oauth/token HTTP/1.1
Host: auth.cyberskill.world
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=<authorization_code>
&redirect_uri=http://127.0.0.1:7421/oauth/callback
&client_id=plugin:cyberos
&code_verifier=<original-verifier>
```

### Token response

```json
{
  "access_token": "<jwt rs256>",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "<opaque-256-bit>",
  "scope": "cyberos:cuo:execute cyberos:memory:read cyberos:skill:list"
}
```

### Refresh request

```http
POST /v1/oauth/token HTTP/1.1
Host: auth.cyberskill.world
Content-Type: application/x-www-form-urlencoded

grant_type=refresh_token
&refresh_token=<previous-refresh-token>
&client_id=plugin:cyberos
```

Response is identical shape; refresh_token is NEW (rotation).

### Scope-vs-tool matrix (Rust)

```rust
// services/plugin-host/src/auth/scope_check.rs
pub const REQUIRED_SCOPES: &[(&str, &[&str])] = &[
    ("cyberos.cuo.list_personas",   &["cyberos:cuo:list"]),
    ("cyberos.cuo.list_workflows",  &["cyberos:cuo:list"]),
    ("cyberos.cuo.route",           &["cyberos:cuo:route"]),
    ("cyberos.cuo.execute_workflow",&["cyberos:cuo:execute", "cyberos:memory:write"]),
    ("cyberos.memory.read_audit",    &["cyberos:memory:read"]),
    ("cyberos.memory.append_audit",  &["cyberos:memory:write"]),
    ("cyberos.skill.list_catalog",  &["cyberos:skill:list"]),
    ("cyberos.skill.invoke_skill",  &["cyberos:skill:invoke"]),
];
```

---

## §4 — Acceptance criteria

1. **PKCE handshake completes** — end-to-end test: bridge generates verifier, hits authorize, AUTH validates challenge, token issued.
2. **Access token is RS256 JWT** — header.alg == "RS256"; payload has required claims.
3. **aud claim matches plugin id** — `aud == "plugin:cyberos"` for cyberos plugin.
4. **Cross-plugin aud rejection** — token issued for `plugin:cyberos` is rejected when calling `plugin:cyberos-vn` bridge.
5. **Access token lifetime = 3600s** — test checks `exp - iat == 3600`.
6. **Refresh token lifetime = 86400s default** — verify in DB row.
7. **Refresh token rotates on use** — call refresh; old refresh rejected on subsequent use.
8. **JWKS rotation respected** — when AUTH rotates signing key, bridge verifies with new key within 5 minutes (cache TTL).
9. **Scope check denies missing scope** — token with only `cyberos:memory:read` calling `cyberos.memory.append_audit` returns authz_denied.
10. **Multi-scope required tool** — `cyberos.cuo.execute_workflow` requires both `cuo:execute` AND `memory:write`; absent either fails.
11. **PKCE verifier is fresh per flow** — two consecutive authorize flows use different verifiers.
12. **State CSRF check enforced** — callback with mismatched state fails.
13. **Token stored in OS keychain (macOS)** — integration test on macOS verifies Keychain item created.
14. **Encrypted file fallback on Linux without libsecret** — test simulates absence, verifies file at `~/.config/cyberos-plugin/tokens.enc` with mode 0600.
15. **Revocation propagates within 60s** — revoke token; subsequent call within 60s may succeed (cache stale); within 120s MUST fail.
16. **plugin.installed audit emitted at install** — DB query for `kind='plugin.installed'` returns 1 row post-install.
17. **plugin.auth_refreshed audit emitted on refresh** — DB query returns row per refresh.
18. **plugin.scope_denied audit emitted on denial** — DB query for failed scope check.
19. **No refresh token in logs** — grep test on stdout/stderr/OTel for refresh bytes returns 0 matches.
20. **api-key auth method rejected at install** — manifest with `auth.method: "api-key"` fails TASK-PLUGIN-001 validation, install blocked.
21. **PKCE code_verifier ≥ 43 chars** — RFC 7636 §4.1.
22. **code_challenge_method MUST be S256** — `plain` rejected.

---

## §5 — Verification

```rust
// services/plugin-host/tests/pkce_handshake_test.rs
#[tokio::test]
async fn pkce_handshake_end_to_end() {
    let bridge = TestBridge::new().await;
    let (verifier, challenge) = bridge.gen_pkce();
    let auth_url = bridge.build_authorize_url(&challenge);
    assert!(auth_url.contains("code_challenge_method=S256"));
    let code = TestAuth::mock_authorize(&auth_url).await;
    let token = bridge.exchange_code(&code, &verifier).await.unwrap();
    assert!(token.access_token.starts_with("eyJ"));
    let claims = decode_jwt(&token.access_token);
    assert_eq!(claims["aud"], "plugin:cyberos");
    assert!(claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap() == 3600);
}
```

```rust
// services/plugin-host/tests/jwt_audience_binding_test.rs
#[tokio::test]
async fn wrong_audience_rejected() {
    let bridge = TestBridge::new_for_plugin("cyberos-vn").await;
    let token = TestAuth::issue_jwt(json!({"aud":"plugin:cyberos","sub":"...","scope":"..."}));
    let resp = bridge.tools_call_with_token("cyberos.cuo.list_personas", json!({}), &token).await;
    assert_eq!(resp["error"]["data"]["class"], "authz_denied");
    assert!(resp["error"]["data"]["hint"].as_str().unwrap().contains("audience"));
}
```

```rust
// services/plugin-host/tests/refresh_token_rotation_test.rs
#[tokio::test]
async fn refresh_token_rotates() {
    let bridge = TestBridge::new().await.authenticated().await;
    let old_refresh = bridge.current_refresh_token();
    let new = bridge.refresh().await.unwrap();
    assert_ne!(old_refresh, new.refresh_token);
    let resp = bridge.try_refresh_with(&old_refresh).await;
    assert!(resp.is_err());  // old refresh denied
}
```

```rust
// services/plugin-host/tests/scope_denial_test.rs
#[tokio::test]
async fn missing_scope_denies_call() {
    let bridge = TestBridge::new_with_scopes(&["cyberos:memory:read"]).await;
    let resp = bridge.tools_call("cyberos.memory.append_audit", json!({"kind":"x","body":{}})).await;
    assert_eq!(resp["error"]["data"]["class"], "authz_denied");
    let missing: Vec<String> = serde_json::from_value(resp["error"]["data"]["missing_scopes"].clone()).unwrap();
    assert_eq!(missing, vec!["cyberos:memory:write"]);
}

#[tokio::test]
async fn multi_scope_tool_requires_all() {
    let bridge = TestBridge::new_with_scopes(&["cyberos:cuo:execute"]).await;  // missing memory:write
    let resp = bridge.tools_call("cyberos.cuo.execute_workflow", workflow_args()).await;
    assert_eq!(resp["error"]["data"]["class"], "authz_denied");
}
```

```rust
// services/plugin-host/tests/revocation_propagation_test.rs
#[tokio::test]
async fn revocation_propagates_within_60s() {
    let bridge = TestBridge::new().await.authenticated().await;
    TestAuth::revoke(&bridge.current_jti()).await;
    bridge.advance_time(Duration::from_secs(120)).await;
    let resp = bridge.tools_call("cyberos.cuo.list_personas", json!({})).await;
    assert_eq!(resp["error"]["data"]["class"], "authz_denied");
}
```

---

## §6 — Implementation skeleton

(API contract above + Postgres schema are the skeleton. `auth/` directory has 5 files for ~600 lines of Rust.)

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-001 (manifest declares `auth.method: "oauth-pkce"`); TASK-AUTH-004 (JWT/JWKS issuance, shipped).
- **Downstream:** TASK-PLUGIN-002 (bridge enforces scope on every call); TASK-PLUGIN-006 (audit rows for auth events); TASK-PLUGIN-007 (per-runtime adapter handles browser redirect to localhost callback).
- **Cross-module:** TASK-MCP-004 (OAuth-PKCE protocol shape — clause 10 inherits); TASK-AUTH-007 (planned: OAuth-PKCE authorize/token endpoints at AUTH service — placeholder until that task ships).

---

## §8 — Example payloads

(See §3 for authorize URL, token request/response, refresh shape, schema.)

`plugin.installed` memory audit row:
```json
{
  "kind": "plugin.installed",
  "actor_id": "<subject_uuid>",
  "tenant_id": "<tenant_uuid>",
  "body": {
    "plugin_id": "cyberos",
    "plugin_version": "1.0.0",
    "granted_scopes": ["cyberos:cuo:execute","cyberos:memory:read"],
    "consent_screen_version": "v1",
    "trace_id": "01HX..."
  }
}
```

`plugin.scope_denied` audit row:
```json
{
  "kind": "plugin.scope_denied",
  "actor_id": "<subject_uuid>",
  "tenant_id": "<tenant_uuid>",
  "body": {
    "tool": "cyberos.memory.append_audit",
    "missing_scopes": ["cyberos:memory:write"],
    "trace_id": "01HX..."
  }
}
```

---

## §9 — Open questions

All resolved.

- ~~Should refresh token lifetime be a user-set per-grant value?~~ → No, per-plugin via manifest with min 600s per DEC-2442. Per-grant config is operational burden without security gain.
- ~~Should the scope catalogue be extensible per plugin?~~ → No, locked in this task per DEC-2443. New scopes require successor task; keeps host-side consent UI predictable.
- ~~Should we support OAuth Device Code grant for headless servers?~~ → Deferred to task-PLUGIN-005a. v1 is interactive-browser only; headless servers run with pre-baked grants per operations playbook.
- ~~Should JWT verification cache JWKS per-key or whole-document?~~ → Whole-document with 5-minute TTL per clause 3. Per-key complicates rotation; whole-document is simpler with minor latency.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Authorize redirect to non-localhost URI | OAuth spec checks redirect_uri prefix | refuse auth | Use 127.0.0.1 callback only |
| CSRF state mismatch | callback handler | fail with hint | Restart authorize flow |
| Code reuse attempt | AUTH rejects second exchange | error | inherent (one-time-use code) |
| JWT signature invalid | bridge verify | authz_denied | Refresh token; if still bad → AUTH key rotation in flight, retry in 5min |
| JWT exp expired | exp claim check | authz_denied with hint "refresh" | Bridge auto-refreshes; client retries |
| Refresh token expired | DB lookup | authz_denied with hint "re-authenticate" | Trigger consent flow |
| Refresh token reuse | AUTH detects duplicate (rotation invariant) | revoke all related grants | User re-authenticates; admin notified via TASK-OBS-007 |
| Cross-plugin token | aud claim mismatch | authz_denied with hint "audience" | Use the right plugin's token |
| Missing scope | scope_check.rs | authz_denied with missing_scopes array | User re-authorises with broader scope |
| Revoked token within cache TTL | cache stale | request succeeds (≤ 60s window) | Acceptable per DEC-2446; next refresh detects |
| Keychain unavailable | OS API error | warn + fallback to encrypted file | Log to OTel; operator restores keychain |
| Encrypted file permissions wrong | os.stat check | warn + force-chmod 0600 | Inherent fix on next boot |
| Time skew between bridge + AUTH | iat/exp drift | JWT rejected | Use NTP; AUTH MAY tolerate ±30s skew |
| Plugin manifest auth.method != "oauth-pkce" | TASK-PLUGIN-001 schema check | install fails | Author fixes manifest |
| Concurrent refresh race | DB unique constraint on (tenant,subject,plugin) | last writer wins; first refresh response stale | Bridge retries with stale → triggers re-auth |
| Browser cannot open (headless server) | host UI absent | authorize fails | Use task-PLUGIN-005a Device Code flow when shipped |

---

## §11 — Implementation notes

- §11.1 **PKCE library choice.** Rust `oauth2` crate (4.x) implements the spec but doesn't handle our custom audience binding cleanly. Bridge wraps `oauth2` for PKCE primitives, implements aud binding in `auth/pkce.rs`.

- §11.2 **JWKS caching.** `reqwest` GET https://auth.cyberskill.world/.well-known/jwks.json every 5 minutes. Cache in `auth/jwks_cache.rs` as `Arc<RwLock<HashMap<KeyId, RsaPublicKey>>>`. On verify, lookup by `kid` header; on cache miss, refresh once.

- §11.3 **Refresh token rotation logic.** `auth/refresh.rs::refresh()` does atomic: BEGIN → UPDATE grants SET refresh_token_hash=new_hash, refresh_token_issued_at=now() WHERE refresh_token_hash=old_hash → if 0 rows, fail; COMMIT. Concurrent rotation is serialised by the WHERE clause; second writer gets 0 rows and the AUTH service marks it suspicious.

- §11.4 **Keychain abstraction.** `token_store.rs::SecretStore` trait with `MacosKeychain`, `WindowsCredentialManager`, `LinuxSecretService`, `EncryptedFile` impls. Bridge picks one at boot based on target_os + libsecret availability. Falls back with a single OTel warn span.

- §11.5 **Refresh-token-in-logs scrubbing.** OTel layer applies a redaction filter to spans whose attributes match `token`, `refresh`, `auth.bearer`. Set value to `<redacted>` before export. Same filter applies to error.rs message formatting.

- §11.6 **Test mock of AUTH.** Tests use `TestAuth` (in-process) that signs with a fixed RSA keypair. Tests don't hit network.

- §11.7 **Why 60-second revocation TTL specifically.** Less than 60s = AUTH service request volume scales with cache instances (bad). More than 60s = revocation feels slow. 60s is the longest tolerable by typical admin response time.

- §11.8 **Scope expansion path.** task-PLUGIN-005a covers Device Code flow. Task-PLUGIN-005b covers richer scope semantics (per-resource grants like `cyberos:memory:write:project-X`). Both deferred to post-v1.

- §11.9 **Why audience is `plugin:<id>` not `https://...`.** Compact, predictable string. URLs in audience strings create canonicalisation traps (trailing slash, http vs https). `plugin:<id>` is unambiguous.

- §11.10 **Revocation API for tenant admin.** Admin UI (not in scope here) calls `POST /v1/oauth/revoke?token=<jti>&reason=<text>`. Reason persists to memory audit. Operator playbook documents the procedure.

---

*End of TASK-PLUGIN-005 spec.*
