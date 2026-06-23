#!/usr/bin/env sh
# awh evidence gate for the proj module. Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/proj/.awh/
cd "$ROOT"
awh eval modules/proj/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/proj/.awh/eval-baseline.json --max-regression 0.0
