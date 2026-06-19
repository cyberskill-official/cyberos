#!/usr/bin/env sh
# awh evidence gate for the SKILL module (roadmap wave 2). Deterministic floor only
# (Rust broker + trigger-tests/baseline validators); LLM parity is a later Stage-2 add.
# Non-zero exit blocks the transition.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/skill/.awh/
cd "$ROOT"
awh eval modules/skill/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/skill/.awh/eval-baseline.json --max-regression 0.0
