#!/usr/bin/env bash
# update.sh — check/apply CyberOS updates (always runs update-check first).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# When installed under .cyberos/, lib is sibling
if [ -f "$here/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$here/lib/update-check.sh"
  CYBEROS_UPDATE_CHECK="${CYBEROS_UPDATE_CHECK:-always}" _cyberos_update_check || true
fi
[ -f "$here/init.sh" ] || { echo "cyberos: init.sh not found beside update.sh" >&2; exit 2; }
case "${1:-}" in
  --apply)
    shift
    exec bash "$here/init.sh" "$@"
    ;;
  --check)
    shift
    ;;
esac
exec bash "$here/init.sh" --check "$@"
