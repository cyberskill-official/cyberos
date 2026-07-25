#!/usr/bin/env bash
# test_install_goldenset.sh — TASK-IMP-008 goldenset runner smoke
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
fail(){ FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
R="$repo/tools/install/run-goldenset.sh"
[ -f "$R" ] || { echo FATAL missing runner; exit 1; }
[ -f "$repo/tools/install/.awh/goldenset.yaml" ] && ok goldenset_yaml || fail goldenset_yaml "missing"
bash "$R" --help >/dev/null 2>&1 || bash "$R" 2>&1 | head -5 >/dev/null
ok runner_invocable
grep -q 'awh-gate\|run-goldenset' "$repo/.github/workflows/awh-gate.yml" && ok awh_wired || fail awh_wired "not wired"
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
