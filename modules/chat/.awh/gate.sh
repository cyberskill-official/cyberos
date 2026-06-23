#!/usr/bin/env sh
# awh evidence gate for the chat module. Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/chat/.awh/
cd "$ROOT"
awh eval modules/chat/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/chat/.awh/eval-baseline.json --max-regression 0.0
