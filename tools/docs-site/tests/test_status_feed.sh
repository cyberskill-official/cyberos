#!/usr/bin/env bash
# test_status_feed.sh - TASK-DOCS-010 / TASK-DOCS-011 wrapper for the status-feed@1 unit suite.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
node "$here/test_status_feed.mjs"
