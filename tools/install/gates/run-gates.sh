#!/usr/bin/env bash
# run-gates.sh - the machine-gate floor for a task. Reads .cyberos/gates.env and runs
# the target repo's own build / lint / test / coverage, plus optional caf / awh when
# enabled, plus the memory doctor when a BRAIN store is installed. GREEN here is
# necessary but NOT sufficient: the two human-acceptance gates (review acceptance,
# final acceptance) are always still required and are never run here.
# Exit codes: 0 green (or acknowledged-empty), 1 a gate ran and failed, 2 missing
# gates.env / malformed config.yaml, 3 empty floor (nothing configured - fail closed).
#
# TASK-IMP-011: on RED (exit 1 or 3) emit structured failure taxonomy —
#   .cyberos/last-gate-failure.json  +  one GATE_FAILURE_JSON:{...} stdout line.
# Classes: build|lint|test|coverage|doctor|caf|awh|empty-floor|other
set -uo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
env_file="$root/.cyberos/gates.env"
if [ ! -f "$env_file" ]; then
  echo "run-gates: missing $env_file - run bash .cyberos/install.sh first" >&2
  exit 2
fi
# shellcheck disable=SC1090
. "$env_file"

# Soft update check on every gates run (anything under .cyberos triggers this)
if [ -f "$root/.cyberos/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$root/.cyberos/lib/update-check.sh"
  _cyberos_update_check || true
fi

# --- .cyberos/config.yaml layer (TASK-CUO-207) ---------------------------------
cfg_file="$root/.cyberos/config.yaml"
cfg_get() {
  [ -f "$cfg_file" ] || return 0
  awk -v top="$1" -v child="${2:-}" '
    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
    /^[a-z_]+:/ { cur=$1; sub(/:$/,"",cur)
      if (child == "" && cur == top) { line=$0; sub(/^[a-z_]+:[[:space:]]*/,"",line); print line; exit } next }
    /^  [a-z_]+:/ { if (child != "" && cur == top) { k=$1; sub(/:$/,"",k)
      if (k == child) { line=$0; sub(/^  [a-z_]+:[[:space:]]*/,"",line); print line; exit } } }
  ' "$cfg_file" | sed -e 's/[[:space:]]*#.*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//"
}
if [ -f "$cfg_file" ]; then
  bad="$(grep -nEv '^[[:space:]]*#|^[[:space:]]*$|^[a-z_]+:([[:space:]].*)?$|^  [a-z_]+:[[:space:]].*$' "$cfg_file" | head -1)"
  if [ -n "$bad" ]; then
    echo "run-gates: MALFORMED $cfg_file at line ${bad%%:*}: ${bad#*:}" >&2
    echo "run-gates: no gate was run (config must parse fully or not exist)" >&2
    exit 2
  fi
fi

CFG_BUILD="$(cfg_get gates build)"; CFG_LINT="$(cfg_get gates lint)"
CFG_TEST="$(cfg_get gates test)";  CFG_COVERAGE="$(cfg_get gates coverage)"
thr="$(cfg_get coverage_threshold)"
export CYBEROS_COVERAGE_THRESHOLD="${thr:-${COVERAGE_MIN:-90}}"

# --- TASK-IMP-011 failure taxonomy --------------------------------------------
FAILURE_FILE="$root/.cyberos/last-gate-failure.json"
# Accumulate failure records as lines: class|gate|cmd|source  (cmd/source may contain spaces — use SOH)
_fail_records=""
gate_class() {
  case "$1" in
    build|lint|test|coverage|doctor|caf|awh) printf '%s' "$1" ;;
    empty-floor) printf 'empty-floor' ;;
    *) printf 'other' ;;
  esac
}
json_escape() {
  # minimal JSON string escape for gate cmds
  printf '%s' "$1" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()), end="")' 2>/dev/null \
    || printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g')"
}
record_failure() {
  local class="$1" gname="$2" cmd="$3" src="$4"
  _fail_records="${_fail_records}${class}"$'\t'"${gname}"$'\t'"${cmd}"$'\t'"${src}"$'\n'
}
emit_failure_artifact() {
  local exit_code="$1"
  local tmp failures_json first=1
  tmp="$(mktemp)"
  failures_json="["
  while IFS=$'\t' read -r class gname cmd src; do
    [ -n "${class:-}" ] || continue
    [ "$first" -eq 1 ] || failures_json+=","
    first=0
    failures_json+=$(printf '{"class":%s,"gate":%s,"cmd":%s,"source":%s}' \
      "$(json_escape "$class")" "$(json_escape "$gname")" \
      "$(json_escape "$cmd")" "$(json_escape "$src")")
  done <<< "$_fail_records"
  failures_json+="]"
  printf '{"schema":"gate-failure@1","exit_code":%s,"failures":%s}\n' "$exit_code" "$failures_json" > "$tmp"
  mkdir -p "$root/.cyberos"
  mv "$tmp" "$FAILURE_FILE"
  # single-line miner hook
  printf 'GATE_FAILURE_JSON:%s\n' "$(tr -d '\n' < "$FAILURE_FILE")"
}
clear_failure_artifact() {
  rm -f "$FAILURE_FILE"
}

