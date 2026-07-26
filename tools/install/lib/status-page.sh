#!/usr/bin/env bash
# status-page.sh — internal status-page regen (hooks + run-gates).
# NOT a user-facing command. Users: install once; page auto-syncs on commit/gates.
# Usage: bash .cyberos/lib/status-page.sh [repo-root] [--coverage-only]
#        CYBEROS_STATUS_COVERAGE_ONLY=1 bash .cyberos/lib/status-page.sh [repo-root]
#        source + _cyberos_status_page <root>
# shellcheck shell=bash

_cyberos_status_page() {
  local root="${1:-.}"
  local mode="${2:-}"
  root="$(cd "$root" 2>/dev/null && pwd)" || { echo "cyberos status-page: bad root: $1" >&2; return 2; }
  local cy="$root/.cyberos"
  local kit=""
  local install_kit
  install_kit="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)"
  # Platform self-host: prefer tools/install + tools/docs-site over a possibly-stale
  # .cyberos vendored copy so mothership hooks always run the source tree (TASK-DOCS-017).
  if [ -f "$root/tools/docs-site/render-status-hub.mjs" ] && [ -f "$install_kit/lib/task-migrate.sh" ]; then
    kit="$install_kit"
  elif [ -d "$cy/docs-tools" ]; then kit="$cy"
  elif [ -d "$install_kit/docs-tools" ]; then kit="$install_kit"
  else
    echo "cyberos status-page: docs-tools missing — run: bash install.sh $root" >&2
    return 2
  fi
  # soft update check when anything under .cyberos runs
  if [ -f "$kit/lib/update-check.sh" ]; then
    # shellcheck source=/dev/null
    source "$kit/lib/update-check.sh"
    _cyberos_update_check || true
  fi
  # shellcheck source=/dev/null
  source "$kit/lib/task-migrate.sh"
  if [ "$mode" = "--coverage-only" ] || [ "${CYBEROS_STATUS_COVERAGE_ONLY:-0}" = "1" ]; then
    CYBEROS_STATUS_COVERAGE_ONLY=1 PAGE_ONLY=1 _cyberos_task_migrate "$root" "$kit"
  else
    PAGE_ONLY=1 _cyberos_task_migrate "$root" "$kit"
  fi
}

if [ "${BASH_SOURCE[0]:-}" = "$0" ]; then
  _cyberos_status_page "${1:-.}" "${2:-}"
fi
