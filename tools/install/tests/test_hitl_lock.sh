#!/usr/bin/env bash
# test_hitl_lock.sh - TASK-CUO-303 (t01-t07 -> AC 1-7): mechanical HITL lock on the two
# human-acceptance gate transitions. Bare reviewing->ready_to_test / testing->done refuse
# exit 8; flags unlock; route-backs stay flag-free; status_overridden kind validates;
# audit-before-action; HITL_REQUIRED dead flag gone from gates.env; ship-tasks + CHANGELOG
# document the refusal.
#
# Never touches the live BACKLOG.md or .cyberos/memory/store — every case uses a scratch
# fixture under mktemp (same discipline as the implementer's 22-case matrix).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
BM="$repo/tools/install/docs-tools/backlog-mutate.mjs"
MA="$repo/tools/install/docs-tools/memory-append.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
sha()  { shasum -a 256 "$1" | cut -d' ' -f1; }
export CYBEROS_NOW="2026-07-23T03:46:00Z"
unset CYBEROS_STORE

mk_repo() { # $1=dir $2=row_status $3=spec_status
  mkdir -p "$1/docs/tasks/improvement/TASK-GUARD-001-truth-index"
  cat > "$1/docs/tasks/BACKLOG.md" <<EOF
# CyberOS task backlog (regenerated 2026-07-09)

Totals: 1 $2

## improvement  (1 $2)

- [$2] TASK-GUARD-001-truth-index - Truth precedes index (improvement)
EOF
  printf -- '---\nid: TASK-GUARD-001\nstatus: %s\n---\n# body\n' "$3" \
    > "$1/docs/tasks/improvement/TASK-GUARD-001-truth-index/spec.md"
}
EVID="$TMP/review-note.md"
printf 'Reviewed 2026-07-23: acceptance verdict recorded here.\n' > "$EVID"

echo "test_hitl_lock.sh (TASK-CUO-303)"

t01_bare_gate_flip_refused() {                                         # AC 1
  local d="$TMP/t01" pre rc
  mk_repo "$d" reviewing ready_to_test; pre=$(sha "$d/docs/tasks/BACKLOG.md")
  node "$BM" flip TASK-GUARD-001 reviewing ready_to_test --root "$d" >"$TMP/out" 2>"$TMP/err"; rc=$?
  grep -q "STATUS-REFERENCE" "$TMP/err" && grep -qi "verdict" "$TMP/err" && [ "$rc" -eq 8 ] \
    && [ "$(sha "$d/docs/tasks/BACKLOG.md")" = "$pre" ] || { fail t01 "bare reviewing->ready_to_test rc=$rc err=$(cat "$TMP/err")"; return; }
  # testing -> done bare also refuses 8
  mk_repo "$d" testing done; pre=$(sha "$d/docs/tasks/BACKLOG.md")
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 8 ] && [ "$(sha "$d/docs/tasks/BACKLOG.md")" = "$pre" ] || { fail t01 "bare testing->done rc=$rc"; return; }
  # flagged succeeds
  mk_repo "$d" reviewing ready_to_test
  node "$BM" flip TASK-GUARD-001 reviewing ready_to_test --root "$d" \
    --verdict-by "fixture-human" --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] && grep -q '^- \[ready_to_test\]' "$d/docs/tasks/BACKLOG.md" \
    || { fail t01 "flagged flip rc=$rc err=$(cat "$TMP/err")"; return; }
  # route-back needs no flags
  mk_repo "$d" testing ready_to_implement
  node "$BM" flip TASK-GUARD-001 testing ready_to_implement --root "$d" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] && ok t01 || fail t01 "route-back rc=$rc err=$(cat "$TMP/err")"
}

t02_evidence_must_exist_nonempty() {                                   # AC 2
  local d="$TMP/t02" rc
  mk_repo "$d" testing done
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" --verdict-by human --verdict-evidence "$TMP/nope.md" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 8 ] || { fail t02 "missing evidence rc=$rc"; return; }
  : > "$TMP/empty.md"
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" --verdict-by human --verdict-evidence "$TMP/empty.md" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 8 ] || { fail t02 "empty evidence rc=$rc"; return; }
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" --verdict-by "" --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 8 ] && ok t02 || fail t02 "empty actor rc=$rc"
}

t03_refusal_precedence_six_before_eight() {                            # AC 3
  local d="$TMP/t03" rc
  # cell already moved → pre-image drift (6), not verdict gate (8)
  mk_repo "$d" ready_to_test ready_to_test
  node "$BM" flip TASK-GUARD-001 reviewing ready_to_test --root "$d" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 6 ] && grep -q "pre-image drifted" "$TMP/err" && ok t03 \
    || fail t03 "rc=$rc (want 6) err=$(cat "$TMP/err")"
}

