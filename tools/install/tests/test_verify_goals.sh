#!/usr/bin/env bash
# TASK-IMP-109 - standing goals. One arm per AC, plus the security arms that ARE this task.
# No model, no network. The guard arms come first: this tool executes commands read from files.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"
VG="$root/tools/install/docs-tools/verify-goals.mjs"
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
no(){ FAIL=$((FAIL+1)); echo "  FAIL $1: ${2:-}"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# a scratch repo with a tracked passing suite and a goal that cites it
mk(){
  local d="$1"; mkdir -p "$d/docs/goals" "$d/tests"
  ( cd "$d" && git init -q -b main . )
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/tests/pass.sh"; chmod +x "$d/tests/pass.sh"
  printf '#!/usr/bin/env bash\nexit 1\n' > "$d/tests/fail.sh"; chmod +x "$d/tests/fail.sh"
  ( cd "$d" && git add -A && git -c user.email=t@t -c user.name=t commit -q -m i )
}
goal(){ # goal <dir> <name> <status> <predicate...>
  local d="$1" n="$2" st="$3"; shift 3
  { echo "---"; echo "source: TASK-XX-001"; echo "born: 2026-07-17"; echo "status: $st"
    echo "last_pass: 2026-07-17"; echo "on_violation: report"
    if [ "$#" -gt 0 ]; then echo "predicates:"; for p in "$@"; do echo "  - $p"; done; fi
    echo "---"; echo "# goal"; } > "$d/docs/goals/$n.md"
}

# --- AC 1 (#1.1,#1.2): the done flip writes a goal whose predicates ARE the cited tests
t01_done_emits_goal(){
  # Structural: the emission is a workflow rule (§11c), and the artefact is the contract - the
  # same discipline TASK-IMP-104's t05 uses. A rule with no check is a comment; this is the check.
  local w="$root/modules/cuo/chief-technology-officer/workflows/ship-tasks.md"
  grep -q '^## 11c. Standing goals' "$w"                       || { no t01_done_emits_goal "no §11c section"; return; }
  grep -q 'At the .done. flip, ship-tasks MUST write .docs/goals/' "$w" || { no t01_done_emits_goal "emission is not a MUST"; return; }
  grep -q 'predicate set is the CITED TESTS, nothing invented' "$w"     || { no t01_done_emits_goal "predicates not bound to the cited tests"; return; }
  grep -q 'DETECTION ONLY' "$w"                                || { no t01_done_emits_goal "detection-only rule missing"; return; }
  grep -q 'node .cyberos/docs-tools/verify-goals.mjs' "$w"     || { no t01_done_emits_goal "runner path not named"; return; }
  # the payload must carry what the doctrine names, or the rule is live in the source and dead
  # in the machine that runs (this happened three times on 2026-07-17 before it was checked)
  local pay="$root/dist/cyberos"
  [ -f "$pay/docs-tools/verify-goals.mjs" ] || { no t01_done_emits_goal "payload lacks verify-goals.mjs - §11c names a path that does not exist"; return; }
  grep -q '^## 11c. Standing goals' "$pay/cuo/ship-tasks.md"   || { no t01_done_emits_goal "payload ship-tasks lacks §11c"; return; }
  ok t01_done_emits_goal
}
# --- AC 2 (#1.5,#1.6): a broken cited test violates, appends a ledger row, exits non-zero
t02_broken_test_violates(){
  local d="$TMP/t02"; mk "$d"; goal "$d" g satisfied tests/fail.sh
  local out; out="$(node "$VG" --repo "$d" 2>&1)"; local rc=$?
  [ "$rc" -eq 1 ] || { no t02_broken_test_violates "expected exit 1, got $rc"; return; }
  grep -q "VIOLATED" <<<"$out" || { no t02_broken_test_violates "not named violated: $out"; return; }
  grep -q "TASK-XX-001" <<<"$out" || { no t02_broken_test_violates "task not named: $out"; return; }
  grep -q "^status: violated" "$d/docs/goals/g.md" || { no t02_broken_test_violates "goal file not flipped"; return; }
  grep -q "VIOLATED" "$d/docs/goals/.ledger.tsv" || { no t02_broken_test_violates "no ledger row"; return; }
  ok t02_broken_test_violates
}
# --- AC 3 (#1.5): a passing goal refreshes last_pass and stays satisfied
t03_passing_refreshes(){
  local d="$TMP/t03"; mk "$d"; goal "$d" g satisfied tests/pass.sh
  sed -i 's/^last_pass:.*/last_pass: 2020-01-01/' "$d/docs/goals/g.md"
  node "$VG" --repo "$d" >/dev/null 2>&1 || { no t03_passing_refreshes "passing goal exited non-zero"; return; }
  grep -q "^status: satisfied" "$d/docs/goals/g.md" || { no t03_passing_refreshes "status not satisfied"; return; }
  grep -q "^last_pass: 2020-01-01" "$d/docs/goals/g.md" && { no t03_passing_refreshes "last_pass not refreshed"; return; }
  ok t03_passing_refreshes
}
# --- AC 4 (#1.3,#1.4): no runnable predicate -> a goal marked none, NOT a silent pass
t13_violated_outranks_unverifiable(){
  # The whole point of a separate code is that "cannot be checked" and "was passing, now broken"
  # stay different facts. A corpus with BOTH must exit 1: a real regression outranks a known gap.
  local d="$TMP/t13"; mk "$d"
  goal "$d" gap satisfied                                  # no predicate -> unverifiable
  printf '#!/usr/bin/env bash\nexit 1\n' > "$d/breaks.sh"; chmod +x "$d/breaks.sh"
  ( cd "$d" && git add -A >/dev/null 2>&1 && git -c user.email=t@t -c user.name=t commit -qm p >/dev/null 2>&1 ) || true
  goal "$d" broken satisfied breaks.sh                     # predicate fails -> violated
  node "$VG" --repo "$d" >/dev/null 2>&1; local rc=$?
  [ "$rc" -eq 1 ] || { no t13_violated_outranks_unverifiable "violated+unverifiable exited $rc, want 1 - a regression must not be masked by a gap"; return; }
  ok t13_violated_outranks_unverifiable
}

t04_unrunnable_named_not_faked(){
  local d="$TMP/t04"; mk "$d"; goal "$d" g satisfied
  local out; out="$(node "$VG" --repo "$d" 2>&1)"
  grep -q "no_predicate" <<<"$out" || { no t04_unrunnable_named_not_faked "absent predicate not named: $out"; return; }
  grep -q "satisfied " <<<"$out" && { no t04_unrunnable_named_not_faked "no-predicate goal reported as satisfied - it must not read as a pass"; return; }
  grep -q "NO_PREDICATE" "$d/docs/goals/.ledger.tsv" || { no t04_unrunnable_named_not_faked "no ledger row"; return; }
  # §1.4 says the absence "must not read as a pass". CI and operators read the EXIT CODE, not the
  # prose - and this arm only ever checked the prose, so the tool printed the finding and exited 0
  # for two weeks. A green tick for a task nobody checked. (Greptile P1, 2026-07-17.)
  node "$VG" --repo "$d" >/dev/null 2>&1; local rc=$?
  [ "$rc" -eq 3 ] || { no t04_unrunnable_named_not_faked "an unverifiable goal exited $rc - 3 means 'cannot be checked'; 0 would read as a pass"; return; }
  ok t04_unrunnable_named_not_faked
}
# --- AC 5 (#1.7): DETECTION ONLY. No task status changes, no code is written.
t05_detection_only(){
  local d="$TMP/t05"; mk "$d"; mkdir -p "$d/docs/tasks/x/TASK-XX-001"
  printf -- "---\nid: TASK-XX-001\nstatus: done\n---\n# t\n" > "$d/docs/tasks/x/TASK-XX-001/spec.md"
  goal "$d" g satisfied tests/fail.sh
  local before; before="$(cat "$d/docs/tasks/x/TASK-XX-001/spec.md")"
  node "$VG" --repo "$d" >/dev/null 2>&1
  [ "$before" = "$(cat "$d/docs/tasks/x/TASK-XX-001/spec.md")" ] || { no t05_detection_only "the task spec was MODIFIED by a violation - violates 1.7"; return; }
  ( cd "$d" && git diff --quiet -- tests/ ) || { no t05_detection_only "code under tests/ was modified"; return; }
  ok t05_detection_only
}
# --- AC 6 (#1.8): a hanging predicate is a violation NAMED as a timeout
t06_timeout_is_violation(){
  local d="$TMP/t06"; mk "$d"
  printf '#!/usr/bin/env bash\nsleep 30\n' > "$d/tests/hang.sh"; chmod +x "$d/tests/hang.sh"
  ( cd "$d" && git add -A && git -c user.email=t@t -c user.name=t commit -q -m h )
  goal "$d" g satisfied tests/hang.sh
  local out; out="$(node "$VG" --repo "$d" --timeout 1 2>&1)"; local rc=$?
  [ "$rc" -eq 1 ] || { no t06_timeout_is_violation "timeout did not violate (rc=$rc)"; return; }
  grep -qi "TIMED OUT" <<<"$out" || { no t06_timeout_is_violation "timeout not named: $out"; return; }
  ok t06_timeout_is_violation
}
# --- SECURITY (§3, the task's central risk): the guard, three ways. These are the arms that matter.
t07_predicate_escaping_root_refused(){
  local d="$TMP/t07"; mk "$d"; goal "$d" g satisfied ../../../etc/passwd
  local out; out="$(node "$VG" --repo "$d" 2>&1)"
  grep -q "escapes the repo root" <<<"$out" || { no t07_predicate_escaping_root_refused "traversal not refused: $out"; return; }
  grep -q "REFUSED, not executed" <<<"$out" || { no t07_predicate_escaping_root_refused "refusal not explicit: $out"; return; }
  ok t07_predicate_escaping_root_refused
}
t08_untracked_predicate_refused(){
  local d="$TMP/t08"; mk "$d"
  # on disk, executable, NOT tracked at HEAD: a crafted file must never run
  printf '#!/usr/bin/env bash\ntouch "%s/PWNED"\nexit 0\n' "$d" > "$d/tests/evil.sh"; chmod +x "$d/tests/evil.sh"
  goal "$d" g satisfied tests/evil.sh
  local out; out="$(node "$VG" --repo "$d" 2>&1)"
  [ -e "$d/PWNED" ] && { no t08_untracked_predicate_refused "AN UNTRACKED PREDICATE EXECUTED - the rung-5 defect, live"; return; }
  grep -q "not tracked at HEAD" <<<"$out" || { no t08_untracked_predicate_refused "untracked not named: $out"; return; }
  grep -q "REFUSED, not executed" <<<"$out" || { no t08_untracked_predicate_refused "refusal not explicit: $out"; return; }
  ok t08_untracked_predicate_refused
}
t09_refusal_is_a_violation_not_a_skip(){
  local d="$TMP/t09"; mk "$d"; goal "$d" g satisfied tests/nowhere.sh
  local out; out="$(node "$VG" --repo "$d" 2>&1)"; local rc=$?
  [ "$rc" -eq 1 ] || { no t09_refusal_is_a_violation_not_a_skip "a refused predicate exited 0 - a silent skip is how a guard becomes a comment"; return; }
  grep -q "resolves nowhere" <<<"$out" || { no t09_refusal_is_a_violation_not_a_skip "missing predicate not named: $out"; return; }
  ok t09_refusal_is_a_violation_not_a_skip
}
t10_retired_goal_skipped(){
  local d="$TMP/t10"; mk "$d"; goal "$d" g retired tests/fail.sh
  node "$VG" --repo "$d" >/dev/null 2>&1 || { no t10_retired_goal_skipped "a retired (quarantined) goal still ran"; return; }
  ok t10_retired_goal_skipped
}
# --- §3 (operator, review gate): the report MUST state how many done tasks have NO goal.
t11_report_states_its_coverage(){
  local d="$TMP/t11"; mk "$d"; goal "$d" g satisfied tests/pass.sh
  # two done tasks, only one enrolled
  mkdir -p "$d/docs/tasks/x/TASK-XX-001" "$d/docs/tasks/x/TASK-XX-002"
  printf -- "---\nid: TASK-XX-001\nstatus: done\n---\n# a\n" > "$d/docs/tasks/x/TASK-XX-001/spec.md"
  printf -- "---\nid: TASK-XX-002\nstatus: done\n---\n# b\n" > "$d/docs/tasks/x/TASK-XX-002/spec.md"
  local out; out="$(node "$VG" --repo "$d" 2>&1)"
  grep -q "1/2 done tasks enrolled" <<<"$out" || { no t11_report_states_its_coverage "coverage not stated: $out"; return; }
  grep -q "1 have NO goal" <<<"$out" || { no t11_report_states_its_coverage "unenrolled count not named: $out"; return; }
  # and in the machine-readable form
  node "$VG" --repo "$d" --json 2>/dev/null | grep -q '"without_goal": 1' || { no t11_report_states_its_coverage "coverage absent from --json"; return; }
  ok t11_report_states_its_coverage
}

# --- external review 2026-07-17: a RETIRED goal's task HAS a goal - it must not read as "no goal"
t12_retired_goal_is_not_missing(){
  local d="$TMP/t12"; mk "$d"; goal "$d" g retired tests/pass.sh
  mkdir -p "$d/docs/tasks/x/TASK-XX-001"
  printf -- "---\nid: TASK-XX-001\nstatus: done\n---\n# a\n" > "$d/docs/tasks/x/TASK-XX-001/spec.md"
  sed -i 's/^source: .*/source: TASK-XX-001/' "$d/docs/goals/g.md"
  local out; out="$(node "$VG" --repo "$d" 2>&1)"
  grep -q "0 have NO goal" <<<"$out" || { no t12_retired_goal_is_not_missing "a quarantined goal counted as missing: $out"; return; }
  grep -q "QUARANTINED" <<<"$out" || { no t12_retired_goal_is_not_missing "quarantine not named: $out"; return; }
  node "$VG" --repo "$d" --json 2>/dev/null | grep -q '"retired": 1' || { no t12_retired_goal_is_not_missing "retired count absent from --json"; return; }
  ok t12_retired_goal_is_not_missing
}

echo "test_verify_goals.sh (TASK-IMP-109)"
t01_done_emits_goal; t02_broken_test_violates; t03_passing_refreshes; t04_unrunnable_named_not_faked
t05_detection_only; t06_timeout_is_violation
t07_predicate_escaping_root_refused; t08_untracked_predicate_refused
t09_refusal_is_a_violation_not_a_skip; t10_retired_goal_skipped
t11_report_states_its_coverage; t12_retired_goal_is_not_missing; t13_violated_outranks_unverifiable
echo "  ---"; echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
