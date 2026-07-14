#!/usr/bin/env bash
# update.sh — manual CyberOS update check (and optional apply).
#
# Soft update checks already run automatically whenever anything under .cyberos is used
# (gates, help, status-page hooks, MCP tools, …). Use this command when you want an
# explicit, human-triggered check — or to apply a newer payload.
#
#   bash .cyberos/update.sh              # check only (installed / payload / latest)
#   bash .cyberos/update.sh --apply      # re-vendor from this payload / $CYBEROS_PAYLOAD
#   bash <payload>/update.sh [repo] [--apply]
#
# First-time setup is install.sh (once). This is not install.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

apply=0
root=""
for a in "$@"; do
  case "$a" in
    --apply) apply=1 ;;
    --check) ;; # synonym for check-only
    -*) echo "cyberos update: unknown flag $a" >&2; exit 2 ;;
    *)
      if [ -z "$root" ]; then root="$a"
      else echo "cyberos update: unexpected arg $a" >&2; exit 2
      fi
      ;;
  esac
done
if [ -n "$root" ]; then
  root="$(cd "$root" 2>/dev/null && pwd)" || { echo "cyberos update: bad root: $root" >&2; exit 2; }
else
  root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi

# Soft library check (always mode for manual invocation)
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

echo "installed=$inst"
echo "payload=$payload_ver"
echo "$latest_line"

is_ver() { printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; }
ver_lt() { [ "$1" = "$2" ] && return 1; [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]; }

if is_ver "$latest" && is_ver "$payload_ver" && ver_lt "$payload_ver" "$latest"; then
  echo "verdict=payload_stale"
  echo "next: curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz | tar -xz -C /tmp && bash /tmp/cyberos/install.sh $root"
elif [ "$inst" = "none" ]; then
  echo "verdict=not_installed"
  echo "next: bash ${CYBEROS_PAYLOAD:-$here}/install.sh $root"
elif { is_ver "$latest" && is_ver "$inst" && ver_lt "$inst" "$latest"; } \
  || { is_ver "$inst" && is_ver "$payload_ver" && ver_lt "$inst" "$payload_ver"; }; then
  echo "verdict=repo_stale"
  echo "next: bash ${CYBEROS_PAYLOAD:-$here}/update.sh --apply $root"
  echo "  or: bash ${CYBEROS_PAYLOAD:-$here}/install.sh $root"
else
  echo "verdict=up_to_date"
  case "$latest_line" in latest=unknown*) echo "  note: remote check skipped or unavailable - answer only as fresh as the local payload" ;; esac
fi

if [ "$apply" = 1 ]; then
  installer=""
  if [ -f "$here/install.sh" ]; then installer="$here/install.sh"
  elif [ -n "${CYBEROS_PAYLOAD:-}" ] && [ -f "${CYBEROS_PAYLOAD}/install.sh" ]; then installer="${CYBEROS_PAYLOAD}/install.sh"
  else
    echo "cyberos update: ERROR no install.sh beside this update.sh (set CYBEROS_PAYLOAD)" >&2
    exit 2
  fi
  echo "cyberos update: applying via $installer $root"
  exec bash "$installer" "$root"
fi
exit 0
