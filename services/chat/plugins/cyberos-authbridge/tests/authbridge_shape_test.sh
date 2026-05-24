#!/usr/bin/env bash
# FR-CHAT-002 structural tests. Go is intentionally not required in the
# agent/runtime image; these checks pin the authbridge source contract.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)"

fail() { echo "::error::$*" >&2; exit 1; }
ok() { echo "  ✓ $*"; }

[[ -f "$ROOT/plugin.json" ]] || fail "plugin.json missing"
grep -q '"id": "world.cyberos.chat.authbridge"' "$ROOT/plugin.json" || fail "plugin id mismatch"
ok "plugin manifest id pinned"

grep -q 'CompareAndSwap(false, true)' "$ROOT/main.go" || fail "OnActivate must reject double activation"
grep -q 'builtin_auth_disabled' "$ROOT/main.go" || fail "builtin password paths must be disabled"
grep -q 'tenant_mismatch' "$ROOT/main.go" || fail "tenant mismatch envelope missing"
grep -q 'traceparent' "$ROOT/main.go" || fail "traceparent propagation missing"
grep -q 'RevocationClient' "$ROOT/main.go" || fail "revocation fail-secure client missing"
grep -q 'validErrorCode' "$ROOT/main.go" || fail "closed error envelope guard missing"
ok "main auth flow markers present"

grep -q 'TTL() time.Duration' "$ROOT/jwks_cache.go" || fail "JWKS TTL helper missing"
grep -q 'time.Hour' "$ROOT/jwks_cache.go" || fail "JWKS TTL must be 1h"
grep -q 'TenantID string `json:"tenant_id"`' "$ROOT/jwks_cache.go" || fail "tenant_id claim missing"
grep -q 'rsa.VerifyPKCS1v15' "$ROOT/jwks_cache.go" || fail "RS256 signature verification missing"
grep -q 'http.NewRequestWithContext' "$ROOT/jwks_cache.go" || fail "JWKS fetch must be context-bound"
ok "JWT/JWKS claim markers present"

grep -q 'sync.Mutex' "$ROOT/jit_provision.go" || fail "JIT provisioner must serialize per-subject creates"
grep -q 'sanitizeUsername' "$ROOT/jit_provision.go" || fail "username sanitizer missing"
ok "JIT provision markers present"

echo "✓ FR-CHAT-002 authbridge structural tests pass"
