#!/usr/bin/env bash
# test_status_dom.sh - TASK-DOCS-015 wrapper. Ensures jsdom (tests-only) then runs the DOM suite.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if [ ! -d "$here/node_modules/jsdom" ]; then
  (cd "$here" && npm install --no-fund --no-audit --silent) || {
    echo "  FAIL could not install jsdom for docs-site DOM tests"
    exit 1
  }
fi
node "$here/test_status_dom.mjs"
