#!/usr/bin/env bash
# update.sh - root CLI entry (FR-IMP-076): check/apply CyberOS updates directly from the shell.
# Mirrors the plugin's /cyberos:update: read-only by default, --apply to actually update.
# Thin wrapper over init.sh, which owns the real logic (idempotent vendoring + --check report).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
[ -f "$here/init.sh" ] || { echo "cyberos: init.sh not found beside update.sh" >&2; exit 2; }
case "${1:-}" in
  --apply)
    shift
    exec bash "$here/init.sh" "$@"
    ;;
  --check)
    # Drop the explicit flag - it is re-added below. Without this shift, init.sh received
    # "--check --check" and read the second one as its TARGET argument ("cd: --: invalid
    # option", root detection broken) - caught by the FR-IMP-076 testing pass 2026-07-13.
    shift
    ;;
esac
exec bash "$here/init.sh" --check "$@"
