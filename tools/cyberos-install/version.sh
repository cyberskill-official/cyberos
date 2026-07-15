#!/usr/bin/env bash
# version.sh — check whether a newer CyberOS is available (manual).
#
# Soft update-checks already run automatically on any .cyberos use (gates, hooks, MCP, …).
# This command is the explicit human check. Re-vendor is always `install` (never a second path).
#
#   bash .cyberos/version.sh [repo]
#   bash <payload>/version.sh [repo]
#
# If stale and stdin is a TTY, asks: update now? y → runs install.sh from this payload.
# Non-interactive (CI / no TTY / CYBEROS_NONINTERACTIVE=1): report only.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

root=""
for a in "$@"; do
  case "$a" in
    -*) echo "cyberos version: unknown flag $a (this command only checks; use install to re-vendor)" >&2; exit 2 ;;
    *)
      if [ -z "$root" ]; then root="$a"
      else echo "cyberos version: unexpected arg $a" >&2; exit 2
      fi
      ;;
  esac
done
if [ -n "$root" ]; then
  root="$(cd "$root" 2>/dev/null && pwd)" || { echo "cyberos version: bad root: $root" >&2; exit 2; }
else
  root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi

# Soft library check (always for manual invocation)
if [ -f "$here/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$here/lib/update-check.sh"
  CYBEROS_UPDATE_CHECK="${CYBEROS_UPDATE_CHECK:-always}" _cyberos_update_check || true
fi

inst="none"
[ -f "$root/.cyberos/VERSION" ] && inst="$(tr -d ' \n\r' < "$root/.cyberos/VERSION")"
payload_ver="unknown"
if [ -f "$here/VERSION" ]; then
  payload_ver="$(tr -d ' \n\r' < "$here/VERSION")"
elif [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/VERSION" ]; then
  payload_ver="$(tr -d ' \n\r' < "${CYBEROS_PAYLOAD}/VERSION")"
fi
latest_line="latest=unknown source=offline"
if [ "${CYBEROS_OFFLINE:-0}" != "1" ]; then
  if [ -f "$here/check-latest.sh" ]; then
    latest_line="$(bash "$here/check-latest.sh" 2>/dev/null || echo "latest=unknown source=offline")"
  elif [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/check-latest.sh" ]; then
    latest_line="$(bash "${CYBEROS_PAYLOAD}/check-latest.sh" 2>/dev/null || echo "latest=unknown source=offline")"
  fi
fi
latest="${latest_line#latest=}"; latest="${latest%% *}"

# rules_sha — the rule-content fingerprint (TASK-IMP-074 §10). Reported alongside the versions
# because two payloads can share a VERSION and still ship different rules.
_rs() {
  [ -f "${1:-}" ] || return 1
  local v; v="$(grep -E '^rules_sha:' "$1" 2>/dev/null | head -1 | awk '{print $2}' | tr -d ' \n\r')"
  printf '%s' "$v"; [ -n "$v" ]
}
inst_sha="$(_rs "$root/.cyberos/manifest.yaml" || true)"
pay_sha="$(_rs "$here/manifest.yaml" || true)"
if [ -z "$pay_sha" ] && [ -n "${CYBEROS_PAYLOAD:-}" ]; then
  pay_sha="$(_rs "${CYBEROS_PAYLOAD}/manifest.yaml" || true)"
fi

echo "installed=$inst"
echo "payload=$payload_ver"
echo "$latest_line"
echo "installed_rules_sha=${inst_sha:-none}"
echo "payload_rules_sha=${pay_sha:-none}"

is_ver() { printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; }
ver_lt() { [ "$1" = "$2" ] && return 1; [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]; }

verdict="up_to_date"
if is_ver "$latest" && is_ver "$payload_ver" && ver_lt "$payload_ver" "$latest"; then
  verdict="payload_stale"
elif [ "$inst" = "none" ]; then
  verdict="not_installed"
elif { is_ver "$latest" && is_ver "$inst" && ver_lt "$inst" "$latest"; } \
  || { is_ver "$inst" && is_ver "$payload_ver" && ver_lt "$inst" "$payload_ver"; }; then
  verdict="repo_stale"
elif [ -n "$pay_sha" ] && [ "$inst_sha" != "$pay_sha" ]; then
  # Same VERSION, different rules — invisible to the version compare above. This is the
  # case that let 23/24 repos keep running the pre-rename ruleset while reporting healthy.
  verdict="rules_drift"
fi

echo "verdict=$verdict"
case "$verdict" in
  payload_stale)
    echo "next: fetch latest payload, then bash <payload>/install.sh $root"
    echo "  curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz | tar -xz -C /tmp && bash /tmp/cyberos/install.sh $root"
    ;;
  not_installed)
    echo "next: bash ${CYBEROS_PAYLOAD:-$here}/install.sh $root"
    ;;
  repo_stale)
    echo "next: bash ${CYBEROS_PAYLOAD:-$here}/install.sh $root"
    ;;
  rules_drift)
    echo "  same version ($inst), different rules — vendored copy does not match this payload"
    echo "next: bash ${CYBEROS_PAYLOAD:-$here}/install.sh $root"
    ;;
  up_to_date)
    case "$latest_line" in latest=unknown*) echo "  note: remote check skipped or unavailable — answer only as fresh as the local payload" ;; esac
    exit 0
    ;;
esac

# Offer install when we can apply from this payload (repo_stale / not_installed with install.sh beside us)
installer=""
if [ -f "$here/install.sh" ]; then installer="$here/install.sh"
elif [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/install.sh" ]; then installer="${CYBEROS_PAYLOAD}/install.sh"
fi

if [ -z "$installer" ]; then
  echo "cyberos version: no install.sh reachable (set CYBEROS_PAYLOAD) — apply with install after fetching payload" >&2
  exit 0
fi

if [ "${CYBEROS_NONINTERACTIVE:-0}" = "1" ] || [ ! -t 0 ]; then
  echo "cyberos version: non-interactive — re-vendor with: bash $installer $root"
  exit 0
fi

prompt="Update CyberOS in this repo now (runs install)?"
[ "$verdict" = "not_installed" ] && prompt="Install CyberOS in this repo now?"
printf '%s [y/N] ' "$prompt" >&2
read -r ans || ans=""
case "$ans" in
  y|Y|yes|YES)
    echo "cyberos version: running $installer $root"
    exec bash "$installer" "$root"
    ;;
  *)
    echo "cyberos version: skipped. When ready: bash $installer $root"
    exit 0
    ;;
esac
