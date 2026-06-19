#!/usr/bin/env sh
# awh module gate for pre-commit. For each staged module that has a golden set,
# rerun its gate against the sealed baseline. Non-zero exit blocks the commit.
# Local fast check (1 seed); full coverage runs in CI via .github/workflows/awh-gate.yml.
set -eu
staged=$(git diff --cached --name-only | awk -F/ '/^modules\//{print $2}' | sort -u)
status=0
for m in $staged; do
  gs="modules/$m/.awh/goldenset.yaml"
  [ -f "$gs" ] || continue
  echo "[awh] gate: $m"
  base="modules/$m/.awh/eval-baseline.json"
  if [ -f "$base" ]; then
    awh eval "$gs" --base-dir . --seeds 1 --baseline "$base" --max-regression 0.0 || status=1
  else
    awh eval "$gs" --base-dir . --seeds 1 || status=1
  fi
done
exit "$status"
