#!/usr/bin/env bash
# caf_precommit_check.sh - structural fail-closed check for the caf-gate (no toolchain, sandbox-safe).
#
# Every module with an awh goldenset (i.e. a gated module) MUST also declare a CAF audit-profile.yaml,
# so ship-tasks step 29 has a target-health command to run. This hook proves the gate is
# DECLARED for every gated module; it does NOT run builds or tests (that is scripts/caf_gate.sh on a
# build machine). Wire into .git/hooks/pre-commit (or a pre-commit config) next to the awh check.
#
# Exit 0 = every gated module declares an audit-profile.yaml; 1 = a gap (commit should fail closed).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
miss=0
found=0
for gs in "$ROOT"/modules/*/.awh/goldenset.yaml; do
  [ -e "$gs" ] || continue
  found=$((found+1))
  m="$(basename "$(dirname "$(dirname "$gs")")")"
  if [ ! -f "$ROOT/modules/$m/audit-profile.yaml" ]; then
    echo "FAIL-CLOSED: gated module '$m' has .awh/goldenset.yaml but no modules/$m/audit-profile.yaml (caf target health undeclared)"
    miss=1
  fi
done
[ "$found" -eq 0 ] && { echo "caf-precommit: no gated modules found (nothing to check)"; exit 0; }
[ "$miss" -eq 0 ] && echo "caf-precommit: OK - all $found gated module(s) declare an audit-profile.yaml"
exit "$miss"
