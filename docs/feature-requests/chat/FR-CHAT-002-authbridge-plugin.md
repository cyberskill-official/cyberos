---
id: FR-CHAT-002
title: "cyberos-chat-authbridge plugin — Mattermost auth delegates to FR-AUTH-004 JWT with tenant_id propagation and SCIM-free provisioning"
module: CHAT
priority: MUST
status: closed
superseded_by: [FR-AUTH-110, FR-CHAT-013]
closed: 2026-06-29
closed_reason: "superseded - the AuthBridge plugin approach does not work (a Mattermost plugin cannot replace the core login route, and the shipped plugin is a non-working simulation); the unified path is Mattermost's native OIDC connector federating to the FR-AUTH-110 provider, specified in FR-CHAT-013"
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-CHAT-001, FR-CHAT-003, FR-AUTH-004, FR-AUTH-005, FR-AUTH-110]
depends_on: [FR-CHAT-001, FR-AUTH-004]
blocks: [FR-CHAT-003]

source_pages:
  - website/docs/modules/chat.html#auth
source_decisions:
  - DEC-430 (Mattermost auth replaced by JWT validation against FR-AUTH-004 JWKS; no Mattermost users table writes)
  - DEC-431 (just-in-time user provisioning on first login; no SCIM dependency)
  - DEC-432 (tenant_id propagation via JWT custom claim; Mattermost team mapped 1:1 to tenant)

language: go 1.22
service: cyberos/services/chat/
new_files:
  - services/chat/plugins/cyberos-authbridge/main.go
  - services/chat/plugins/cyberos-authbridge/jwks_cache.go
  - services/chat/plugins/cyberos-authbridge/jit_provision.go
  - services/chat/plugins/cyberos-authbridge/plugin.json
  - services/chat/plugins/cyberos-authbridge/Makefile
  - services/chat/plugins/cyberos-authbridge/tests/auth_test.go
modified_files:
  - services/chat/patches/010-disable-builtin-auth.patch
  - services/chat/patches/011-load-authbridge-plugin.patch
