#!/usr/bin/env bash
# run-gates.sh - the machine-gate floor for an FR. Reads .cyberos/fr.gates.env and runs
# the target repo's own build / lint / test / coverage, plus optional caf / awh when
# enabled. GREEN here is necessary but NOT sufficient: the two human-acceptance gates
# (review acceptance, final acceptance) are always still required and are never run here.
set -uo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
env_file="$root/.cyberos/fr.gates.env"
if [ ! -f "$env_file" ]; then
  echo "run-gates: missing $env_file - run .cyberos/fr-pack/init.sh first" >&2
  exit 2
fi
# shellcheck disable=SC1090
. "$env_file"

fail=0
gate() {
  desc="$1"; cmd="$2"
  if [ -z "$cmd" ]; then printf 'SKIP  %-9s (no command configured)\n' "$desc"; return; fi
  printf 'GATE  %-9s %s\n' "$desc" "$cmd"
  if ( cd "$root" && eval "$cmd" ); then printf 'PASS  %s\n' "$desc"; else printf 'FAIL  %s\n' "$desc"; fail=1; fi
}

gate build    "${BUILD_CMD:-}"
gate lint     "${LINT_CMD:-}"
gate test     "${TEST_CMD:-}"
gate coverage "${COVERAGE_CMD:-}"
[ "${CAF_ENABLED:-false}" = "true" ] && gate caf "${CAF_CMD:-}"
[ "${AWH_ENABLED:-false}" = "true" ] && gate awh "${AWH_CMD:-}"

echo "----------------------------------------------------------------------"
if [ "$fail" -ne 0 ]; then
  echo "GATES: RED - route the FR back to ready_to_implement and fix; do not advance."
  exit 1
fi
echo "GATES: GREEN (machine gates only)."
echo "HITL still required: a human records the review verdict and the final acceptance."
echo "The agent must NOT set the FR to done itself."