fail=0
gate() {
  desc="$1"; auto="$2"; cfg="$3"; stack="$4"
  if [ -n "$cfg" ]; then cmd="$cfg"; src="config"
  elif [ -n "$auto" ]; then cmd="$auto"; src="autodetect:${stack:-env}"
  else cmd=""; src="absent"; fi
  echo "gate $desc: ${cmd:-} (source: $src)"
  if [ -z "$cmd" ]; then printf 'SKIP  %-9s (no command configured)\n' "$desc"; return; fi
  printf 'GATE  %-9s %s\n' "$desc" "$cmd"
  if ( cd "$root" && eval "$cmd" ); then
    printf 'PASS  %s\n' "$desc"
  else
    printf 'FAIL  %s\n' "$desc"
    record_failure "$(gate_class "$desc")" "$desc" "$cmd" "$src"
    fail=1
  fi
}

gate build    "${BUILD_CMD:-}"    "$CFG_BUILD"    "${SRC_BUILD:-}"
gate lint     "${LINT_CMD:-}"     "$CFG_LINT"     "${SRC_LINT:-}"
gate test     "${TEST_CMD:-}"     "$CFG_TEST"     "${SRC_TEST:-}"
gate coverage "${COVERAGE_CMD:-}" "$CFG_COVERAGE" "${SRC_COVERAGE:-}"
[ "${CAF_ENABLED:-false}" = "true" ] && gate caf "${CAF_CMD:-}" "" "gates.env"
[ "${AWH_ENABLED:-false}" = "true" ] && gate awh "${AWH_CMD:-}" "" "gates.env"

# --- doctor gate (TASK-MEMORY-303 #1.6) -----------------------------------------
# BRAIN health joins the machine floor when memory is installed. Presence-gated twice:
# the live store must exist AND the cyberos memory module must be importable (an import
# probe, never a bare $PATH name - the TASK-IMP-130 collision lesson applied to gating).
# Repos without memory see exactly one SKIP provenance line and no behavior change.
# Doctor FAIL maps to the ordinary gate RED (exit 1); doctor does NOT count toward the
# empty-floor check below (it is additive, like caf/awh - never a substitute for the floor).
if [ -d "$root/.cyberos/memory/store" ]; then
  if command -v python3 >/dev/null 2>&1 && ( cd "$root" && python3 -c "import cyberos.core" >/dev/null 2>&1 ); then
    gate doctor "python3 -m cyberos doctor" "" "memory"
  else
    printf 'SKIP  %-9s (memory store present, cyberos memory CLI not importable - source: memory)\n' doctor
  fi
else
  printf 'SKIP  %-9s (no memory store at .cyberos/memory/store - source: memory)\n' doctor
fi

echo "----------------------------------------------------------------------"
if [ "$fail" -ne 0 ]; then
  echo "GATES: RED - route the task back to ready_to_implement and fix; do not advance."
  emit_failure_artifact 1
  exit 1
fi
# --- fail-closed floor (TASK-CUO-302 #1.1-#1.3) ---------------------------------
# An all-empty floor used to print an advisory line and fall through to GREEN - a green
# that verified nothing. Now it is RED with its own exit code: 3 is deliberately distinct
# from 1 (a configured gate failed) and 2 (missing gates.env / malformed config.yaml), so
# automation can tell "gates ran and failed" from "nothing was configured". This check
# runs only after the config.yaml layer parsed (malformed config already exited 2 above).
# Escape hatch: CYBEROS_ALLOW_EMPTY_GATES set to the LITERAL value 1 (any other value -
# true/yes/0 - behaves as unset; an acknowledgment must be exact to be honest).
if [ -z "${BUILD_CMD:-}$CFG_BUILD${LINT_CMD:-}$CFG_LINT${TEST_CMD:-}$CFG_TEST${COVERAGE_CMD:-}$CFG_COVERAGE" ]; then
  if [ "${CYBEROS_ALLOW_EMPTY_GATES:-}" = "1" ]; then
    echo "GATES: EMPTY-ACKNOWLEDGED - the floor ran nothing (build/lint/test/coverage all empty); CYBEROS_ALLOW_EMPTY_GATES=1 accepted that for THIS run only."
    clear_failure_artifact
  else
    echo "GATES: RED - EMPTY FLOOR: zero gate commands are configured, so nothing was verified and this run cannot be green."
    echo "  Fix durably: set gates.build / gates.lint / gates.test / gates.coverage in .cyberos/config.yaml,"
    echo "  or re-run the install (bash .cyberos/install.sh) so autodetect can seed commands from your repo."
    echo "  Genuinely nothing to run (docs-only repo)? Acknowledge it per run: CYBEROS_ALLOW_EMPTY_GATES=1"
    record_failure "empty-floor" "floor" "" "fail-closed"
    emit_failure_artifact 3
    exit 3
  fi
else
  echo "GATES: GREEN (machine gates only)."
  clear_failure_artifact
fi
echo "HITL still required: a human records the review verdict and the final acceptance."
echo "The agent must NOT set the task to done itself."

# Status page via internal lib (not a user-facing command)
if command -v node >/dev/null 2>&1 && [ -f "$root/.cyberos/lib/status-page.sh" ]; then
  if bash "$root/.cyberos/lib/status-page.sh" "$root" >/dev/null 2>&1; then
    echo "status: docs/status/ regenerated"
  fi
fi
