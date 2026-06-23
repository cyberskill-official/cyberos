#!/usr/bin/env sh
# awh evidence gate for the email module. Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
awh eval modules/email/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/email/.awh/eval-baseline.json --max-regression 0.0
