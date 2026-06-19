#!/usr/bin/env sh
# awh evidence gate for the auth module. Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/auth/.awh/
cd "$ROOT"
awh eval modules/auth/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/auth/.awh/eval-baseline.json --max-regression 0.0
