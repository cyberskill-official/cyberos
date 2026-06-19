#!/usr/bin/env sh
# awh evidence gate for the cuo module. Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/cuo/.awh/
cd "$ROOT"
awh eval modules/cuo/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/cuo/.awh/eval-baseline.json --max-regression 0.0
