#!/usr/bin/env bash
# test_task_state_engine.sh — TASK-IMP-144: regen refuses invented edges; task-state transitions.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
BM="$repo/tools/install/docs-tools/backlog-mutate.mjs"
TS="$repo/tools/install/docs-tools/task-state.mjs"
REGEN=(python3 "$repo/scripts/migrate_improvement_to_task.py" --backlog)
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
export CYBEROS_NOW="2026-07-25T00:00:00Z"
unset CYBEROS_STORE

# Isolate regen by pointing ROOT... migrate script hardcodes ROOT from __file__.
# So run regen inside a copied mini tree is hard. Instead: test receipt + flip locally,
# and test invented-edge detection with a Python snippet mirroring the gate, plus a
# live flip+regen round-trip on a scratch that reuses the script via PYTHONPATH hack.
#
# Practical approach: use --root flips for receipts; invoke the receipt check by
# temporarily swapping docs/tasks in a subprocess with CYBEROS... Actually the migrate
# script uses Path(__file__).parents[1]. We'll copy the script's regen function logic
# into an inline test that imports by exec, OR test via flipping on real paths under TMP
# by running a minimal copy of migrate with ROOT overridden.

echo "test_task_state_engine.sh (TASK-IMP-144)"

# Build a scratch repo that mirrors the migrate script's expected layout by copying
# the migrate script with ROOT patched.
cp "$repo/scripts/migrate_improvement_to_task.py" "$TMP/migrate.py"
# Patch ROOT to TMP
python3 - <<PY
from pathlib import Path
p = Path("$TMP/migrate.py")
t = p.read_text()
t = t.replace("ROOT = Path(__file__).resolve().parents[1]", "ROOT = Path(r'$TMP')")
# drop one-time migration parsers that need old trees — keep regen_backlog + helpers
p.write_text(t)
PY
mkdir -p "$TMP/docs/tasks/improvement/TASK-ENG-001-engine"
cat > "$TMP/docs/tasks/BACKLOG.md" <<'EOF'
# CyberOS task backlog (regenerated 2026-07-09)

Totals: 1 implementing

## improvement  (1 implementing)

- [implementing] TASK-ENG-001-engine - Engine fixture (improvement)
EOF
cat > "$TMP/docs/tasks/improvement/TASK-ENG-001-engine/spec.md" <<'EOF'
---
id: TASK-ENG-001
title: Engine fixture
type: improvement
module: improvement
status: implementing
---
# body
EOF

t01_regen_refuses_invented() {
  # Edit FM to done without receipt — regen must refuse
  printf -- '---\nid: TASK-ENG-001\ntitle: Engine fixture\ntype: improvement\nmodule: improvement\nstatus: done\n---\n# body\n' \
    > "$TMP/docs/tasks/improvement/TASK-ENG-001-engine/spec.md"
  pre=$(shasum -a 256 "$TMP/docs/tasks/BACKLOG.md" | awk '{print $1}')
  python3 "$TMP/migrate.py" --backlog >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -ne 0 ] && grep -q 'invented transition' "$TMP/err" \
    && [ "$(shasum -a 256 "$TMP/docs/tasks/BACKLOG.md" | awk '{print $1}')" = "$pre" ] \
    && ok t01_regen_refuses_invented \
    || fail t01 "rc=$rc err=$(cat "$TMP/err")"
}

t02_task_state_then_regen() {
  # Restore FM to implementing, then task-state to ready_to_review (non-gate)
  printf -- '---\nid: TASK-ENG-001\ntitle: Engine fixture\ntype: improvement\nmodule: improvement\nstatus: implementing\n---\n# body\n' \
    > "$TMP/docs/tasks/improvement/TASK-ENG-001-engine/spec.md"
  # BACKLOG still implementing from original (t01 did not write)
  node "$TS" --root "$TMP" transition TASK-ENG-001 implementing ready_to_review >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] || { fail t02 "transition rc=$rc err=$(cat "$TMP/err") out=$(cat "$TMP/out")"; return; }
  grep -q '^- \[ready_to_review\]' "$TMP/docs/tasks/BACKLOG.md" || { fail t02 "backlog not updated"; return; }
  grep -q '^status: ready_to_review' "$TMP/docs/tasks/improvement/TASK-ENG-001-engine/spec.md" || { fail t02 "fm not updated"; return; }
  ls "$TMP/docs/tasks/_state/receipts"/TASK-ENG-001--implementing--ready_to_review--*.json >/dev/null 2>&1 \
    || { fail t02 "receipt missing"; return; }
  # Regen must now succeed (no invented edge — statuses agree)
  python3 "$TMP/migrate.py" --backlog >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] && ok t02_task_state_then_regen \
    || fail t02 "regen after receipt rc=$rc err=$(cat "$TMP/err")"
}

t03_gate_needs_verdict() {
  # Advance to reviewing via task-state, then try testing->done without verdict
  node "$TS" --root "$TMP" transition TASK-ENG-001 ready_to_review reviewing >"$TMP/out" 2>"$TMP/err"
  node "$TS" --root "$TMP" transition TASK-ENG-001 reviewing ready_to_test >"$TMP/out" 2>"$TMP/err"; rc=$?
  # reviewing->ready_to_test is a HITL gate — must refuse without verdict
  [ "$rc" -eq 8 ] && ok t03_gate_needs_verdict \
    || fail t03 "expected exit 8, got rc=$rc err=$(cat "$TMP/err")"
}

t01_regen_refuses_invented
t02_task_state_then_regen
t03_gate_needs_verdict

echo "task_state_engine: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
