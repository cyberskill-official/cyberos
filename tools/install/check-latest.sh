#!/usr/bin/env bash
# check-latest.sh - resolve the latest PUBLISHED CyberOS version (TASK-IMP-070). Never breaks a
# caller: always exit 0, always exactly one line:
#   latest=<X.Y.Z|unknown> source=<endpoint|offline>
# env: CYBEROS_RELEASE_ENDPOINT  https URL or local file path; bare X.Y.Z or GitHub
#                                /releases/latest JSON (tag_name minus the v) both accepted
#      CYBEROS_OFFLINE=1         skip immediately (no read, no network)
set -uo pipefail

DEFAULT_ENDPOINT="https://api.github.com/repos/cyberskill-official/cyberos/releases/latest"
REDIRECT_URL="https://github.com/cyberskill-official/cyberos/releases/latest"

if [ "${CYBEROS_OFFLINE:-0}" = "1" ]; then echo "latest=unknown source=offline"; exit 0; fi

# Default path: the releases/latest REDIRECT first - its Location header names the tag and,
# unlike api.github.com, it is not subject to the 60/h unauthenticated API rate limit (observed
# live 2026-07-12: API 403 rate-limited while the release was published). API stays the fallback.
if [ -z "${CYBEROS_RELEASE_ENDPOINT:-}" ]; then
  loc="$(curl -sI --max-time 3 "$REDIRECT_URL" 2>/dev/null | tr -d '\r' | awk 'tolower($1)=="location:"{print $2}' | head -1)"
  tag="${loc##*/tag/}"
  if [ "$loc" != "$tag" ] && printf '%s' "${tag#v}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "latest=${tag#v} source=$REDIRECT_URL"; exit 0
  fi
fi
ep="${CYBEROS_RELEASE_ENDPOINT:-$DEFAULT_ENDPOINT}"

raw=""
if [ -f "$ep" ]; then
  raw="$(cat "$ep" 2>/dev/null || true)"
else
  raw="$(curl -sf --max-time 3 "$ep" 2>/dev/null || true)"
fi

ver=""
bare="$(printf '%s' "$raw" | tr -d ' \n\r')"
if printf '%s' "$bare" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  ver="$bare"
else
  tag="$(printf '%s' "$raw" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"v\{0,1\}\([0-9][^"]*\)".*/\1/')"
  printf '%s' "$tag" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' && ver="$tag"
fi

if [ -n "$ver" ]; then echo "latest=$ver source=$ep"; else echo "latest=unknown source=offline"; fi