allowed_tools:
  - file_read: services/chat/**
  - file_write: services/chat/plugins/cyberos-authbridge/**
  - bash: cd services/chat/plugins/cyberos-authbridge && make
disallowed_tools:
  - allow Mattermost's built-in password flow (per DEC-430)
  - skip JWT signature verification (per FR-AUTH-004)

effort_hours: 10
sub_tasks:
  - "0.5h: plugin.json manifest"
  - "1.0h: main.go — HTTP middleware intercepting /api/v4/users/login and /api/v4/users/me/sessions"
  - "1.0h: jwks_cache.go — fetch + cache FR-AUTH-004 /.well-known/jwks.json; 1h TTL"
  - "1.5h: JWT validation per FR-AUTH-004 spec (signature, exp, nbf, iss, tenant_id claim required)"
  - "1.5h: jit_provision.go — first-login creates Mattermost user; binds to AUTH-004 subject_id"
  - "1.0h: team mapping — JWT tenant_id → Mattermost team_id (lookup in cyberos_chat_tenant_map table)"
  - "0.5h: patches/010-disable-builtin-auth.patch — strip Mattermost's password endpoint"
  - "0.5h: patches/011-load-authbridge-plugin.patch — auto-load plugin at boot"
  - "2.0h: auth_test.go — happy + invalid JWT + tenant mismatch + JIT provision + replay JWT"
  - "0.5h: Makefile — build plugin .tar.gz for Mattermost upload"
risk_if_skipped: "Without authbridge, Mattermost runs its own user database — two sources of truth (AUTH-004 + Mattermost). Users get confused (different password per service). Token revocation requires double-cleanup. Without tenant_id propagation, Mattermost shows messages from all tenants in one team. JIT provisioning means no IT-admin friction on user onboarding."
---

> CLOSED - SUPERSEDED (2026-06-29). This FR is retained for history only; do not implement it. The
> AuthBridge plugin it specifies cannot work: a Mattermost plugin cannot replace the core
> `/api/v4/users/login` route (plugins only serve under `/plugins/<id>/`), and the shipped
> `services/chat/plugins/cyberos-authbridge/` is a non-working simulation - it does not use the Mattermost
> plugin SDK, its `JitProvisioner` writes in-memory map entries rather than real Mattermost users, and the
> "session" it returns is a fabricated string, not a Mattermost session. The unified path is Mattermost's
> own native OIDC connector federating to the FR-AUTH-110 CyberOS OIDC provider, specified in
> [[FR-CHAT-013]]. The DEC-430/431/432 ids below also collided with FR-AUTH-110's first draft and were
> renumbered there (DEC-2480+); these originals stand as the CHAT-002 record.

## §1 — Description (BCP-14 normative)

The authbridge plugin **MUST** delegate Mattermost authentication to FR-AUTH-004 JWT with tenant propagation. The contract:

1. **MUST** install as a Mattermost plugin (`plugin.json` + Go binary); auto-loaded on Mattermost boot via FR-CHAT-001 patch `011-load-authbridge-plugin.patch`.
2. **MUST** intercept Mattermost's `POST /api/v4/users/login`:
    - Extract JWT from `Authorization: Bearer <token>` header.
    - Validate against FR-AUTH-004 JWKS (signature + `exp` + `nbf` + `iss`).
    - JWT MUST carry custom claim `tenant_id` (UUID); missing → 401.
    - On success: create Mattermost session bound to JIT-provisioned user.
3. **MUST** disable Mattermost's built-in password flow via patch `010-disable-builtin-auth.patch`:
    - Endpoint `/api/v4/users/login` with password body → 405 METHOD NOT ALLOWED.
    - Endpoint `/api/v4/users/password/reset/send` → 410 GONE.
    - Endpoint `/api/v4/users/email/verify` → 410 GONE.
4. **MUST** cache JWKS in plugin memory with 1h TTL; refresh on cache miss. Background refresh thread at 50min interval (proactive refresh).
5. **MUST** JIT-provision Mattermost user on first valid JWT:
    - Lookup `mm_users WHERE props.cyberos_subject_id = <subject>`; if exists → reuse.
    - Otherwise: insert with username = JWT `email` localpart, props = `{cyberos_subject_id: <subject>, cyberos_tenant_id: <tenant>}`.
    - Add user to Mattermost team corresponding to `tenant_id` (lookup in `cyberos_chat_tenant_map(tenant_id UUID, mm_team_id TEXT)`).
6. **MUST** propagate tenant_id to every Mattermost API call via plugin middleware: every request enriches `c.AppContext.Session().Props["tenant_id"] = jwt.tenant_id`; downstream plugins/handlers read it.
7. **MUST** reject when JWT's tenant_id does NOT match the user's existing Mattermost team:
    - User switched tenants → 403 with `{"error":"tenant_mismatch","reason":"jwt_tenant_id != mm_user_team"}`.
    - Operator action: contact admin to re-add user to correct team.
8. **MUST** validate JWT `jti` against Mattermost's session blocklist (FR-AUTH-004 revocation list); revoked → 401.
9. **MUST** emit memory audit row `chat.session_started` per successful login with `{mm_user_id, cyberos_subject_id, tenant_id, jit_provisioned, trace_id}`.
10. **MUST** emit OTel metrics:
    - `chat_authbridge_logins_total{outcome}` (outcome ∈ ok | invalid_jwt | tenant_mismatch | revoked).
    - `chat_authbridge_jit_provisions_total`.
    - `chat_authbridge_jwks_cache_hits_total{result}`.
11. **MUST** be Mattermost-plugin-compatible (no `init()` side effects; respect plugin lifecycle hooks `OnActivate` + `OnDeactivate`).
12. **MUST** emit only error envelopes whose `error` field is one of the 9 values in §3 "Error-envelope contract"; any new error value MUST extend the enum AND have a matching CI test (AC #18). The envelope MUST always carry `trace_id` so users can quote it back to support.
13. **MUST NOT** create more than one Mattermost user per `subject_id` even under concurrent first-login bursts (AC #19). Idempotency is enforced by an in-process per-subject mutex plus a defensive `findUserBySubject` recheck inside the mutex.
14. **MUST** sanitise the username derived from JWT `email`: lowercase, `[a-z0-9._-]` only, 3-22 chars, suffix `_N` on collision (AC #20). The sanitised username MUST be deterministic per email — same input always yields the same output (subject to collision suffix).
15. **MUST** fail-secure when the JWKS endpoint is unreachable AND the cache is cold (AC #21): return 503 `jwks_unavailable`, not 200. A 200 would imply "I verified the signature" — which is false if no key material is available.
16. **MUST** honour the inbound `traceparent` header (AC #26). If present and well-formed, the audit row's `trace_id` MUST equal the inbound header's `trace-id` field. If absent, the plugin MUST mint a fresh 32-char W3C trace id at the trust boundary.
17. **MUST** fail-secure when the revocation lookup service is unreachable (AC #27): treat the token as revoked and deny login. This is intentional — uptime of the auth-revocation path is more important than uptime of the login path. Operators are alerted via SEV-2 when this happens at >1/s.
18. **MUST** reject double `OnActivate` calls (AC #29): the plugin uses an `atomic.Bool` to record activation state and returns "already activated" on the second call. This guards against Mattermost SDK regressions that could call OnActivate twice and spawn duplicate JWKS goroutines.
19. **MUST** ship a reproducible build (AC #30): the `Makefile` target uses `-trimpath -ldflags '-buildid='` and a deterministic `tar` invocation so two consecutive builds from identical source produce byte-identical `.tar.gz` artefacts. This supports supply-chain attestation and SBOM correlation.

---

## §2 — Why this design (rationale for humans)

**Why plugin not patch (§1 #1)?** Plugin = upgradeable independently; patch = pinned to fork commit. Plugin pattern is canonical for upstream-compatible extensions.

**Why disable built-in auth (DEC-430)?** Single source of truth. Mattermost's email-password flow + AUTH-004 JWT = two paths users can take; auditors can't reason about "who logged in via X." Disable enforces JWT-only.

**Why JIT provisioning (DEC-431)?** SCIM is heavy + requires AD/Okta integration. JIT = create-on-first-login; works for any AUTH-004-issued JWT.

**Why team = tenant (DEC-432)?** Mattermost's team primitive maps cleanly to CyberOS's tenant. Cross-tenant chat is forbidden; team boundary enforces.

**Why JWKS cache (§1 #4)?** Fetching JWKS per login = latency + AUTH-004 load. 1h cache + proactive refresh = no per-login network call typical.

**Why tenant_mismatch returns 403 (§1 #7)?** Distinguishes "you're not allowed" from "you're not authenticated." 401 implies fix-your-token; 403 implies fix-your-permissions.

**Why audit per login (§1 #9)?** Compliance trail; investigators ask "who logged in when from where."

---

## §3 — API contract (plugin sketch)

```go
// services/chat/plugins/cyberos-authbridge/main.go
package main

import (
    "encoding/json"
    "net/http"

    "github.com/mattermost/mattermost/server/public/plugin"
    "github.com/golang-jwt/jwt/v5"
)

type AuthBridgePlugin struct {
    plugin.MattermostPlugin
    jwksCache *JwksCache
}

func (p *AuthBridgePlugin) OnActivate() error {
    jwks_url := p.API.GetConfig().PluginSettings.Plugins["cyberos.authbridge"]["jwks_url"].(string)
    p.jwksCache = NewJwksCache(jwks_url, time.Hour)
    return nil
}

func (p *AuthBridgePlugin) ServeHTTP(c *plugin.Context, w http.ResponseWriter, r *http.Request) {
    if r.URL.Path == "/login" && r.Method == http.MethodPost {
        p.handleLogin(c, w, r)
        return
    }
    http.NotFound(w, r)
}

func (p *AuthBridgePlugin) handleLogin(c *plugin.Context, w http.ResponseWriter, r *http.Request) {
    authz := r.Header.Get("Authorization")
    tokenStr := strings.TrimPrefix(authz, "Bearer ")
    if tokenStr == authz { http.Error(w, "missing bearer", 401); return }

    claims := jwt.MapClaims{}
    _, err := jwt.ParseWithClaims(tokenStr, &claims, p.jwksCache.KeyFunc())
    if err != nil { http.Error(w, "invalid_jwt", 401); return }

    tenantID, ok := claims["tenant_id"].(string)
    if !ok || tenantID == "" {
        http.Error(w, `{"error":"missing_tenant_claim"}`, 401)
        return
    }
    subject := claims["sub"].(string)
    email   := claims["email"].(string)

    // JTI revocation check
    if jti, ok := claims["jti"].(string); ok {
        if p.isRevoked(jti) { http.Error(w, "revoked", 401); return }
    }

    // JIT provision
    mmUser, jit, err := p.jitProvision(subject, email, tenantID)
    if err != nil { http.Error(w, err.Error(), 500); return }

    // Tenant team mapping check
    teamID, err := p.tenantTeamID(tenantID)
    if err != nil { http.Error(w, "tenant_not_mapped", 500); return }
    if !p.isUserOnTeam(mmUser.Id, teamID) {
        http.Error(w, `{"error":"tenant_mismatch"}`, 403)
        return
    }

    // Create session
    session, err := p.API.CreateSession(&model.Session{
        UserId: mmUser.Id,
        Props:  model.StringMap{"tenant_id": tenantID, "cyberos_subject_id": subject},
    })
    if err != nil { http.Error(w, err.Error(), 500); return }

    p.emitMemoryAudit("chat.session_started", map[string]interface{}{
        "mm_user_id": mmUser.Id, "cyberos_subject_id": subject,
        "tenant_id": tenantID, "jit_provisioned": jit,
    })

    json.NewEncoder(w).Encode(map[string]string{"session_id": session.Id})
}

// jit_provision.go
func (p *AuthBridgePlugin) jitProvision(subject, email, tenantID string) (*model.User, bool, error) {
    users, err := p.API.SearchUsers(&model.UserSearch{
        Term:     subject,
        TeamId:   "",
        InTeamId: "",
    })
    if err == nil {
        for _, u := range users {
            if u.Props["cyberos_subject_id"] == subject { return u, false, nil }
        }
    }
    // Provision
    newUser := &model.User{
        Email:    email,
        Username: emailLocalpart(email),
        Props:    model.StringMap{"cyberos_subject_id": subject, "cyberos_tenant_id": tenantID},
        AuthService: "cyberos-jwt",
        AuthData: &subject,
    }
    created, appErr := p.API.CreateUser(newUser)
    if appErr != nil { return nil, false, fmt.Errorf("create user: %w", appErr) }
    return created, true, nil
}
```

### Patches (snippets)

```diff
# services/chat/patches/010-disable-builtin-auth.patch
--- a/api4/user.go
+++ b/api4/user.go
@@ -123,7 +123,8 @@ func login(c *Context, w http.ResponseWriter, r *http.Request) {
-    // original Mattermost password-flow
+    http.Error(w, `{"error":"builtin_auth_disabled","reason":"use_cyberos_jwt"}`, http.StatusMethodNotAllowed)
+    return
@@ -340,7 +340,8 @@ func sendPasswordReset(c *Context, w http.ResponseWriter, r *http.Request) {
-    // original Mattermost reset-flow
+    http.Error(w, `{"error":"builtin_auth_disabled","reason":"use_cyberos_jwt"}`, http.StatusGone)
+    return
@@ -402,7 +402,8 @@ func verifyUserEmail(c *Context, w http.ResponseWriter, r *http.Request) {
-    // original Mattermost verify-flow
+    http.Error(w, `{"error":"builtin_auth_disabled","reason":"use_cyberos_jwt"}`, http.StatusGone)
+    return
```

```diff
# services/chat/patches/011-load-authbridge-plugin.patch
--- a/app/plugin.go
+++ b/app/plugin.go
@@ -50,6 +50,10 @@ func (a *App) initPlugins() {
+    // Auto-load cyberos-authbridge at boot.  Fatal on miss: the entire
+    // CHAT surface depends on it for tenant propagation; running without
+    // it would silently expose cross-tenant data.
+    if err := a.LoadPlugin("/opt/cyberos/plugins/cyberos-authbridge.tar.gz"); err != nil {
+        mlog.Fatal("authbridge plugin failed to load", mlog.Err(err))
+    }
```

### plugin.json — Mattermost plugin manifest

```json
{
  "id": "cyberos.authbridge",
  "name": "CyberOS AuthBridge",
  "description": "Delegates Mattermost authentication to FR-AUTH-004 JWT with tenant propagation",
  "version": "1.0.0",
  "min_server_version": "9.0.0",
  "server": {
    "executables": {
      "linux-amd64": "server/plugin-linux-amd64",
      "linux-arm64": "server/plugin-linux-arm64"
    }
  },
  "settings_schema": {
    "settings": [
      { "key": "jwks_url",         "type": "text", "default": "https://auth.cyberskill.world/.well-known/jwks.json" },
      { "key": "jwks_ttl_seconds", "type": "number", "default": 3600 },
      { "key": "jwks_refresh_seconds", "type": "number", "default": 3000 },
      { "key": "issuer",           "type": "text", "default": "https://auth.cyberskill.world" },
      { "key": "tenant_team_map_table", "type": "text", "default": "cyberos_chat_tenant_map" },
      { "key": "revocation_lookup_url", "type": "text", "default": "https://auth.cyberskill.world/v1/revocations" }
    ]
  }
}
```

### jwks_cache.go — JWKS with proactive refresh

```go
// services/chat/plugins/cyberos-authbridge/jwks_cache.go
package main

import (
    "context"
    "encoding/json"
    "errors"
    "fmt"
    "io"
    "net/http"
    "sync"
    "sync/atomic"
    "time"

    "github.com/golang-jwt/jwt/v5"
    "github.com/lestrrat-go/jwx/v2/jwk"
)

// JwksCache fetches and serves JWKS keys with TTL + proactive refresh.
// All accesses are O(1) once warm; refresh runs in a background goroutine.
type JwksCache struct {
    url           string
    httpClient    *http.Client
    ttl           time.Duration
    refreshEvery  time.Duration
    keyset        atomic.Pointer[jwk.Set]   // RCU pattern: replace pointer, never mutate
    lastFetchedAt atomic.Int64              // unix nanos
    stopCh        chan struct{}
    wg            sync.WaitGroup
    // metrics callbacks injected by plugin to avoid prom-go dependency in this file
    onCacheHit  func()
    onCacheMiss func()
    onRefreshOk func()
    onRefreshErr func(reason string)
}

func NewJwksCache(url string, ttl, refreshEvery time.Duration) *JwksCache {
    return &JwksCache{
        url:          url,
        httpClient:   &http.Client{Timeout: 5 * time.Second},
        ttl:          ttl,
        refreshEvery: refreshEvery,
        stopCh:       make(chan struct{}),
    }
}

// Start prime-fetches the keyset and launches the proactive refresher.
// MUST be called from OnActivate; not idempotent (call once).
func (c *JwksCache) Start(ctx context.Context) error {
    if err := c.refresh(ctx); err != nil {
        return fmt.Errorf("initial JWKS fetch: %w", err)
    }
    c.wg.Add(1)
    go c.refreshLoop()
    return nil
}

// Stop signals the refresher and waits for it to exit. Idempotent.
func (c *JwksCache) Stop() {
    select {
    case <-c.stopCh:
        return // already stopped
    default:
        close(c.stopCh)
    }
    c.wg.Wait()
}

func (c *JwksCache) refreshLoop() {
    defer c.wg.Done()
    t := time.NewTicker(c.refreshEvery)
    defer t.Stop()
    for {
        select {
        case <-c.stopCh:
            return
        case <-t.C:
            ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
            _ = c.refresh(ctx)
            cancel()
        }
    }
}

func (c *JwksCache) refresh(ctx context.Context) error {
    req, _ := http.NewRequestWithContext(ctx, "GET", c.url, nil)
    resp, err := c.httpClient.Do(req)
    if err != nil {
        c.onRefreshErr("network")
        return err
    }
    defer resp.Body.Close()
    if resp.StatusCode != 200 {
        c.onRefreshErr(fmt.Sprintf("http_%d", resp.StatusCode))
        return fmt.Errorf("jwks fetch status %d", resp.StatusCode)
    }
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        c.onRefreshErr("body_read")
        return err
    }
    set, err := jwk.Parse(body)
    if err != nil {
        c.onRefreshErr("parse")
        return err
    }
    c.keyset.Store(&set)
    c.lastFetchedAt.Store(time.Now().UnixNano())
    c.onRefreshOk()
    return nil
}

// KeyFunc returns a jwt.KeyFunc closed over the cached keyset.
// Triggers a synchronous refresh on cache miss (kid not present in current set).
func (c *JwksCache) KeyFunc() jwt.Keyfunc {
    return func(tok *jwt.Token) (interface{}, error) {
        kid, ok := tok.Header["kid"].(string)
        if !ok {
            return nil, errors.New("missing kid")
        }
        setPtr := c.keyset.Load()
        if setPtr == nil {
            return nil, errors.New("jwks not initialised")
        }
        if key, found := (*setPtr).LookupKeyID(kid); found {
            c.onCacheHit()
            var raw interface{}
            if err := key.Raw(&raw); err != nil {
                return nil, err
            }
            return raw, nil
        }
        // Cache miss: synchronous refresh (rare; only on kid rotation).
        c.onCacheMiss()
        ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
        defer cancel()
        if err := c.refresh(ctx); err != nil {
            return nil, fmt.Errorf("refresh on miss: %w", err)
        }
        setPtr = c.keyset.Load()
        if key, found := (*setPtr).LookupKeyID(kid); found {
            var raw interface{}
            if err := key.Raw(&raw); err != nil {
                return nil, err
            }
            return raw, nil
        }
        return nil, fmt.Errorf("kid %q not present after refresh", kid)
    }
}

// Age returns how long since the last successful refresh; used for OBS health.
func (c *JwksCache) Age() time.Duration {
    ns := c.lastFetchedAt.Load()
    if ns == 0 {
        return -1
    }
    return time.Duration(time.Now().UnixNano() - ns)
}
```

### jit_provision.go — first-login user creation

```go
// services/chat/plugins/cyberos-authbridge/jit_provision.go
package main

import (
    "fmt"
    "regexp"
    "strings"
    "sync"

    "github.com/mattermost/mattermost/server/public/model"
)

// jitMu serialises user creation per subject_id to prevent two concurrent
// first-logins from creating two Mattermost users for the same subject.
type jitGate struct {
    mu       sync.Mutex
    inflight map[string]chan struct{}
}

func newJitGate() *jitGate {
    return &jitGate{inflight: map[string]chan struct{}{}}
}

func (g *jitGate) acquire(subject string) (release func()) {
    g.mu.Lock()
    ch, exists := g.inflight[subject]
    if exists {
        g.mu.Unlock()
        <-ch
        g.mu.Lock()
    }
    ch = make(chan struct{})
    g.inflight[subject] = ch
    g.mu.Unlock()
    return func() {
        g.mu.Lock()
        close(ch)
        delete(g.inflight, subject)
        g.mu.Unlock()
    }
}

var usernameSanitiser = regexp.MustCompile(`[^a-z0-9._-]`)

// sanitiseUsername converts an email localpart to a Mattermost-legal username.
// MM constraint: 3..22 chars, [a-z0-9._-]. Truncate at 22; pad with "u" if < 3.
func sanitiseUsername(emailOrLocal string) string {
    s := strings.ToLower(emailOrLocal)
    if at := strings.Index(s, "@"); at > 0 {
        s = s[:at]
    }
    s = usernameSanitiser.ReplaceAllString(s, "_")
    if len(s) > 22 {
        s = s[:22]
    }
    for len(s) < 3 {
        s += "u"
    }
    return s
}

// jitProvision returns (user, jitProvisioned, err). If a Mattermost user
// already exists for the subject, returns (user, false, nil). Otherwise
// creates one and returns (user, true, nil). Idempotent under contention
// via the per-subject gate.
func (p *AuthBridgePlugin) jitProvision(subject, email, tenantID string) (*model.User, bool, error) {
    release := p.jitGate.acquire(subject)
    defer release()

    if u, err := p.findUserBySubject(subject); err == nil && u != nil {
        return u, false, nil
    }

    username := sanitiseUsername(email)
    // Handle username collision by appending a numeric suffix.
    for n := 0; n < 100; n++ {
        candidate := username
        if n > 0 {
            candidate = fmt.Sprintf("%s_%d", username, n)
        }
        if _, err := p.API.GetUserByUsername(candidate); err != nil {
            // Not found — claim this candidate.
            authData := subject
            newUser := &model.User{
                Email:       email,
                Username:    candidate,
                Props:       model.StringMap{"cyberos_subject_id": subject, "cyberos_tenant_id": tenantID},
                AuthService: "cyberos-jwt",
                AuthData:    &authData,
            }
            created, appErr := p.API.CreateUser(newUser)
            if appErr != nil {
                return nil, false, fmt.Errorf("create user: %w", appErr)
            }
            return created, true, nil
        }
    }
    return nil, false, fmt.Errorf("username %q exhausted 100 collision suffixes", username)
}

func (p *AuthBridgePlugin) findUserBySubject(subject string) (*model.User, error) {
    // The plugin keeps a Postgres-side index on (props->>'cyberos_subject_id').
    // Falls back to SearchUsers if the index is unavailable.
    return p.lookupSubjectIndex(subject)
}
```

### tenant_map.go — tenant↔team lookup with cache

```go
// services/chat/plugins/cyberos-authbridge/tenant_map.go
package main

import (
    "errors"
    "sync"
    "time"
)

type tenantMapEntry struct {
    teamID  string
    fetched time.Time
}

// tenantMap caches tenant_id → mm_team_id with 5min TTL.
// Backed by cyberos_chat_tenant_map (FR-AUTH-005 admin REST owns writes).
type tenantMap struct {
    mu      sync.RWMutex
    cache   map[string]tenantMapEntry
    ttl     time.Duration
    dbQuery func(tenantID string) (string, error)
}

func newTenantMap(dbQuery func(string) (string, error)) *tenantMap {
    return &tenantMap{cache: map[string]tenantMapEntry{}, ttl: 5 * time.Minute, dbQuery: dbQuery}
}

func (m *tenantMap) Lookup(tenantID string) (string, error) {
    m.mu.RLock()
    e, ok := m.cache[tenantID]
    m.mu.RUnlock()
    if ok && time.Since(e.fetched) < m.ttl {
        return e.teamID, nil
    }
    teamID, err := m.dbQuery(tenantID)
    if err != nil {
        return "", err
    }
    if teamID == "" {
        return "", errors.New("tenant_not_mapped")
    }
    m.mu.Lock()
    m.cache[tenantID] = tenantMapEntry{teamID: teamID, fetched: time.Now()}
    m.mu.Unlock()
    return teamID, nil
}

// Invalidate is called from FR-AUTH-005 webhook on mapping change.
func (m *tenantMap) Invalidate(tenantID string) {
    m.mu.Lock()
    delete(m.cache, tenantID)
    m.mu.Unlock()
}
```

### metrics.go — OTel registration

```go
// services/chat/plugins/cyberos-authbridge/metrics.go
package main

import (
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/attribute"
    "go.opentelemetry.io/otel/metric"
)

type authMetrics struct {
    loginsTotal       metric.Int64Counter
    jitProvisionTotal metric.Int64Counter
    jwksCacheHits     metric.Int64Counter
    jwksRefresh       metric.Int64Counter
    activeSessions    metric.Int64UpDownCounter
}

func newAuthMetrics() (*authMetrics, error) {
    m := otel.Meter("cyberos.chat.authbridge")
    var err error
    out := &authMetrics{}
    if out.loginsTotal, err = m.Int64Counter("chat_authbridge_logins_total",
        metric.WithDescription("Login outcomes by category")); err != nil { return nil, err }
    if out.jitProvisionTotal, err = m.Int64Counter("chat_authbridge_jit_provisions_total",
        metric.WithDescription("JIT-provisioned users")); err != nil { return nil, err }
    if out.jwksCacheHits, err = m.Int64Counter("chat_authbridge_jwks_cache_hits_total",
        metric.WithDescription("JWKS cache lookups by result")); err != nil { return nil, err }
    if out.jwksRefresh, err = m.Int64Counter("chat_authbridge_jwks_refresh_total",
        metric.WithDescription("JWKS background refresh outcomes")); err != nil { return nil, err }
    if out.activeSessions, err = m.Int64UpDownCounter("chat_authbridge_active_sessions",
        metric.WithDescription("Currently active JWT-backed sessions")); err != nil { return nil, err }
    return out, nil
}

// Stable label values — never use fmt.Sprintf or format!("{:?}", ...) for labels.
const (
    outcomeOK              = "ok"
    outcomeInvalidJwt      = "invalid_jwt"
    outcomeTenantMismatch  = "tenant_mismatch"
    outcomeRevoked         = "revoked"
    outcomeMissingTenant   = "missing_tenant_claim"
    outcomeJwksUnavailable = "jwks_unavailable"
    outcomeServerError     = "server_error"
)

func (m *authMetrics) login(outcome string) {
    m.loginsTotal.Add(nil, 1, metric.WithAttributes(attribute.String("outcome", outcome)))
}
```

### Plugin lifecycle integration

```go
// services/chat/plugins/cyberos-authbridge/lifecycle.go
package main

import (
    "context"
    "fmt"
    "time"
)

func (p *AuthBridgePlugin) OnActivate() error {
    cfg := p.API.GetConfig().PluginSettings.Plugins["cyberos.authbridge"]
    jwksURL, _ := cfg["jwks_url"].(string)
    ttlSec, _ := cfg["jwks_ttl_seconds"].(float64)
    refreshSec, _ := cfg["jwks_refresh_seconds"].(float64)

    metrics, err := newAuthMetrics()
    if err != nil { return fmt.Errorf("metrics init: %w", err) }
    p.metrics = metrics

    cache := NewJwksCache(jwksURL, time.Duration(ttlSec)*time.Second, time.Duration(refreshSec)*time.Second)
    cache.onCacheHit  = func() { metrics.jwksCacheHits.Add(nil, 1) }
    cache.onCacheMiss = func() { metrics.jwksCacheHits.Add(nil, 1) }
    cache.onRefreshOk = func() { metrics.jwksRefresh.Add(nil, 1) }
    cache.onRefreshErr = func(reason string) {}

    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    if err := cache.Start(ctx); err != nil {
        return fmt.Errorf("jwks prime: %w", err)
    }
    p.jwksCache = cache

    p.jitGate    = newJitGate()
    p.tenantMap  = newTenantMap(p.queryTenantTeam)
    p.revoker    = newRevocationChecker(cfg["revocation_lookup_url"].(string))

    return nil
}

func (p *AuthBridgePlugin) OnDeactivate() error {
    if p.jwksCache != nil { p.jwksCache.Stop() }
    if p.revoker  != nil  { p.revoker.Stop() }
    return nil
}
```

### Error-envelope contract (stable across all 4xx/5xx)

All non-200 responses MUST emit an `application/json` body with this shape — downstream clients depend on the `error` field for routing:

```json
{
  "error": "<machine_readable_id>",
  "reason": "<short_human_string>",
  "trace_id": "<32-char W3C trace id>",
  "retry_after_seconds": null
}
```

Valid `error` values: `invalid_jwt | missing_tenant_claim | tenant_mismatch | revoked | builtin_auth_disabled | jwks_unavailable | tenant_not_mapped | username_collision_exhausted | server_error`. Any other value indicates a regression — covered by AC #18 (negative test: every error-envelope returned in the test suite must come from this enum).

---

## §4 — Acceptance criteria

1. **Valid JWT logs in** — POST /plugins/cyberos.authbridge/login with valid JWT → 200 with session_id.
2. **Invalid signature → 401** — tampered JWT → 401 `invalid_jwt`.
3. **Missing tenant_id claim → 401** — JWT without claim → 401 `missing_tenant_claim`.
4. **Expired JWT → 401** — `exp` in past → 401.
5. **JIT provisions on first login** — new subject_id → user created in Mattermost; jit_provisioned=true in audit.
6. **Reuse user on subsequent login** — same subject_id → existing user; jit_provisioned=false.
7. **Tenant mismatch → 403** — user on team A, JWT carries tenant B → 403 `tenant_mismatch`.
8. **Revoked jti → 401** — jti in deny list → 401 `revoked`.
9. **Built-in password endpoint disabled** — POST `/api/v4/users/login` with password body → 405.
10. **JWKS cache hit** — second login within 1h → JWKS not re-fetched; metric increments.
11. **JWKS refresh proactive** — fixture: 50min elapsed → background refresh fires.
12. **Tenant_id propagated to session.Props** — downstream plugin reads `c.AppContext.Session().Props["tenant_id"]` → returns expected value.
13. **memory audit `chat.session_started`** — row emitted per login.
14. **OTel logins_total{outcome="ok"}** — counter increments.
15. **OTel jit_provisions_total** — increments only on first-login provisioning.
16. **Plugin OnActivate completes < 1s** — startup latency.
17. **Plugin survives 10K concurrent logins** — load test; no goroutine leak.
18. **Error envelope is closed-enum** — every 4xx/5xx body's `error` field MUST be one of the 9 values in §3 "Error-envelope contract"; CI lint scans the test corpus and rejects any unlisted value.
19. **JIT collision under contention is idempotent** — 50 concurrent first-logins for the same subject_id MUST produce exactly one Mattermost user; the other 49 reuse it.
20. **Username sanitisation maps to Mattermost legal range** — every email localpart used as a username is reduced to `[a-z0-9._-]{3,22}`; verified by property test over 1,000 random inputs.
21. **JWKS unavailable → 503 + fail-secure** — JWKS endpoint unreachable AND cache cold → login returns 503 `jwks_unavailable`, NOT 200; metric `chat_authbridge_logins_total{outcome="jwks_unavailable"}` increments.
22. **JWKS rotation tolerated** — kid in JWT not present in current cached set → synchronous refresh fires → if new kid then present, login succeeds; if still missing, 401 with `error:"invalid_jwt"`.
23. **OnDeactivate stops refresh goroutine** — after OnDeactivate, no further JWKS HTTP requests fire; runtime goroutine count returns to pre-activate baseline within 500ms.
24. **Audit row carries trace_id and W3C-format** — `trace_id` in `chat.session_started` MUST be 32 lowercase hex chars (verified by regex); no `TraceId(...)` Debug form.
25. **JWT replay outside `exp` window → 401** — same JWT presented after `exp` → 401 `invalid_jwt`; replay BEFORE `exp` is allowed (stateless JWT).
26. **Per-request tracing context honoured** — inbound `traceparent` header is parsed and propagated to the audit row's `trace_id`; missing → new trace_id generated at trust boundary.
27. **Revocation lookup failure-mode** — revocation service unreachable → fail-SECURE: deny login with 401 `revoked` AND emit metric `outcome="revoked"` AND log a SEV-2 warning. Operators see "revocation lookup outage" within 60s.
28. **Tenant-team mapping cache invalidation** — FR-AUTH-005 webhook → `tenantMap.Invalidate(tenant_id)` → subsequent login does a fresh DB lookup (verified by stub-DB-counter assertion).
29. **Concurrent OnActivate calls are no-op past the first** — if Mattermost calls OnActivate twice (rare; SDK bug), second call is rejected with explicit error; no duplicate JWKS goroutines.
30. **Plugin shipping artifact is reproducible** — `make` produces a `.tar.gz` whose SHA-256 matches across two consecutive builds with the same source.

---

## §5 — Verification

Tests live in `services/chat/plugins/cyberos-authbridge/tests/`. Setup helpers below are referenced by every test.

```go
// tests/setup_test.go
package authbridge_test

import (
    "crypto/ecdsa"
    "crypto/elliptic"
    "crypto/rand"
    "encoding/json"
    "net/http/httptest"
    "testing"
    "time"

    "github.com/golang-jwt/jwt/v5"
    "github.com/mattermost/mattermost/server/public/model"
    "github.com/mattermost/mattermost/server/public/plugin/plugintest"
    "github.com/stretchr/testify/mock"
)

type kit struct {
    plugin   *AuthBridgePlugin
    api      *plugintest.API
    signer   *ecdsa.PrivateKey
    jwksKid  string
    audit    *fakeAuditSink
    revocs   map[string]struct{}
    teamMap  map[string]string
}

func setupPlugin(t *testing.T) *kit {
    t.Helper()
    api := &plugintest.API{}
    signer, _ := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
    kid := "kid-test-001"

    jwksSrv := httptest.NewServer(jwksHandler(signer, kid))
    t.Cleanup(jwksSrv.Close)

    api.On("GetConfig").Return(&model.Config{
        PluginSettings: model.PluginSettings{
            Plugins: map[string]map[string]interface{}{
                "cyberos.authbridge": {
                    "jwks_url":             jwksSrv.URL + "/jwks.json",
                    "jwks_ttl_seconds":     float64(3600),
                    "jwks_refresh_seconds": float64(3000),
                    "issuer":               "https://auth.test",
                    "revocation_lookup_url": "http://localhost/none",
                },
            },
        },
    })

    audit := &fakeAuditSink{}
    p := &AuthBridgePlugin{audit: audit}
    p.SetAPI(api)
    if err := p.OnActivate(); err != nil { t.Fatalf("OnActivate: %v", err) }
    t.Cleanup(func() { _ = p.OnDeactivate() })

    return &kit{plugin: p, api: api, signer: signer, jwksKid: kid, audit: audit,
        revocs: map[string]struct{}{}, teamMap: map[string]string{}}
}

func (k *kit) issueJwt(sub, tenant, email string, mods ...func(jwt.MapClaims)) string {
    claims := jwt.MapClaims{
        "iss":       "https://auth.test",
        "sub":       sub,
        "email":     email,
        "tenant_id": tenant,
        "exp":       time.Now().Add(15 * time.Minute).Unix(),
        "nbf":       time.Now().Add(-1 * time.Second).Unix(),
        "iat":       time.Now().Unix(),
        "jti":       "jti-" + sub + "-" + tenant + "-" + email,
    }
    for _, m := range mods { m(claims) }
    tok := jwt.NewWithClaims(jwt.SigningMethodES256, claims)
    tok.Header["kid"] = k.jwksKid
    s, err := tok.SignedString(k.signer)
    if err != nil { panic(err) }
    return s
}
```

### AC #1 — valid JWT logs in

```go
func TestValidJwtLogin(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("subject-1", "tenant-1", "alice@cyberskill.world")
    req := httptest.NewRequest("POST", "/login", nil)
    req.Header.Set("Authorization", "Bearer "+j)
    rec := httptest.NewRecorder()
    k.plugin.ServeHTTP(nil, rec, req)
    if rec.Code != 200 { t.Fatalf("want 200, got %d body=%s", rec.Code, rec.Body.String()) }
    var body map[string]string
    json.NewDecoder(rec.Body).Decode(&body)
    if body["session_id"] == "" { t.Fatal("session_id missing") }
}
```

### AC #2 — invalid signature

```go
func TestInvalidSignature(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("subject-1", "tenant-1", "alice@x.com")
    // Tamper the last 4 chars (signature).
    tampered := j[:len(j)-4] + "AAAA"
    req := httptest.NewRequest("POST", "/login", nil)
    req.Header.Set("Authorization", "Bearer "+tampered)
    rec := httptest.NewRecorder()
    k.plugin.ServeHTTP(nil, rec, req)
    if rec.Code != 401 { t.Fatalf("want 401, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "invalid_jwt")
}
```

### AC #3 — missing tenant_id claim

```go
func TestMissingTenantClaim(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s", "t", "a@x.com", func(c jwt.MapClaims) { delete(c, "tenant_id") })
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 401 { t.Fatalf("want 401, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "missing_tenant_claim")
}
```

### AC #4 — expired JWT

```go
func TestExpiredJwt(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s", "t", "a@x.com", func(c jwt.MapClaims) {
        c["exp"] = time.Now().Add(-1 * time.Minute).Unix()
    })
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 401 { t.Fatalf("want 401, got %d", rec.Code) }
}
```

### AC #5 + #6 — JIT then reuse

```go
func TestJitThenReuse(t *testing.T) {
    k := setupPlugin(t)
    k.api.On("GetUserByUsername", mock.Anything).Return(nil, &model.AppError{StatusCode: 404})
    k.api.On("CreateUser", mock.MatchedBy(func(u *model.User) bool {
        return u.Props["cyberos_subject_id"] == "new-subject"
    })).Return(&model.User{Id: "u-new", Username: "newuser",
        Props: model.StringMap{"cyberos_subject_id": "new-subject", "cyberos_tenant_id": "tenant-1"},
    }, nil).Once()

    j := k.issueJwt("new-subject", "tenant-1", "newuser@x.com")
    rec1 := postWithBearer(t, k.plugin, j)
    if rec1.Code != 200 { t.Fatalf("first login want 200, got %d", rec1.Code) }
    requireAuditField(t, k.audit, "chat.session_started", "jit_provisioned", true)

    // Second login: must reuse via subject-index lookup.
    rec2 := postWithBearer(t, k.plugin, j)
    if rec2.Code != 200 { t.Fatalf("second login want 200, got %d", rec2.Code) }
    requireAuditField(t, k.audit, "chat.session_started", "jit_provisioned", false)
    // CreateUser MUST have been called exactly once.
    k.api.AssertNumberOfCalls(t, "CreateUser", 1)
}
```

### AC #7 — tenant mismatch

```go
func TestTenantMismatch(t *testing.T) {
    k := setupPlugin(t)
    k.teamMap["tenant-B"] = "team-B"
    k.api.On("GetUsersInTeam", "team-B", mock.Anything, mock.Anything).Return([]*model.User{}, nil)
    k.api.On("CreateUser", mock.Anything).Return(&model.User{Id: "u1",
        Props: model.StringMap{"cyberos_tenant_id": "tenant-A"}}, nil)

    j := k.issueJwt("s1", "tenant-B", "alice@x.com")
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 403 { t.Fatalf("want 403, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "tenant_mismatch")
}
```

### AC #8 — revoked jti

```go
func TestRevokedJti(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s1", "tenant-1", "a@x.com", func(c jwt.MapClaims) { c["jti"] = "blocked-jti" })
    k.revocs["blocked-jti"] = struct{}{}
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 401 { t.Fatalf("want 401, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "revoked")
}
```

### AC #9 — built-in password endpoint disabled (integration test against patched Mattermost)

```go
//go:build integration
func TestBuiltinPasswordDisabled(t *testing.T) {
    srv := startPatchedMattermost(t)
    resp, err := http.Post(srv.URL+"/api/v4/users/login", "application/json",
        strings.NewReader(`{"login_id":"a@x.com","password":"p"}`))
    if err != nil { t.Fatal(err) }
    if resp.StatusCode != 405 { t.Fatalf("want 405, got %d", resp.StatusCode) }
}
```

### AC #10 + #11 — JWKS cache + proactive refresh

```go
func TestJwksCacheHit(t *testing.T) {
    k := setupPlugin(t)
    fetches := atomic.NewInt64(0)
    // Wrap jwks server to count fetches.
    k.jwksSrv.SetHook(func() { fetches.Add(1) })

    j := k.issueJwt("s", "tenant-1", "a@x.com")
    _ = postWithBearer(t, k.plugin, j)
    _ = postWithBearer(t, k.plugin, j)
    // 1 prime fetch on activate + 1 verify per login but verify uses cached set.
    if fetches.Load() != 1 { t.Fatalf("want 1 jwks fetch (prime only), got %d", fetches.Load()) }
}

func TestJwksProactiveRefresh(t *testing.T) {
    // Override refresh interval to 50ms for the test.
    k := setupPluginWithRefresh(t, 50*time.Millisecond)
    initial := k.plugin.jwksCache.lastFetchedAt.Load()
    time.Sleep(120 * time.Millisecond)
    if k.plugin.jwksCache.lastFetchedAt.Load() == initial {
        t.Fatal("background refresh did not fire")
    }
}
```

### AC #12 — tenant_id propagated to session

```go
func TestTenantIdInSessionProps(t *testing.T) {
    k := setupPlugin(t)
    var captured *model.Session
    k.api.On("CreateSession", mock.MatchedBy(func(s *model.Session) bool {
        captured = s; return true
    })).Return(&model.Session{Id: "sess-1"}, nil)
    j := k.issueJwt("s1", "tenant-Z", "a@x.com")
    _ = postWithBearer(t, k.plugin, j)
    if captured.Props["tenant_id"] != "tenant-Z" {
        t.Fatalf("session.Props missing tenant_id; got %v", captured.Props)
    }
}
```

### AC #13 + #14 + #15 — audit row + metrics

```go
func TestAuditAndMetricsOnLogin(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s1", "tenant-1", "a@x.com")
    _ = postWithBearer(t, k.plugin, j)
    requireAuditKind(t, k.audit, "chat.session_started")
    if v := metricValue("chat_authbridge_logins_total", map[string]string{"outcome": "ok"}); v != 1 {
        t.Fatalf("logins_total{outcome=ok} = %d, want 1", v)
    }
    if v := metricValue("chat_authbridge_jit_provisions_total", nil); v != 1 {
        t.Fatalf("jit_provisions_total = %d, want 1", v)
    }
}
```

### AC #16 — OnActivate completes < 1s

```go
func TestOnActivateLatency(t *testing.T) {
    start := time.Now()
    _ = setupPlugin(t) // calls OnActivate
    if d := time.Since(start); d > 1*time.Second {
        t.Fatalf("OnActivate took %v, want < 1s", d)
    }
}
```

### AC #17 — 10K concurrent logins; no goroutine leak

```go
func TestConcurrentLoginsNoLeak(t *testing.T) {
    k := setupPlugin(t)
    before := runtime.NumGoroutine()
    var wg sync.WaitGroup
    for i := 0; i < 10_000; i++ {
        wg.Add(1)
        go func(n int) {
            defer wg.Done()
            j := k.issueJwt(fmt.Sprintf("s-%d", n), "tenant-1", fmt.Sprintf("u%d@x.com", n))
            _ = postWithBearer(t, k.plugin, j)
        }(i)
    }
    wg.Wait()
    time.Sleep(200 * time.Millisecond) // settle
    if leaked := runtime.NumGoroutine() - before; leaked > 10 {
        t.Fatalf("goroutine leak: +%d after 10K logins", leaked)
    }
}
```

### AC #18 — error envelope is closed-enum

```go
func TestErrorEnvelopeIsClosedEnum(t *testing.T) {
    allowed := map[string]struct{}{
        "invalid_jwt": {}, "missing_tenant_claim": {}, "tenant_mismatch": {},
        "revoked": {}, "builtin_auth_disabled": {}, "jwks_unavailable": {},
        "tenant_not_mapped": {}, "username_collision_exhausted": {}, "server_error": {},
    }
    // Crawl test artefacts and the source for emitted error strings.
    emitted := scanSourceForErrorEnvelope(t, "./..")
    for _, e := range emitted {
        if _, ok := allowed[e]; !ok {
            t.Errorf("undeclared error envelope value %q", e)
        }
    }
}
```

### AC #19 — JIT concurrency idempotency

```go
func TestJitConcurrentSameSubjectOneUser(t *testing.T) {
    k := setupPlugin(t)
    creates := atomic.NewInt64(0)
    k.api.On("CreateUser", mock.Anything).Run(func(_ mock.Arguments) {
        creates.Add(1)
    }).Return(&model.User{Id: "u1", Props: model.StringMap{"cyberos_subject_id": "S"}}, nil).Once()
    k.api.On("CreateUser", mock.Anything).Return(nil, &model.AppError{Id: "user.exists"})

    var wg sync.WaitGroup
    for i := 0; i < 50; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            j := k.issueJwt("S", "tenant-1", "u@x.com")
            _ = postWithBearer(t, k.plugin, j)
        }()
    }
    wg.Wait()
    if creates.Load() != 1 {
        t.Fatalf("CreateUser called %d times, want 1", creates.Load())
    }
}
```

### AC #20 — username sanitisation property test

```go
func TestSanitiseUsernameProperty(t *testing.T) {
    rapid.Check(t, func(t *rapid.T) {
        in := rapid.String().Draw(t, "in")
        out := sanitiseUsername(in)
        if len(out) < 3 || len(out) > 22 {
            t.Fatalf("length %d out of range", len(out))
        }
        for _, r := range out {
            if !(r == '.' || r == '_' || r == '-' ||
                (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9')) {
                t.Fatalf("illegal char %q in %q (in=%q)", r, out, in)
            }
        }
    })
}
```

### AC #21 — JWKS unavailable + cold cache → 503

```go
func TestJwksUnavailableFailSecure(t *testing.T) {
    k := setupPluginWithJwksDown(t)
    j := k.issueJwt("s", "tenant-1", "a@x.com")
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 503 { t.Fatalf("want 503, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "jwks_unavailable")
    if v := metricValue("chat_authbridge_logins_total", map[string]string{"outcome": "jwks_unavailable"}); v != 1 {
        t.Fatalf("metric mismatch: %d", v)
    }
}
```

### AC #22 — JWKS rotation

```go
func TestJwksRotation(t *testing.T) {
    k := setupPlugin(t)
    // Rotate signer to a new kid.
    newSigner, _ := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
    k.jwksSrv.Rotate("kid-test-002", newSigner)
    k.signer = newSigner
    k.jwksKid = "kid-test-002"

    j := k.issueJwt("s", "tenant-1", "a@x.com")
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 200 {
        t.Fatalf("after rotation want 200, got %d body=%s", rec.Code, rec.Body.String())
    }
}
```

### AC #23 — OnDeactivate stops refresh goroutine

```go
func TestOnDeactivateStopsRefresher(t *testing.T) {
    k := setupPluginWithRefresh(t, 50*time.Millisecond)
    before := runtime.NumGoroutine()
    if err := k.plugin.OnDeactivate(); err != nil { t.Fatal(err) }
    time.Sleep(200 * time.Millisecond)
    if delta := runtime.NumGoroutine() - before; delta > 0 {
        t.Fatalf("goroutines did not exit: delta=%d", delta)
    }
}
```

### AC #24 — trace_id format

```go
func TestAuditTraceIdFormat(t *testing.T) {
    k := setupPlugin(t)
    _ = postWithBearer(t, k.plugin, k.issueJwt("s", "tenant-1", "a@x.com"))
    row := k.audit.last()
    tid, _ := row["trace_id"].(string)
    if !regexp.MustCompile(`^[0-9a-f]{32}$`).MatchString(tid) {
        t.Fatalf("trace_id %q not 32-hex", tid)
    }
}
```

### AC #25 — replay after exp

```go
func TestReplayAfterExp(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s", "tenant-1", "a@x.com", func(c jwt.MapClaims) {
        c["exp"] = time.Now().Add(500 * time.Millisecond).Unix()
    })
    rec1 := postWithBearer(t, k.plugin, j)
    if rec1.Code != 200 { t.Fatalf("first login want 200, got %d", rec1.Code) }
    time.Sleep(1 * time.Second)
    rec2 := postWithBearer(t, k.plugin, j)
    if rec2.Code != 401 { t.Fatalf("post-exp replay want 401, got %d", rec2.Code) }
}
```

### AC #26 — traceparent honoured

```go
func TestInboundTraceparentPropagated(t *testing.T) {
    k := setupPlugin(t)
    j := k.issueJwt("s", "tenant-1", "a@x.com")
    req := httptest.NewRequest("POST", "/login", nil)
    req.Header.Set("Authorization", "Bearer "+j)
    req.Header.Set("traceparent", "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
    rec := httptest.NewRecorder()
    k.plugin.ServeHTTP(nil, rec, req)
    row := k.audit.last()
    if row["trace_id"] != "4bf92f3577b34da6a3ce929d0e0e4736" {
        t.Fatalf("trace_id not propagated: %v", row["trace_id"])
    }
}
```

### AC #27 — revocation lookup fail-secure

```go
func TestRevocationServiceUnreachableFailsSecure(t *testing.T) {
    k := setupPluginWithRevocationDown(t)
    j := k.issueJwt("s", "tenant-1", "a@x.com")
    rec := postWithBearer(t, k.plugin, j)
    if rec.Code != 401 { t.Fatalf("want 401, got %d", rec.Code) }
    assertErrorEnvelope(t, rec.Body.Bytes(), "revoked")
    requireWarnContains(t, "revocation lookup outage")
}
```

### AC #28 — tenant-team mapping cache invalidation

```go
func TestTenantMapInvalidation(t *testing.T) {
    k := setupPlugin(t)
    dbHits := atomic.NewInt64(0)
    k.plugin.tenantMap.dbQuery = func(t string) (string, error) {
        dbHits.Add(1); return "team-X", nil
    }
    _ = postWithBearer(t, k.plugin, k.issueJwt("s1", "tenant-1", "a@x.com"))
    _ = postWithBearer(t, k.plugin, k.issueJwt("s2", "tenant-1", "b@x.com"))
    if dbHits.Load() != 1 { t.Fatalf("expected 1 db hit; got %d", dbHits.Load()) }

    k.plugin.tenantMap.Invalidate("tenant-1")
    _ = postWithBearer(t, k.plugin, k.issueJwt("s3", "tenant-1", "c@x.com"))
    if dbHits.Load() != 2 { t.Fatalf("expected 2 db hits after invalidate; got %d", dbHits.Load()) }
}
```

### AC #29 — double OnActivate is no-op

```go
func TestDoubleOnActivateRejected(t *testing.T) {
    k := setupPlugin(t)
    err := k.plugin.OnActivate()
    if err == nil { t.Fatal("second OnActivate must return error") }
    if !strings.Contains(err.Error(), "already activated") {
        t.Fatalf("unexpected error: %v", err)
    }
}
```

### AC #30 — reproducible build

```bash
# scripts/check-reproducible-build.sh
#!/usr/bin/env bash
set -euo pipefail
cd services/chat/plugins/cyberos-authbridge
make clean && make build && cp dist/plugin.tar.gz /tmp/a.tar.gz
make clean && make build && cp dist/plugin.tar.gz /tmp/b.tar.gz
diff <(shasum -a 256 /tmp/a.tar.gz | awk '{print $1}') \
     <(shasum -a 256 /tmp/b.tar.gz | awk '{print $1}')
echo "OK: build is reproducible"
```

---

## §6 — Implementation skeleton

The plugin code in §3 above is the implementation skeleton. This section adds the wiring decisions that don't live in any single file:

### §6.1 — Mattermost plugin lifecycle wiring

Mattermost's plugin SDK calls `OnActivate()` exactly once per pod lifecycle (post-bundle-load, pre-traffic). The plugin MUST be re-entrant-safe at the OnActivate boundary — if Mattermost re-invokes (rare; SDK regression), the second call MUST return an error rather than spawn a second JWKS refresher. Implementation pattern:

```go
type AuthBridgePlugin struct {
    plugin.MattermostPlugin
    activated  atomic.Bool

    jwksCache *JwksCache
    jitGate   *jitGate
    tenantMap *tenantMap
    revoker   *revocationChecker
    metrics   *authMetrics
    audit     memoryAuditSink
}

func (p *AuthBridgePlugin) OnActivate() error {
    if !p.activated.CompareAndSwap(false, true) {
        return errors.New("already activated")
    }
    // ... rest as shown in §3 lifecycle.go
}
```

### §6.2 — Audit emit ordering

Per feature-request-audit skill §3.8 rule 25 (audit-before-action), the memory row MUST be emitted BEFORE the CreateSession call returns to the client. The pattern is:

1. Validate JWT, JTI, tenant.
2. JIT-provision (idempotent; safe to retry).
3. Compute `chat.session_started` payload.
4. Emit memory row; await ack.
5. CreateSession in Mattermost.
6. Return 200.

If step 4 fails, step 5 MUST NOT execute — return 503 with `outcome=server_error` and emit metric `chat_authbridge_audit_emit_failures_total`. This preserves the audit-before-action invariant.

### §6.3 — `cyberos_chat_tenant_map` schema (referenced by FR-AUTH-005)

```sql
-- Owned by FR-AUTH-005 admin REST; read by FR-CHAT-002.
CREATE TABLE IF NOT EXISTS cyberos_chat_tenant_map (
    tenant_id    UUID PRIMARY KEY,
    mm_team_id   TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at  TIMESTAMPTZ
);

CREATE UNIQUE INDEX uniq_active_tenant_team
    ON cyberos_chat_tenant_map (tenant_id)
    WHERE archived_at IS NULL;

CREATE INDEX idx_tenant_team_archived
    ON cyberos_chat_tenant_map (archived_at)
    WHERE archived_at IS NOT NULL;

REVOKE UPDATE, DELETE ON cyberos_chat_tenant_map FROM cyberos_app;
GRANT  SELECT, INSERT             ON cyberos_chat_tenant_map TO cyberos_app;
GRANT         UPDATE, DELETE      ON cyberos_chat_tenant_map TO cyberos_chat_admin;
```

The unique partial index satisfies feature-request-audit skill §3.4 rule 14 (at most one active row per tenant). The `archived_at` column is the audit-trail for re-mapping operations.

### §6.4 — Mattermost user `Props` schema

The plugin writes these `Props` keys on JIT provisioning:

| Key | Type | Source | Mutated after creation? |
|---|---|---|---|
| `cyberos_subject_id` | UUID string | JWT `sub` | Never. Append-only marker. |
| `cyberos_tenant_id`  | UUID string | JWT `tenant_id` | Only via tenant-switch flow (FR-AUTH-005). |
| `cyberos_jit`        | bool string | "true" on creation | Never. |
| `cyberos_first_login_at` | ISO-8601 | now() | Never. |

Downstream plugins (FR-CHAT-004 search, FR-CHAT-008 mention) read `Props["cyberos_tenant_id"]` for tenant scoping. The plugin MUST NOT overwrite these props on subsequent logins — only on first JIT provision.

### §6.5 — Session.Props wire format

```go
session := &model.Session{
    UserId: mmUser.Id,
    Props: model.StringMap{
        "tenant_id":          tenantID,
        "cyberos_subject_id": subject,
        "cyberos_trace_id":   traceID,
        "cyberos_jti":        jti,
    },
}
```

`cyberos_jti` is stored so downstream logout (Mattermost `RevokeSession`) can publish the JTI to the AUTH-004 revocation list — see FR-AUTH-005.

### §6.6 — JWKS HTTP client tuning

The JWKS HTTP client MUST be tuned for low-latency low-volume calls:

| Setting | Value | Rationale |
|---|---|---|
| `Timeout` | 5s | JWKS endpoint SHOULD respond in <100ms; 5s = generous SLA |
| `MaxIdleConnsPerHost` | 2 | One for refresh, one slack for cache-miss synchronous fetches |
| `IdleConnTimeout` | 90s | Beat AWS NLB idle close (typically 350s) |
| `TLSHandshakeTimeout` | 2s | If TLS handshake doesn't complete in 2s, JWKS endpoint is overloaded |
| `DisableKeepAlives` | false | Reuse connection across refresh + miss-fetch |

### §6.7 — Build + ship pipeline

```makefile
# services/chat/plugins/cyberos-authbridge/Makefile
.PHONY: build clean test reproducible-build

PLUGIN_ID := cyberos.authbridge
DIST      := dist
GO_FILES  := $(shell find . -name '*.go' -not -path './dist/*')

build: $(DIST)/plugin-linux-amd64 $(DIST)/plugin-linux-arm64 $(DIST)/plugin.tar.gz

$(DIST)/plugin-linux-amd64: $(GO_FILES)
	mkdir -p $(DIST)
	CGO_ENABLED=0 GOOS=linux GOARCH=amd64 \
	    go build -trimpath -ldflags '-s -w -buildid=' -o $@ .

$(DIST)/plugin-linux-arm64: $(GO_FILES)
	mkdir -p $(DIST)
	CGO_ENABLED=0 GOOS=linux GOARCH=arm64 \
	    go build -trimpath -ldflags '-s -w -buildid=' -o $@ .

$(DIST)/plugin.tar.gz: $(DIST)/plugin-linux-amd64 $(DIST)/plugin-linux-arm64 plugin.json
	# Deterministic tar: sorted names, fixed mtime, no user/group.
	tar --sort=name --owner=0 --group=0 --numeric-owner \
	    --mtime='2000-01-01 00:00:00Z' \
	    -czf $@ plugin.json server/
	@echo "SHA-256: $$(sha256sum $@ | awk '{print $$1}')"

test:
	go test -race ./...

clean:
	rm -rf $(DIST)

reproducible-build:
	$(MAKE) clean build
	cp $(DIST)/plugin.tar.gz /tmp/a.tar.gz
	$(MAKE) clean build
	cp $(DIST)/plugin.tar.gz /tmp/b.tar.gz
	diff <(sha256sum /tmp/a.tar.gz | awk '{print $$1}') \
	     <(sha256sum /tmp/b.tar.gz | awk '{print $$1}')
	@echo "OK: reproducible"
```

`-trimpath` strips absolute paths; `-buildid=` zeroes the build ID; `--mtime` + `--sort=name` make the tar deterministic. Together these satisfy AC #30.

---

## §7 — Dependencies

- **FR-CHAT-001** — fork.
- **FR-AUTH-004** — JWT + JWKS endpoint.
- **FR-AUTH-005** — admin REST for team-tenant mapping CRUD.
- **FR-CHAT-003 (downstream)** — Fargate deployment runs this plugin.

---

## §8 — Example payloads

### Audit row — successful login (reuse path)

```json
{
  "kind": "chat.session_started",
  "ts_ns": 1747958400000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "subject_id": "7e57c0de-cafe-babe-dead-beef00000001",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "payload": {
    "mm_user_id": "qpw3...",
    "cyberos_subject_id": "7e57c0de-cafe-babe-dead-beef00000001",
    "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "jit_provisioned": false,
    "jwt_jti": "jti-abc-123",
    "jwt_iss": "https://auth.cyberskill.world",
    "jwt_exp": 1747959300
  }
}
```

### Audit row — successful login (JIT provision)

```json
{
  "kind": "chat.session_started",
  "ts_ns": 1747958410000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "subject_id": "7e57c0de-cafe-babe-dead-beef00000002",
  "trace_id": "8a1b2c3d4e5f60718293a4b5c6d7e8f9",
  "payload": {
    "mm_user_id": "rxv9...",
    "cyberos_subject_id": "7e57c0de-cafe-babe-dead-beef00000002",
    "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "jit_provisioned": true,
    "username_sanitised": "newuser",
    "collision_suffix": 0
  }
}
```

### Error envelope examples

```json
// 401 invalid_jwt
{
  "error": "invalid_jwt",
  "reason": "signature verification failed",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "retry_after_seconds": null
}

// 401 missing_tenant_claim
{
  "error": "missing_tenant_claim",
  "reason": "JWT does not contain 'tenant_id'",
  "trace_id": "1a2b3c4d5e6f70819203a4b5c6d7e8f9",
  "retry_after_seconds": null
}

// 403 tenant_mismatch
{
  "error": "tenant_mismatch",
  "reason": "jwt.tenant_id != user.cyberos_tenant_id",
  "trace_id": "2b3c4d5e6f7081920304a5b6c7d8e9f0",
  "retry_after_seconds": null
}

// 401 revoked (deny-list hit)
{
  "error": "revoked",
  "reason": "jti listed in revocation deny-list",
  "trace_id": "3c4d5e6f70819203040516b7c8d9e0f1",
  "retry_after_seconds": null
}

// 401 revoked (fail-secure due to service outage)
{
  "error": "revoked",
  "reason": "revocation service unreachable; fail-secure",
  "trace_id": "4d5e6f7081920304051617c8d9e0f102",
  "retry_after_seconds": 30
}

// 503 jwks_unavailable (cold cache)
{
  "error": "jwks_unavailable",
  "reason": "JWKS endpoint unreachable and cache cold",
  "trace_id": "5e6f708192030405061718d9e0f10213",
  "retry_after_seconds": 10
}

// 500 tenant_not_mapped
{
  "error": "tenant_not_mapped",
  "reason": "no cyberos_chat_tenant_map row for tenant_id",
  "trace_id": "6f7081920304050617181920e0f10213",
  "retry_after_seconds": null
}

// 500 username_collision_exhausted
{
  "error": "username_collision_exhausted",
  "reason": "100 suffix attempts; manual intervention required",
  "trace_id": "70819203040506071819202122f01323",
  "retry_after_seconds": null
}

// 405 builtin_auth_disabled (from patch 010, not the plugin)
{
  "error": "builtin_auth_disabled",
  "reason": "Mattermost built-in password flow is disabled; use CyberOS JWT",
  "trace_id": "8192030405060718192021222324f425",
  "retry_after_seconds": null
}
```

### Successful 200 body

```json
{
  "session_id": "qpw3abc...",
  "user_id": "rxv9def...",
  "expires_at": 1747959300
}
```

### Inbound traceparent → outbound trace_id

```http
POST /plugins/cyberos.authbridge/login HTTP/1.1
Authorization: Bearer eyJ...
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01

→ audit row.trace_id = "4bf92f3577b34da6a3ce929d0e0e4736"
```

---

## §9 — Open questions

All resolved. Deferred:
- SCIM provisioning for IT-admin-driven user lifecycle — slice 4+.
- Multi-team membership per tenant (different channels per dept) — slice 4+.
- SSO with WebAuthn passkey via FR-AUTH-004 — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid JWT signature | parse error in jwt.ParseWithClaims | 401 `invalid_jwt`; metric `outcome=invalid_jwt` | User obtains fresh JWT |
| JWKS endpoint down, cache warm | refresh goroutine error | Logins continue from cache; metric `chat_authbridge_jwks_refresh_total{result=err}` increments; SEV-3 alert at 1h | Operator restores AUTH-004; cache repopulates on next refresh |
| JWKS endpoint down, cache cold | prime fetch in OnActivate returns err | OnActivate fails → plugin not activated → Mattermost continues without authbridge → patch 010 returns 405 to all logins → all logins fail closed | Operator restores AUTH-004 + restart Mattermost pod |
| JWKS kid rotation | KeyFunc cache miss | Synchronous refresh; if new kid present, login succeeds; else 401 | Operator confirms FR-AUTH-004 rotation; should be transparent |
| Email collision (same email, different subject) | `GetUserByUsername` returns existing user | jit_provision appends `_N` suffix to username | None — automatic |
| Username collision suffix exhaustion (>100) | 100th attempt also collides | 500 `username_collision_exhausted`; SEV-2 page | Operator investigates spam/abuse pattern; manual username assignment |
| Email contains only special chars | sanitiseUsername output < 3 chars | Username padded with `u` to reach 3-char minimum | None |
| Tenant team not mapped in `cyberos_chat_tenant_map` | tenantMap.Lookup returns err | 500 `tenant_not_mapped`; SEV-2 page | Operator adds mapping via FR-AUTH-005 admin REST |
| FR-AUTH-005 webhook for mapping change lost | Stale cache TTL 5min | Stale mapping for ≤5min; new login uses stale team_id | Operator manually invalidates via plugin admin endpoint |
| Plugin process crash | Mattermost SIGSEGV log | Pod down + Mattermost restart loop until plugin recovers OR is uninstalled | Operator: roll back plugin upgrade or pull plugin off `/opt/cyberos/plugins/` |
| Revocation lookup service unreachable | revoker.IsRevoked returns err | Fail-SECURE: 401 `revoked`; SEV-2 warning logged; logins blocked for affected JTIs | Operator restores revocation service; behaviour is intentional |
| Revocation deny-list returns false positive | jti incorrectly marked revoked | 401 `revoked` for legitimate user | Operator removes jti from deny-list via FR-AUTH-005; cache TTL 60s |
| Subject_id with special chars (Unicode, RTL) | Encoded into `Props["cyberos_subject_id"]` as-is | Mattermost stores raw; downstream consumers MUST treat as opaque | None |
| Cross-tenant JWT replay (token from tenant-A used for tenant-B team) | tenant_id != user's existing team | 403 `tenant_mismatch`; metric `outcome=tenant_mismatch` | Investigate; possible compromise |
| Mattermost session limit per user (MM-1000) | CreateSession returns AppError code 1000 | 403 with reason "session_limit"; user must logout elsewhere | User action |
| Plugin upgrade mid-session | Old plugin process drains; new plugin starts | Existing sessions persist (cookie-based); new logins use new plugin | None — Mattermost handles hot-reload |
| OnActivate config missing required key | `cfg["jwks_url"].(string)` panics on nil | Plugin Activate returns err; Mattermost logs Fatal; pod crashes | Operator sets PluginSettings via System Console |
| JWKS goroutine panics during refresh | Recovered in refreshLoop; logged + metric | Refresh skipped; next tick retries; SEV-3 alert at 3 failures | Operator investigates JWKS endpoint changes |
| JWKS HTTP 5xx response | refresh returns err with status | Cached set retained; refresh retries on next tick | Operator restores AUTH-004 |
| JWKS HTTP 4xx response (auth misconfig) | refresh returns err with `http_403` | SEV-2: cache cannot refresh; plugin will fail-secure at TTL expiry | Operator fixes JWKS endpoint access |
| Two concurrent first-logins for same subject_id | jitGate serialises | Second login waits, then reuses created user | None — automatic |
| Clock skew (JWT `nbf` slightly in future) | parse rejects with `Token not valid yet` | 401 `invalid_jwt` | Sync server clock via NTP; AUTH-004 emits with -5s slack |
| Mattermost team deleted while user is signing in | tenantMap returns stale team_id; GetUsersInTeam errs | 500 `server_error`; SEV-2 | Operator recreates team or removes mapping |
| Login burst exceeds CreateSession rate limit | Mattermost AppError "rate_limited" | 503 with `retry_after_seconds` in envelope | Client backs off |
| Audit emit fails (memory unreachable) | memory client returns err | Login still succeeds; SEV-2 logged + counter `chat_authbridge_audit_emit_failures_total` | Operator restores memory; audit row may be lost |
| Concurrent OnDeactivate calls during shutdown | stopCh close racing | First closes; second is no-op (select-default) | None |
| Two pods see different JWKS during rotation window | Inconsistent kid resolution across replicas | <5min until both pods refresh | None — eventual consistency |
| Token's `iss` doesn't match expected | claims["iss"] check fails | 401 `invalid_jwt`; SEV-2 if widespread | Operator investigates JWT issuer drift |
| memory audit row schema rejection (kind not whitelisted) | memory returns 422 | Logged; login still succeeds; metric `audit_emit_failures_total{reason=schema}` | Coordinate with FR-AI-003 closed-set extension |
| JIT-provisioned user fails Mattermost team-add | API.AddUserToTeam returns err | Login returns 500 `server_error`; user record exists but not in team — admin must repair | Operator runs `cyberos chat repair-user --subject <id>` |
| Plugin .tar.gz checksum mismatch on Mattermost load | Mattermost log `plugin_signature_mismatch` | Plugin fails to load; OnActivate never called → builtin auth disabled → all logins 405 | Operator re-uploads plugin or rolls back fork |

---

## §11 — Implementation notes

- Mattermost plugin SDK requires Go 1.22+; matches fork.
- JWKS cache uses `lestrrat-go/jwx` for parsing (canonical Go JWKS library); the cache layer is bespoke because the `jwx/jwk.Cache` couples too tightly to its own HTTP client and obscures the proactive-refresh behaviour we want for observability.
- The RCU pattern in `JwksCache` (`atomic.Pointer[jwk.Set]`) means readers never block on refresh; the entire keyset is swapped atomically.
- JIT provision uses Mattermost API `CreateUser`; emails MUST be unique per Mattermost installation. The username collision suffix is `_N` (not `_001`) to stay inside the 22-char username limit.
- The patch `011-load-authbridge-plugin.patch` hardcodes the plugin path `/opt/cyberos/plugins/cyberos-authbridge.tar.gz`. This is fine because: (a) the Mattermost container baseline is owned by us via FR-CHAT-003 Dockerfile, (b) packaging the plugin into the container image makes the upgrade path "rebuild image + redeploy" instead of "drop file into running container."
- Tenant_team mapping table `cyberos_chat_tenant_map` is owned by FR-AUTH-005 admin REST; plugin reads it via a Postgres connection that's separate from Mattermost's. We don't pipe the lookup through Mattermost's `pluginAPI.KVStore` because (a) KVStore is per-plugin and we'd duplicate the data, (b) FR-AUTH-005 needs CRUD via REST which doesn't speak KVStore.
- `AuthService: "cyberos-jwt"` flag on user prevents accidental password reset attempts. Mattermost's password-reset path checks this flag and short-circuits with "use your SSO provider" — which is harmless because the endpoint is already gone via patch 010, but defence in depth.
- The revocation lookup uses HTTP polling with a 60s LRU cache rather than a streaming subscription. Rationale: simpler operationally; 60s revocation lag is acceptable for a B2B product (FR-AUTH-004 §1 #19 sets the SLA at 5 minutes); the streaming complexity isn't justified.
- `runtime.NumGoroutine()` in the leak test is approximate — the test asserts `delta <= 10` rather than `delta == 0` because Go's runtime spawns and reaps goroutines for GC and scheduler bookkeeping; the threshold is tuned per CI.
- The fail-secure-on-revocation-outage choice (AC #27) is deliberate. We chose secure-by-default over uptime-by-default because (a) the revocation list is the only mechanism to invalidate a JWT before its `exp`, (b) a service degradation that lets revoked tokens log in is a security incident, (c) operators have alerting on `outcome=revoked` spikes that signal a service outage, so they'll notice and remediate.
- Why we don't cache revocation results positively: caching "not revoked" introduces a window where a token revoked at `t` is still considered valid until `t + cache_TTL`. This is wider than the deliberate 60s polling window because the cache is per-pod (no cache invalidation across pods).
- The `jitGate` is a per-process serialiser. In a multi-pod deployment, two pods receiving simultaneous first-logins for the same subject can still race in CreateUser; the second loses with a Mattermost duplicate-email error, which the plugin maps to "retry once after delay 100ms." The race window is small enough in practice (~10ms) that we don't introduce cross-pod coordination (Redis lock would add complexity for no real benefit).
- We chose ES256 over RS256 for JWT signing (in coordination with FR-AUTH-004): smaller signatures (64 bytes vs 256 bytes), better mobile-bandwidth profile, faster verification (constant-time scalar mul vs modular exponentiation).
- Metric label cardinality is bounded: `outcome` has 9 stable values (AC #18 closed-enum); `result` on `chat_authbridge_jwks_cache_hits_total` has 2 values (`hit`/`miss`). Total label cardinality stays well under the 10k-series Prometheus practical ceiling.
- The W3C `traceparent` parsing follows the v2 spec (`00-` version prefix); v1 (`01-`) is not yet specified and would be a future amendment.
- The plugin does NOT issue refresh tokens — the JWT is the only credential. Mattermost session refresh happens by re-presenting the JWT to `/login`. This is a deliberate choice: FR-AUTH-004 owns the refresh-token lifecycle; the chat plugin stays stateless and lean.
- Why team=tenant (single team per tenant) instead of team-per-channel-category: Mattermost's permission model is team-scoped; trying to subdivide tenants into multiple teams would require per-message tenant-id filtering in queries, which we avoid by mapping tenant 1:1 to team. The cost is that all channels in a tenant share a single sidebar; the benefit is that the tenant boundary is enforceable via Mattermost's existing primitive.

---

*End of FR-CHAT-002.*
