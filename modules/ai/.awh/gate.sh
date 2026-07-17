#!/usr/bin/env sh
# awh evidence gate for the AI module. Non-zero exit blocks the transition.
# Requires Redis at 127.0.0.1:6379 (the cache-isolation tests connect to it).
# Prefers the `awh` console script; falls back to the vendored source (tools/awh) when it
# is not installed, so the gate runs without a global install.
set -e
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
if command -v awh >/dev/null 2>&1; then
  AWH() { awh "$@"; }
else
  AWH() { PYTHONPATH="tools/awh${PYTHONPATH:+:$PYTHONPATH}" python3 -m harness.cli "$@"; }
fi
AWH eval modules/ai/.awh/goldenset.yaml \
  --base-dir . --seeds 1 \
  --baseline modules/ai/.awh/eval-baseline.json --max-regression 0.0
