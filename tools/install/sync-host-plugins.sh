#!/usr/bin/env bash
# sync-host-plugins.sh — refresh Claude Code + Grok host plugin installs from a built payload.
#
# Why this exists: the payload at dist/cyberos is the source of truth, but host agents keep
# their own install caches (Claude: ~/.claude/plugins/cache/...; Grok: ~/.grok/installed-plugins/).
# A rebuild does not update those caches. Soft /version checks only cover the REPO machine
# (.cyberos/), not the host plugin. Without this step, operators keep shipping an old plugin
# while dist/cyberos is already new (the 1.2.0-stuck-while-dist-is-1.0.7 class of bug).
#
# usage:
#   bash tools/install/sync-host-plugins.sh [payload-dir]
#     default payload-dir: <repo>/dist/cyberos
#
# env:
#   CYBEROS_SYNC_HOST_PLUGINS_DRY_RUN=1   print plan only; never run host CLIs
#   CYBEROS_SYNC_CLAUDE=0                 skip Claude even if claude is on PATH
#   CYBEROS_SYNC_GROK=0                   skip Grok even if grok is on PATH
#   CYBEROS_OFFLINE=1                     same skip signal for both hosts
#
# exit:
#   0  all attempted hosts ok, or every host skipped (cli missing / disabled)
#   1  at least one host attempted and failed
#   2  payload unreadable / not a built marketplace
#
# Called best-effort from build.sh after a successful default-path assemble. Never fatal to
# the build: missing CLIs are skips, not errors. Force reinstall (uninstall + install) so
# version DOWNGRADES and same-version rule content bumps still land.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
payload="${1:-$repo/dist/cyberos}"
payload="$(cd "$payload" 2>/dev/null && pwd -P)" || {
  echo "cyberos sync-host-plugins: ERROR payload dir missing: ${1:-$repo/dist/cyberos}" >&2
  exit 2
}

plugin_dir="$payload/plugin"
manifest="$plugin_dir/.claude-plugin/plugin.json"
market="$payload/.claude-plugin/marketplace.json"

[ -f "$manifest" ] || {
  echo "cyberos sync-host-plugins: ERROR not a built plugin payload (missing $manifest)" >&2
  exit 2
}
[ -f "$market" ] || {
  echo "cyberos sync-host-plugins: ERROR not a marketplace root (missing $market)" >&2
  exit 2
}

expected=""
if [ -f "$payload/VERSION" ]; then
  expected="$(tr -d ' \n\r' < "$payload/VERSION")"
fi
if [ -z "$expected" ] && command -v node >/dev/null 2>&1; then
  expected="$(node -e 'const j=require(process.argv[1]);process.stdout.write(j.version||"")' "$manifest" 2>/dev/null || true)"
fi
[ -n "$expected" ] || {
  echo "cyberos sync-host-plugins: ERROR cannot read payload version" >&2
  exit 2
}

dry=0
[ "${CYBEROS_SYNC_HOST_PLUGINS_DRY_RUN:-0}" = "1" ] && dry=1
[ "${CYBEROS_OFFLINE:-0}" = "1" ] && {
  echo "cyberos sync-host-plugins: skip (CYBEROS_OFFLINE=1)"
  exit 0
}

echo "cyberos sync-host-plugins: payload=$payload version=$expected"

failed=0
skipped=0
synced=0

run() {
  # shellcheck disable=SC2145
  if [ "$dry" -eq 1 ]; then
    echo "  dry-run: $*"
    return 0
  fi
  "$@"
}

