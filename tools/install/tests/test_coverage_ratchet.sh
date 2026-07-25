#!/usr/bin/env bash
# test_coverage_ratchet.sh — TASK-IMP-012 install-suite coverage ratchet.
#
#   t01  fixture measurement matches expected covered/total/pct
#   t02  pct below baseline → exit 1
#   t03  pct >= baseline → exit 0
#   t04  missing baseline → exit 2
#   t05  --write-baseline seeds baseline
#   t06  payload build vendors coverage-ratchet.mjs
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
CR="$repo/tools/install/docs-tools/coverage-ratchet.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# Minimal fake install tree for measurement
fx="$TMP/fx"
mkdir -p "$fx/tools/install/gates" "$fx/tools/install/docs-tools" "$fx/tools/install/tests"
printf '#!/bin/bash\n' > "$fx/tools/install/alpha.sh"
printf '#!/bin/bash\n' > "$fx/tools/install/beta.sh"
printf '#!/bin/bash\n' > "$fx/tools/install/gates/run-gates.sh"
printf '//x\n' > "$fx/tools/install/docs-tools/helper.mjs"
# only alpha + run-gates referenced
printf '# mentions alpha.sh and run-gates.sh\n' > "$fx/tools/install/tests/test_alpha.sh"
# 4 scripts, 2 covered → 50.0%

t01_fixture_measurement() {
  local out
  out="$(node "$CR" --repo "$fx" --write-baseline --baseline "$TMP/b1.json" --json 2>/dev/null)"
  echo "$out" | grep -q '"pct":50' || echo "$out" | grep -q '"pct": 50' \
    || { fail t01 "pct want 50: $out"; return; }
  echo "$out" | grep -q '"covered":2' || echo "$out" | grep -q '"covered": 2' \
    || { fail t01 "covered want 2: $out"; return; }
  echo "$out" | grep -q '"total":4' || echo "$out" | grep -q '"total": 4' \
    || { fail t01 "total want 4: $out"; return; }
  ok t01_fixture_measurement
}

t02_regression_exits_1() {
  printf '{"schema":"coverage-ratchet@1","pct":90,"covered":9,"total":10}\n' > "$TMP/high.json"
  node "$CR" --repo "$fx" --baseline "$TMP/high.json" >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 1 ] || { fail t02 "rc=$rc want 1"; return; }
  ok t02_regression_exits_1
}

t03_at_or_above_passes() {
  printf '{"schema":"coverage-ratchet@1","pct":50,"covered":2,"total":4}\n' > "$TMP/eq.json"
  node "$CR" --repo "$fx" --baseline "$TMP/eq.json" >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 0 ] || { fail t03 "equal rc=$rc want 0"; return; }
  printf '{"schema":"coverage-ratchet@1","pct":40,"covered":1,"total":4}\n' > "$TMP/low.json"
  node "$CR" --repo "$fx" --baseline "$TMP/low.json" >/dev/null 2>&1
  rc=$?
  [ "$rc" -eq 0 ] || { fail t03 "above rc=$rc want 0"; return; }
  ok t03_at_or_above_passes
}

t04_missing_baseline() {
  node "$CR" --repo "$fx" --baseline "$TMP/no-such-baseline.json" >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 2 ] || { fail t04 "rc=$rc want 2"; return; }
  ok t04_missing_baseline
}

t05_write_baseline() {
  rm -f "$TMP/seed.json"
  node "$CR" --repo "$fx" --baseline "$TMP/seed.json" --write-baseline >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 0 ] || { fail t05 "rc=$rc want 0"; return; }
  [ -f "$TMP/seed.json" ] || { fail t05 "baseline not written"; return; }
  grep -q '"pct": 50' "$TMP/seed.json" || grep -q '"pct":50' "$TMP/seed.json" \
    || { fail t05 "bad seed: $(cat "$TMP/seed.json")"; return; }
  ok t05_write_baseline
}

t06_payload_vendors() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { fail t06 "build failed"; return; }
  [ -f "$TMP/payload/docs-tools/coverage-ratchet.mjs" ] || { fail t06 "not vendored"; return; }
  ok t06_payload_vendors
}

t01_fixture_measurement
t02_regression_exits_1
t03_at_or_above_passes
t04_missing_baseline
t05_write_baseline
t06_payload_vendors

echo "----"
echo "coverage-ratchet: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
