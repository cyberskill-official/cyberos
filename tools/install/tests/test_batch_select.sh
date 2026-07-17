#!/usr/bin/env bash
# TASK-IMP-104-evidence / ship-tasks v2.8.0 - batch-select.mjs.
#
# WHY THIS SUITE EXISTS: batch-select shipped as a MANDATORY workflow step (§11a, before step 1)
# with NO test suite, while its sibling verify-goals.mjs got eleven arms. An external review
# caught an argument-parsing defect that made the DOCUMENTED invocation
# (`node .cyberos/docs-tools/batch-select.mjs --json`) exit 3 every single time - and named the
# absence of this suite as the reason it went unnoticed. t01 is that arm.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"
BS="$root/tools/install/docs-tools/batch-select.mjs"
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
no(){ FAIL=$((FAIL+1)); echo "  FAIL $1: ${2:-}"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

spec(){ # spec <dir> <id> <status> <priority> <service> <deps> [files...]
  local d="$1" id="$2" st="$3" pr="$4" svc="$5" deps="$6"; shift 6
  mkdir -p "$d/docs/tasks/m/$id"
  { echo "---"; echo "id: $id"; echo "status: $st"; echo "priority: $pr"; echo "service: $svc"
    echo "depends_on: [$deps]"; echo "new_files:"; echo "  - (none)"; echo "modified_files:"
    for f in "$@"; do echo "  - $f"; done
    echo "---"; echo "# $id"; } > "$d/docs/tasks/m/$id/spec.md"
}

# --- t01: THE DOCUMENTED INVOCATION. The arm whose absence let the defect ship.
t01_documented_invocation_works(){
  local d="$TMP/t01"; spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js
  local out rc
  out="$(cd "$d" && node "$BS" --json 2>&1)"; rc=$?      # exactly what ship-tasks §11a mandates
  [ "$rc" -eq 0 ] || { no t01_documented_invocation_works "documented invocation exited $rc: $out"; return; }
  grep -q '"batch"' <<<"$out" || { no t01_documented_invocation_works "no batch in output: $out"; return; }
  grep -q "no docs/tasks" <<<"$out" && { no t01_documented_invocation_works "read --json as the repo root - the shipped defect, back"; return; }
  ok t01_documented_invocation_works
}
# --- t02: flags in any order, and --repo still works
t02_flag_order_independent(){
  local d="$TMP/t02"; spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js
  node "$BS" --repo "$d" --json  >/dev/null 2>&1 || { no t02_flag_order_independent "--repo --json failed"; return; }
  node "$BS" --json --repo "$d"  >/dev/null 2>&1 || { no t02_flag_order_independent "--json --repo failed"; return; }
  node "$BS" --repo "$d"         >/dev/null 2>&1 || { no t02_flag_order_independent "--repo alone failed"; return; }
  ok t02_flag_order_independent
}
# --- t03: eligibility = ready_to_implement AND every depends_on done
t03_eligibility(){
  local d="$TMP/t03"
  spec "$d" TASK-A-001 done               p1 svc/a "" src/a.js
  spec "$d" TASK-A-002 ready_to_implement p1 svc/b "TASK-A-001" src/b.js    # dep done -> eligible
  spec "$d" TASK-A-003 ready_to_implement p1 svc/c "TASK-A-009" src/c.js    # dep MISSING -> not
  spec "$d" TASK-A-004 draft              p1 svc/d "" src/d.js              # not ready -> not
  local out; out="$(node "$BS" --repo "$d" --json 2>/dev/null)"
  grep -q "TASK-A-002" <<<"$out" || { no t03_eligibility "task with a done dep not eligible"; return; }
  grep -q "TASK-A-003" <<<"$out" && { no t03_eligibility "task with an UNSATISFIABLE dep was eligible"; return; }
  grep -q "TASK-A-004" <<<"$out" && { no t03_eligibility "a draft was eligible"; return; }
  ok t03_eligibility
}
# --- t04: cone conflict on a shared FILE
t04_file_conflict_excludes(){
  local d="$TMP/t04"
  spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" shared/x.js
  spec "$d" TASK-A-002 ready_to_implement p2 svc/b "" shared/x.js
  local out; out="$(node "$BS" --repo "$d" 2>/dev/null)"
  grep -q "excluded TASK-A-002" <<<"$out" || { no t04_file_conflict_excludes "shared file did not exclude: $out"; return; }
  grep -q "shared/x.js" <<<"$out" || { no t04_file_conflict_excludes "the conflicting file is not named: $out"; return; }
  ok t04_file_conflict_excludes
}
# --- t05: cone conflict on SERVICE. TASK-IMP-104 proved declared file lists are optimistic:
#          it declared install.sh and edited version.sh + update-check.sh, both in its service.
t05_service_conflict_excludes(){
  local d="$TMP/t05"
  spec "$d" TASK-A-001 ready_to_implement p1 tools/install "" tools/install/a.sh
  spec "$d" TASK-A-002 ready_to_implement p2 tools/install "" tools/install/b.sh   # disjoint FILES
  local out; out="$(node "$BS" --repo "$d" 2>/dev/null)"
  grep -q "excluded TASK-A-002" <<<"$out" || { no t05_service_conflict_excludes "same service batched together - files-only cone reading: $out"; return; }
  ok t05_service_conflict_excludes
}
# --- t06: a file INSIDE another task's service conflicts (the case my hand-analysis missed)
t06_file_inside_sibling_service_conflicts(){
  local d="$TMP/t06"
  spec "$d" TASK-A-001 ready_to_implement p1 modules/skill "" tools/install/templates/T.md
  spec "$d" TASK-A-002 ready_to_implement p2 tools/install "" tools/install/uninstall.sh
  local out; out="$(node "$BS" --repo "$d" 2>/dev/null)"
  grep -q "excluded TASK-A-002" <<<"$out" || { no t06_file_inside_sibling_service_conflicts "a file inside a sibling's service did not conflict: $out"; return; }
  ok t06_file_inside_sibling_service_conflicts
}
# --- t07: swarm_required is arithmetic, not opinion
t07_swarm_required_flag(){
  local d="$TMP/t07"; spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js
  node "$BS" --repo "$d" --json 2>/dev/null | grep -q '"swarm_required": false' || { no t07_swarm_required_flag "1-member batch claimed swarm_required"; return; }
  spec "$d" TASK-A-002 ready_to_implement p2 svc/b "" src/b.js
  node "$BS" --repo "$d" --json 2>/dev/null | grep -q '"swarm_required": true' || { no t07_swarm_required_flag "2-member batch did not require a swarm"; return; }
  ok t07_swarm_required_flag
}
# --- t08: every exclusion names its reason, or the artefact cannot make a skipped batch visible
t08_exclusions_name_the_conflict(){
  local d="$TMP/t08"
  spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" shared/x.js
  spec "$d" TASK-A-002 ready_to_implement p2 svc/a "" shared/x.js
  node "$BS" --repo "$d" --json 2>/dev/null | grep -q '"conflict"' || { no t08_exclusions_name_the_conflict "exclusion carries no conflict reason"; return; }
  node "$BS" --repo "$d" --json 2>/dev/null | grep -q '"blocked_by": "TASK-A-001"' || { no t08_exclusions_name_the_conflict "exclusion does not name the blocker"; return; }
  ok t08_exclusions_name_the_conflict
}
# --- t09: an empty corpus is not a crash; a missing one exits 3 with a reason
t09_empty_and_missing_corpus(){
  local d="$TMP/t09"; mkdir -p "$d/docs/tasks"
  node "$BS" --repo "$d" >/dev/null 2>&1 || { no t09_empty_and_missing_corpus "empty corpus did not exit 0"; return; }
  local e="$TMP/t09b"; mkdir -p "$e"
  node "$BS" --repo "$e" >/dev/null 2>&1; [ $? -eq 3 ] || { no t09_empty_and_missing_corpus "missing corpus did not exit 3"; return; }
  ok t09_empty_and_missing_corpus
}
# --- t10: the payload carries it. §11a names the vendored path; a payload without it cannot obey.
t12_undeclared_cone_ships_alone(){
  # An undeclared cone is UNKNOWN, not empty. Before this arm, {} conflicted with nothing, so a
  # silent spec joined EVERY batch - TASK-IMP-117 rewrites 501 specs, TASK-TEMPLATE.md and
  # build.sh, declared none of it, and was admitted alongside a task excluded for touching
  # build.sh. Two sub-agents, one file, one parallel round. (Operator decision, 2026-07-17.)
  #
  # Written by hand, not via spec(): that helper always emits `new_files:\n  - (none)` and a
  # `service:` line, so it CANNOT express a spec that declares nothing - which is exactly the
  # shape under test. A fixture that cannot produce the defect cannot test the fix.
  local d="$TMP/t12"
  spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js     # declares a cone
  mkdir -p "$d/docs/tasks/m/TASK-A-002"
  printf -- '---\nid: TASK-A-002\nstatus: ready_to_implement\npriority: p1\ndepends_on: []\n---\n# TASK-A-002\n' \
    > "$d/docs/tasks/m/TASK-A-002/spec.md"                          # no service, no new_files, no modified_files
  local out; out="$(node "$BS" --repo "$d" 2>/dev/null)"
  grep -qE "BATCH +TASK-A-001" <<<"$out" || { no t12_undeclared_cone_ships_alone "a declared task was not batched: $out"; return; }
  grep -qE "excluded TASK-A-002" <<<"$out" || { no t12_undeclared_cone_ships_alone "an UNDECLARED cone was admitted - it conflicts with nothing only because it says nothing: $out"; return; }
  grep -q "no cone declared" <<<"$out" || { no t12_undeclared_cone_ships_alone "exclusion did not name the reason: $out"; return; }
  grep -q "conflicts with null" <<<"$out" && { no t12_undeclared_cone_ships_alone "printed a blocking task that does not exist: $out"; return; }
  ok t12_undeclared_cone_ships_alone
}

t13_declaring_a_cone_readmits_it(){
  # The refusal must be about the DECLARATION, not the task. Same task, cone declared -> batched.
  # Without this, "exclude everything" would pass t12 and break batching entirely.
  local d="$TMP/t13"
  spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js
  spec "$d" TASK-A-002 ready_to_implement p1 svc/b "" src/b.js     # now declares, and does not clash
  local out; out="$(node "$BS" --repo "$d" 2>/dev/null)"
  grep -qE "BATCH +TASK-A-002" <<<"$out" || { no t13_declaring_a_cone_readmits_it "a declared, non-clashing task was refused: $out"; return; }
  ok t13_declaring_a_cone_readmits_it
}

t11_deterministic_output_is_byte_identical(){
  # The file header says "Deterministic by construction". Nothing asserted it, and the code had
  # drifted: `generated: new Date()` made one corpus yield a different artefact each day, so a
  # consumer diffing batch-selection@1 for equality would see a change that was only the calendar.
  # A claim with no test is a comment. (External review 2026-07-17 - TASK-IMP-118's shape exactly.)
  local d="$TMP/t11"
  spec "$d" TASK-A-001 ready_to_implement p1 svc/a "" src/a.js
  spec "$d" TASK-A-002 ready_to_implement p1 svc/b "" src/b.js
  local a b
  a="$(node "$BS" --repo "$d" --json 2>/dev/null)"
  b="$(node "$BS" --repo "$d" --json 2>/dev/null)"
  [ -n "$a" ] || { no t11_deterministic_output_is_byte_identical "no output"; return; }
  grep -q '"generated"' <<<"$a" && { no t11_deterministic_output_is_byte_identical "artefact still carries a wall-clock field"; return; }
  if [ "$a" = "$b" ]; then ok t11_deterministic_output_is_byte_identical
  else no t11_deterministic_output_is_byte_identical "two runs over one corpus differ - the header's determinism claim is false"; fi
}

t10_payload_carries_it(){
  local p="$root/dist/cyberos/docs-tools/batch-select.mjs"
  [ -f "$p" ] || { no t10_payload_carries_it "payload lacks batch-select.mjs - §11a names a path that does not exist"; return; }
  grep -q 'node .cyberos/docs-tools/batch-select.mjs' "$root/dist/cyberos/cuo/ship-tasks.md" \
    || { no t10_payload_carries_it "payload doctrine does not name the runner"; return; }
  ok t10_payload_carries_it
}
echo "test_batch_select.sh (ship-tasks v2.8.0 mandatory step)"
t01_documented_invocation_works; t02_flag_order_independent; t03_eligibility
t04_file_conflict_excludes; t05_service_conflict_excludes; t06_file_inside_sibling_service_conflicts
t07_swarm_required_flag; t08_exclusions_name_the_conflict; t09_empty_and_missing_corpus
t10_payload_carries_it; t11_deterministic_output_is_byte_identical; t12_undeclared_cone_ships_alone; t13_declaring_a_cone_readmits_it
echo "  ---"; echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