# --- Claude Code -------------------------------------------------------------
sync_claude() {
  if [ "${CYBEROS_SYNC_CLAUDE:-1}" = "0" ]; then
    echo "claude: skip (CYBEROS_SYNC_CLAUDE=0)"
    skipped=$((skipped + 1))
    return 0
  fi
  if ! command -v claude >/dev/null 2>&1; then
    echo "claude: skip (cli not on PATH)"
    skipped=$((skipped + 1))
    return 0
  fi

  echo "claude: ensuring marketplace 'cyberos' -> $payload"
  # add is idempotent enough for directory sources; if name already exists pointing
  # elsewhere, update refreshes the checkout/path. Either way we reinstall the plugin.
  if [ "$dry" -eq 1 ]; then
    echo "  dry-run: claude plugin marketplace add $payload   # or update if present"
    echo "  dry-run: claude plugin uninstall cyberos@cyberos -y"
    echo "  dry-run: claude plugin install cyberos@cyberos"
    synced=$((synced + 1))
    return 0
  fi

  # Prefer update of the named marketplace when present; fall back to add.
  if claude plugin marketplace list 2>/dev/null | grep -qE '(^|[[:space:]])cyberos([[:space:]]|$)'; then
    claude plugin marketplace update cyberos 2>/dev/null \
      || claude plugin marketplace add "$payload" 2>/dev/null \
      || true
  else
    claude plugin marketplace add "$payload" 2>/dev/null \
      || true
  fi

  # Force reinstall so a lower VERSION (product reset) or same-version content bump
  # replaces the cache. `plugin update` alone can no-op or refuse a "downgrade".
  claude plugin uninstall cyberos@cyberos -y 2>/dev/null || true
  if ! claude plugin install cyberos@cyberos; then
    echo "claude: FAIL install cyberos@cyberos" >&2
    failed=$((failed + 1))
    return 0
  fi

  # Verify the active cache reports the expected version when the cache dir exists.
  local cache_root="$HOME/.claude/plugins/cache/cyberos/cyberos"
  if [ -d "$cache_root/$expected" ]; then
    echo "claude: ok cache=$cache_root/$expected"
    synced=$((synced + 1))
    return 0
  fi
  # installed_plugins.json is the authoritative pointer even if directory name differs.
  if [ -f "$HOME/.claude/plugins/installed_plugins.json" ] && command -v node >/dev/null 2>&1; then
    local got
    got="$(node -e '
      const fs=require("fs");
      const d=JSON.parse(fs.readFileSync(process.env.HOME+"/.claude/plugins/installed_plugins.json","utf8"));
      const arr=(d.plugins&&d.plugins["cyberos@cyberos"])||[];
      const v=(arr[0]&&arr[0].version)||"";
      process.stdout.write(v);
    ' 2>/dev/null || true)"
    if [ "$got" = "$expected" ]; then
      echo "claude: ok installed_plugins version=$got"
      synced=$((synced + 1))
      return 0
    fi
    echo "claude: FAIL expected version=$expected got=${got:-<none>}" >&2
    failed=$((failed + 1))
    return 0
  fi

  echo "claude: ok install returned success (could not verify cache path)"
  synced=$((synced + 1))
}

# --- Grok --------------------------------------------------------------------
sync_grok() {
  if [ "${CYBEROS_SYNC_GROK:-1}" = "0" ]; then
    echo "grok: skip (CYBEROS_SYNC_GROK=0)"
    skipped=$((skipped + 1))
    return 0
  fi
  if ! command -v grok >/dev/null 2>&1; then
    echo "grok: skip (cli not on PATH)"
    skipped=$((skipped + 1))
    return 0
  fi

  echo "grok: reinstalling from $plugin_dir"
  if [ "$dry" -eq 1 ]; then
    echo "  dry-run: grok plugin install $plugin_dir --trust"
    synced=$((synced + 1))
    return 0
  fi

  # Local installs are snapshotted into ~/.grok/installed-plugins/. `plugin update` on a
  # local source often reports "already live" without re-copying. Re-install is the
  # reliable refresh (same pattern as the manual recovery that fixed the stuck cache).
  if ! grok plugin install "$plugin_dir" --trust; then
    echo "grok: FAIL install $plugin_dir" >&2
    failed=$((failed + 1))
    return 0
  fi

  # Confirm grok sees cyberos at the expected version when `plugin details` works.
  local details
  details="$(grok plugin details cyberos 2>/dev/null || true)"
  if printf '%s' "$details" | grep -qE "cyberos v?${expected}|version[: ]+${expected}|v${expected}"; then
    echo "grok: ok cyberos v$expected"
    synced=$((synced + 1))
    return 0
  fi
  # Fall back to registry.json
  if [ -f "$HOME/.grok/installed-plugins/registry.json" ] && command -v node >/dev/null 2>&1; then
    local got
    got="$(node -e '
      const fs=require("fs");
      const d=JSON.parse(fs.readFileSync(process.env.HOME+"/.grok/installed-plugins/registry.json","utf8"));
      let v="";
      for (const r of Object.values(d.repos||{})) {
        if (r.plugins && r.plugins.cyberos && r.plugins.cyberos.version) { v=r.plugins.cyberos.version; break; }
      }
      process.stdout.write(v);
    ' 2>/dev/null || true)"
    if [ "$got" = "$expected" ]; then
      echo "grok: ok registry version=$got"
      synced=$((synced + 1))
      return 0
    fi
    echo "grok: FAIL expected version=$expected got=${got:-<none>}" >&2
    failed=$((failed + 1))
    return 0
  fi

  echo "grok: ok install returned success (could not verify registry)"
  synced=$((synced + 1))
}

sync_claude
sync_grok

echo "cyberos sync-host-plugins: synced=$synced skipped=$skipped failed=$failed"
[ "$failed" -eq 0 ] || exit 1
exit 0
