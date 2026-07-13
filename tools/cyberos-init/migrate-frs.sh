#!/usr/bin/env bash
# migrate-frs.sh — compatibility shim. Prefer: bash init.sh --page|.|--migrate
# Combined logic lives in lib/fr-migrate.sh, invoked by init.sh.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
init="$here/init.sh"
if [ ! -f "$init" ]; then
  # installed under .cyberos/ — sibling init may not exist; use lib directly
  lib="$here/lib/fr-migrate.sh"
  [ -f "$lib" ] || { echo "migrate-frs: missing $lib (re-run cyberos init)" >&2; exit 2; }
  # shellcheck source=/dev/null
  source "$lib"
  PAGE_ONLY=0
  if [ "${1:-}" = "--page" ]; then PAGE_ONLY=1; shift; fi
  root="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
  _cyberos_fr_migrate "$root" "$here"
  exit $?
fi
if [ "${1:-}" = "--page" ]; then
  shift
  exec bash "$init" --page "${1:-.}"
fi
exec bash "$init" --migrate "${1:-.}"
