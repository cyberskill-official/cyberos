#!/usr/bin/env sh
# awh evidence gate for the MEMORY module (roadmap wave 1).
# Wire this into pre-commit, CI, and the CUO testing->done step. A non-zero exit
# blocks the transition. It reruns the module's real build+test out-of-band and
# blocks on any regression against the recorded baseline.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"   # repo root, from modules/memory/.awh/
cd "$ROOT"
awh eval modules/memory/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/memory/.awh/eval-baseline.json --max-regression 0.0
