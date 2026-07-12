#!/usr/bin/env bash
# update.sh - root CLI entry (FR-IMP-076): check/apply CyberOS updates directly from the shell.
# Mirrors the plugin's /cyberos:update: read-only by default, --apply to actually update.
# Thin wrapper over init.sh, which owns the real logic (idempotent vendoring + --check report).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
[ -f "$here/init.sh" ] || { echo "cyberos: init.sh not found beside update.sh" >&2; exit 2; }
if [ "${1:-}" = "--apply" ]; then
  shift
  exec bash "$here/init.sh" "$@"
fi
exec bash "$here/init.sh" --check "$@"
