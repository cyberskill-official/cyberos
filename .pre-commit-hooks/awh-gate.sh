#!/usr/bin/env sh
# awh module gate for pre-commit. For each staged module that has a golden set,
# rerun its gate against the SEALED baseline. A missing baseline FAILS CLOSED: a golden
# set with no baseline is un-gated (awh eval without --baseline always exits 0), so we
# refuse the commit rather than let it pass. Local fast check (1 seed); full coverage
# runs in CI via .github/workflows/awh-gate.yml.
set -eu

# Prefer the `awh` console script; fall back to the vendored source (tools/awh) so the gate
# still runs when the standalone package is not pip-installed. CyberOS runs awh from source,
# so the hook must not depend on a globally installed binary.
if command -v awh >/dev/null 2>&1; then
  AWH() { awh "$@"; }
else
  AWH() { PYTHONPATH="tools/awh${PYTHONPATH:+:$PYTHONPATH}" python3 -m harness.cli "$@"; }
fi

staged=$(git diff --cached --name-only | awk -F/ '/^modules\//{print $2}' | sort -u)
status=0
for m in $staged; do
  gs="modules/$m/.awh/goldenset.yaml"
  [ -f "$gs" ] || continue
  base="modules/$m/.awh/eval-baseline.json"
  if [ ! -f "$base" ]; then
    echo "[awh] $m: golden set present but no baseline ($base) - failing closed."
    echo "[awh]   capture it: awh eval $gs --base-dir . --seeds 1 --out $base"
    status=1
    continue
  fi
  echo "[awh] gate: $m"
  awh eval "$gs" --base-dir . --seeds 1 --baseline "$base" --max-regression 0.0 || status=1
done
exit "$status"