t04_status_overridden_kind_validated() {                               # AC 4
  local S="$TMP/t04-store" rc vrc
  printf '{"actor":"human","task_id":"TASK-X-001","prior_status":"testing","new_status":"done","reason":"note.md"}' > "$TMP/p.json"
  node "$MA" append "$S" status_overridden "$TMP/p.json" >"$TMP/out" 2>"$TMP/err"; rc=$?
  node "$MA" verify "$S" >"$TMP/v" 2>&1; vrc=$?
  [ "$rc" -eq 0 ] && [ "$vrc" -eq 0 ] || { fail t04 "complete payload rc=$rc vrc=$vrc err=$(cat "$TMP/err")"; return; }
  # missing required field refuses 2 before any write
  S="$TMP/t04-miss"; printf '{"actor":"a","task_id":"TASK-X-001","prior_status":"testing","new_status":"done"}' > "$TMP/pm.json"
  node "$MA" append "$S" status_overridden "$TMP/pm.json" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 2 ] && [ ! -e "$S" ] || { fail t04 "missing field rc=$rc"; return; }
  # unknown kind still refused
  S="$TMP/t04-unk"; printf '{"x":1}' > "$TMP/pu.json"
  node "$MA" append "$S" bogus_kind "$TMP/pu.json" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 2 ] && [ ! -e "$S" ] && ok t04 || fail t04 "unknown kind rc=$rc"
}

t05_audit_before_action() {                                            # AC 5
  local d="$TMP/t05" S rc rows vrc pre
  # store present → exactly one status_overridden row
  mk_repo "$d" reviewing ready_to_test; S="$d/.cyberos/memory/store"; mkdir -p "$S"
  node "$BM" flip TASK-GUARD-001 reviewing ready_to_test --root "$d" \
    --verdict-by "Stephen Cheng (CTO)" --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  rows=$(grep -ao '"op":"status_overridden"' "$S/audit/current.binlog" 2>/dev/null | wc -l | tr -d ' ')
  node "$MA" verify "$S" >"$TMP/v" 2>&1; vrc=$?
  [ "$rc" -eq 0 ] && [ "$rows" = "1" ] && [ "$vrc" -eq 0 ] \
    || { fail t05 "store-present rc=$rc rows=$rows vrc=$vrc"; return; }
  # unwritable present store → exit 9, backlog unchanged
  d="$TMP/t05u"; mk_repo "$d" testing done; S="$d/.cyberos/memory/store"; mkdir -p "$S"; chmod 555 "$S"
  pre=$(sha "$d/docs/tasks/BACKLOG.md")
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" --verdict-by human --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 9 ] && [ "$(sha "$d/docs/tasks/BACKLOG.md")" = "$pre" ] \
    || { chmod 755 "$S"; fail t05 "unwritable rc=$rc"; return; }
  chmod 755 "$S"
  # no store → flip succeeds; stderr notes evidence is the record
  d="$TMP/t05n"; mk_repo "$d" reviewing ready_to_test
  node "$BM" flip TASK-GUARD-001 reviewing ready_to_test --root "$d" \
    --verdict-by human --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] && grep -q "evidence file is the record" "$TMP/err" && ok t05 \
    || fail t05 "no-store rc=$rc err=$(cat "$TMP/err")"
}

t06_dead_flag_removed() {                                              # AC 6
  # Prefer a fast source-level check (HITL_REQUIRED already removed from install.sh)
  # plus a scratch install that proves gates.env never carries the dead flag and keeps
  # the human-gates prose comment.
  if grep -q 'HITL_REQUIRED=' "$repo/tools/install/install.sh"; then
    fail t06 "HITL_REQUIRED= still in install.sh"; return
  fi
  grep -q 'human-acceptance gates' "$repo/tools/install/install.sh" \
    || { fail t06 "human-gates prose comment missing from install.sh"; return; }
  echo "building scratch payload for t06..."
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { fail t06 "payload build failed"; return; }
  local d="$TMP/t06inst"
  mkdir -p "$d" && ( cd "$d" && git init -q . 2>/dev/null
    CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1 )
  [ -f "$d/.cyberos/gates.env" ] || { fail t06 "gates.env missing after install"; return; }
  ! grep -q 'HITL_REQUIRED' "$d/.cyberos/gates.env" \
    && grep -qi 'human-acceptance\|HITL is required' "$d/.cyberos/gates.env" \
    && ok t06 || fail t06 "gates.env still carries HITL_REQUIRED or lost prose"
}

t07_docs_and_changelog() {                                             # AC 7
  local ship="$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md"
  local top
  grep -q -- '--verdict-by' "$ship" && grep -q -- '--verdict-evidence' "$ship" \
    || { fail t07 "ship-tasks.md missing verdict flags at HITL steps"; return; }
  # both HITL transitions named with the flags
  grep -q 'reviewing → ready_to_test' "$ship" && grep -q 'testing → done' "$ship" \
    || { fail t07 "ship-tasks.md missing HITL transition wording"; return; }
  # Scan every versioned ## […] section — top entry moves with each cut.
  top="$(awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md")"
  echo "$top" | grep -qi 'breaking' \
    && echo "$top" | grep -q 'exit code 8\|exit 8\|code 8' \
    && echo "$top" | grep -qi 'verdict\|refuse\|REFUSE' \
    && ok t07 || fail t07 "CHANGELOG versioned entry missing breaking/exit-8/refusal wording"
}

t01_bare_gate_flip_refused
t02_evidence_must_exist_nonempty
t03_refusal_precedence_six_before_eight
t04_status_overridden_kind_validated
t05_audit_before_action
t06_dead_flag_removed
t07_docs_and_changelog

echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
