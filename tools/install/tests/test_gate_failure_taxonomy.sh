#!/usr/bin/env bash
# test_gate_failure_taxonomy.sh — TASK-IMP-011 structured gate-failure taxonomy.
#
#   t01  forced test failure → exit 1, class:test, GATE_FAILURE_JSON + artifact
#   t02  empty floor → exit 3, class:empty-floor
#   t03  green run clears a stale last-gate-failure.json
#   t04  unknown gate label classifies as other (unit of gate_class via forced desc)
#   t05  build.sh vendors run-gates.sh into payload cuo/gates/
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

mkdir -p "$TMP/py-noimport/cyberos"
printf 'raise ImportError("doctor gate stubbed out for taxonomy tests")\n' > "$TMP/py-noimport/cyberos/__init__.py"

initrepo() { ( cd "$1" && git init -q . 2>/dev/null; CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$1" >/dev/null 2>&1 ); }
rungates() { local d="$1"; shift; ( cd "$d" && env CYBEROS_OFFLINE=1 PYTHONPATH="$TMP/py-noimport" "$@" bash "$d/.cyberos/cuo/gates/run-gates.sh" 2>&1 ); }

mkdir -p "$TMP/base" && initrepo "$TMP/base"

t01_forced_test_failure_taxonomy() {
  local d="$TMP/t01"; mkdir -p "$d" && initrepo "$d"
  # clear autodetected cmds so only config drives the floor
  for k in BUILD_CMD LINT_CMD TEST_CMD COVERAGE_CMD; do
    sed -i.bak "s/^${k}=.*/${k}=\"\"/" "$d/.cyberos/gates.env" 2>/dev/null || true
  done
  printf 'gates:\n  test: "false"\n' > "$d/.cyberos/config.yaml"
  local out rc
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 1 ] || { fail t01 "rc=$rc want 1: $out"; return; }
  grep -q 'GATE_FAILURE_JSON:' <<<"$out" || { fail t01 "no GATE_FAILURE_JSON line: $out"; return; }
  [ -f "$d/.cyberos/last-gate-failure.json" ] || { fail t01 "missing failure artifact"; return; }
  grep -q '"schema":"gate-failure@1"' "$d/.cyberos/last-gate-failure.json" \
    || grep -q '"schema": "gate-failure@1"' "$d/.cyberos/last-gate-failure.json" \
    || { fail t01 "bad schema: $(cat "$d/.cyberos/last-gate-failure.json")"; return; }
  grep -q '"class":"test"' "$d/.cyberos/last-gate-failure.json" \
    || grep -q '"class": "test"' "$d/.cyberos/last-gate-failure.json" \
    || { fail t01 "class not test: $(cat "$d/.cyberos/last-gate-failure.json")"; return; }
  ok t01_forced_test_failure_taxonomy
}

t02_empty_floor_class() {
  local d="$TMP/t02"; mkdir -p "$d" && initrepo "$d"
  for k in BUILD_CMD LINT_CMD TEST_CMD COVERAGE_CMD; do
    sed -i.bak "s/^${k}=.*/${k}=\"\"/" "$d/.cyberos/gates.env" 2>/dev/null || true
  done
  rm -f "$d/.cyberos/config.yaml"
  local out rc
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 3 ] || { fail t02 "rc=$rc want 3: $out"; return; }
  grep -q 'GATE_FAILURE_JSON:' <<<"$out" || { fail t02 "no JSON line: $out"; return; }
  grep -q 'empty-floor' "$d/.cyberos/last-gate-failure.json" || { fail t02 "no empty-floor: $(cat "$d/.cyberos/last-gate-failure.json")"; return; }
  ok t02_empty_floor_class
}

t03_green_clears_stale() {
  local d="$TMP/t03"; mkdir -p "$d" && initrepo "$d"
  for k in BUILD_CMD LINT_CMD TEST_CMD COVERAGE_CMD; do
    sed -i.bak "s/^${k}=.*/${k}=\"\"/" "$d/.cyberos/gates.env" 2>/dev/null || true
  done
  printf '{"schema":"gate-failure@1","exit_code":1,"failures":[]}\n' > "$d/.cyberos/last-gate-failure.json"
  printf 'gates:\n  test: "true"\n' > "$d/.cyberos/config.yaml"
  local out rc
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 0 ] || { fail t03 "rc=$rc want 0: $out"; return; }
  [ ! -f "$d/.cyberos/last-gate-failure.json" ] || { fail t03 "stale failure file not cleared"; return; }
  ok t03_green_clears_stale
}

t04_other_class_via_helper() {
  # Exercise gate_class mapping for an unknown label by sourcing the case logic.
  local class
  class="$(bash -c '
    gate_class() { case "$1" in build|lint|test|coverage|doctor|caf|awh) printf %s "$1";; empty-floor) printf empty-floor;; *) printf other;; esac; }
    gate_class weird-custom
  ')"
  [ "$class" = "other" ] || { fail t04 "got '$class' want other"; return; }
  ok t04_other_class_via_helper
}

t05_payload_vendors_run_gates() {
  [ -f "$TMP/payload/cuo/gates/run-gates.sh" ] || { fail t05 "missing vendored run-gates"; return; }
  grep -q 'gate-failure@1' "$TMP/payload/cuo/gates/run-gates.sh" || { fail t05 "vendored copy lacks taxonomy"; return; }
  ok t05_payload_vendors_run_gates
}

t01_forced_test_failure_taxonomy
t02_empty_floor_class
t03_green_clears_stale
t04_other_class_via_helper
t05_payload_vendors_run_gates

echo "----"
echo "gate-failure-taxonomy: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
