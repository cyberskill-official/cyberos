#!/usr/bin/env bash
# run-gates.sh - the machine-gate floor for an FR. Reads .cyberos/gates.env and runs
# the target repo's own build / lint / test / coverage, plus optional caf / awh when
# enabled. GREEN here is necessary but NOT sufficient: the two human-acceptance gates
# (review acceptance, final acceptance) are always still required and are never run here.
set -uo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
env_file="$root/.cyberos/gates.env"
if [ ! -f "$env_file" ]; then
  echo "run-gates: missing $env_file - run .cyberos/init.sh first" >&2
  exit 2
fi
# shellcheck disable=SC1090
. "$env_file"

# --- .cyberos/config.yaml layer (FR-CUO-207) ---------------------------------
# Dependency-free YAML subset: top-level `key: value` + one nesting level under
# `gates:`. Comments/blank lines ignored. Anything else = malformed -> fail loudly
# citing the line, run NO gate (never half-apply).
cfg_file="$root/.cyberos/config.yaml"
cfg_get() { # cfg_get <top> [child] -> value (empty if unset/commented)
  [ -f "$cfg_file" ] || return 0
  awk -v top="$1" -v child="${2:-}" '
    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
    /^[a-z_]+:/ { cur=$1; sub(/:$/,"",cur)
      if (child == "" && cur == top) { line=$0; sub(/^[a-z_]+:[[:space:]]*/,"",line); print line; exit } next }
    /^  [a-z_]+:/ { if (child != "" && cur == top) { k=$1; sub(/:$/,"",k)
      if (k == child) { line=$0; sub(/^  [a-z_]+:[[:space:]]*/,"",line); print line; exit } } }
  ' "$cfg_file" | sed -e 's/[[:space:]]*#.*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'\$//"
}
if [ -f "$cfg_file" ]; then
  bad="$(grep -nEv '^[[:space:]]*#|^[[:space:]]*$|^[a-z_]+:([[:space:]].*)?$|^  [a-z_]+:[[:space:]].*$' "$cfg_file" | head -1)"
  if [ -n "$bad" ]; then
    echo "run-gates: MALFORMED $cfg_file at line ${bad%%:*}: ${bad#*:}" >&2
    echo "run-gates: no gate was run (config must parse fully or not exist)" >&2
    exit 2
  fi
  for k in gates coverage_threshold fr_template profile; do :; done
  # warn on unknown top-level keys (never fail)
  grep -E '^[a-z_]+:' "$cfg_file" | sed 's/:.*//' | while read -r k; do
    case "$k" in gates|coverage_threshold|fr_template|profile) ;; *) echo "run-gates: WARN unknown config key: $k" >&2 ;; esac
  done
fi

CFG_BUILD="$(cfg_get gates build)"; CFG_LINT="$(cfg_get gates lint)"
CFG_TEST="$(cfg_get gates test)";  CFG_COVERAGE="$(cfg_get gates coverage)"
thr="$(cfg_get coverage_threshold)"
export CYBEROS_COVERAGE_THRESHOLD="${thr:-${COVERAGE_MIN:-90}}"

fail=0
gate() { # gate <name> <autodetected-cmd> <config-cmd> <autodetect-stack>
  desc="$1"; auto="$2"; cfg="$3"; stack="$4"
  if [ -n "$cfg" ]; then cmd="$cfg"; src="config"
  elif [ -n "$auto" ]; then cmd="$auto"; src="autodetect:${stack:-env}"
  else cmd=""; src="absent"; fi
  echo "gate $desc: ${cmd:-} (source: $src)"
  if [ -z "$cmd" ]; then printf 'SKIP  %-9s (no command configured)\n' "$desc"; return; fi
  printf 'GATE  %-9s %s\n' "$desc" "$cmd"
  if ( cd "$root" && eval "$cmd" ); then printf 'PASS  %s\n' "$desc"; else printf 'FAIL  %s\n' "$desc"; fail=1; fi
}

gate build    "${BUILD_CMD:-}"    "$CFG_BUILD"    "${SRC_BUILD:-}"
gate lint     "${LINT_CMD:-}"     "$CFG_LINT"     "${SRC_LINT:-}"
gate test     "${TEST_CMD:-}"     "$CFG_TEST"     "${SRC_TEST:-}"
gate coverage "${COVERAGE_CMD:-}" "$CFG_COVERAGE" "${SRC_COVERAGE:-}"
[ "${CAF_ENABLED:-false}" = "true" ] && gate caf "${CAF_CMD:-}" "" "gates.env"
[ "${AWH_ENABLED:-false}" = "true" ] && gate awh "${AWH_CMD:-}" "" "gates.env"

echo "----------------------------------------------------------------------"
if [ "$fail" -ne 0 ]; then
  echo "GATES: RED - route the FR back to ready_to_implement and fix; do not advance."
  exit 1
fi
if [ -z "${BUILD_CMD:-}$CFG_BUILD${LINT_CMD:-}$CFG_LINT${TEST_CMD:-}$CFG_TEST${COVERAGE_CMD:-}$CFG_COVERAGE" ]; then
  echo "GATES: floor only - nothing detected and no overrides. Set commands in .cyberos/config.yaml (gates.build/lint/test/coverage)."
fi
echo "GATES: GREEN (machine gates only)."
echo "HITL still required: a human records the review verdict and the final acceptance."
echo "The agent must NOT set the FR to done itself."
