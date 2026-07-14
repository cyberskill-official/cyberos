#!/usr/bin/env bash
# status.sh — open the repo's CyberOS status page (docs/status/index.html) in the default browser.
# That is the only job of this command. Soft update-check still runs (any .cyberos use).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

root=""
if [ -n "${1:-}" ]; then
  root="$(cd "$1" 2>/dev/null && pwd)" || root="$1"
else
  root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi

if [ -f "$here/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$here/lib/update-check.sh"
  _cyberos_update_check || true
fi

page="$root/docs/status/index.html"
if [ ! -f "$page" ]; then
  # try regenerate once (internal)
  if [ -f "$root/.cyberos/lib/status-page.sh" ]; then
    bash "$root/.cyberos/lib/status-page.sh" "$root" >/dev/null 2>&1 || true
  elif [ -f "$here/lib/status-page.sh" ]; then
    bash "$here/lib/status-page.sh" "$root" >/dev/null 2>&1 || true
  fi
fi

if [ ! -f "$page" ]; then
  echo "cyberos status: no docs/status/index.html yet (install + add FRs, or open after first install migrate)" >&2
  echo "  expected: $page" >&2
  exit 1
fi

# Prefer file:// URL for default browser
abs="$(cd "$(dirname "$page")" && pwd)/$(basename "$page")"
url="file://${abs}"

echo "cyberos status: opening $abs"
if command -v open >/dev/null 2>&1; then
  open "$abs"   # macOS
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$abs" >/dev/null 2>&1 &
elif command -v wslview >/dev/null 2>&1; then
  wslview "$abs" >/dev/null 2>&1 &
elif command -v cmd.exe >/dev/null 2>&1; then
  cmd.exe /c start "" "$abs" >/dev/null 2>&1 &
else
  echo "cyberos status: no browser opener found — open manually: $url" >&2
  exit 0
fi
exit 0
