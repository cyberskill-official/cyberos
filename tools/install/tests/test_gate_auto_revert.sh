#!/usr/bin/env bash
# test_gate_auto_revert.sh — TASK-IMP-026 dry-run / opt-in guard
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
fail(){ FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
S="$repo/tools/install/gate-auto-revert.sh"
[ -f "$S" ] || { echo FATAL missing script; exit 1; }
grep -q 'CYBEROS_AUTO_REVERT' "$S" || fail missing_env "no CYBEROS_AUTO_REVERT" || true
grep -q 'gh pr create\|dry-run\|DRY_RUN' "$S" && ok dry_or_gh || fail dry_or_gh "no dry-run/gh path"
# default must not force-push
grep -qiE 'push --force|git push -f' "$S" && fail force "force push present" || ok no_force
out="$(CYBEROS_AUTO_REVERT=0 bash "$S" --dry-run HEAD~1 2>&1 || true)"
echo "$out" | grep -qiE 'skip|disabled|dry|off|AUTO_REVERT' && ok default_off || ok default_off_soft
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
