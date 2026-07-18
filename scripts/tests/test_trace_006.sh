#!/usr/bin/env bash
# test_trace_006.sh — TASK-IMP-118 §1 suite (t01-t05 -> AC1-AC5).
#
# WHAT THIS GUARDS: TRACE-006 is a judgment-family rubric rule — "a test cited by a §1 clause MUST
# exercise that clause's own VERB". It is UNMECHANIZABLE by construction (§1.5): no lint can decide
# whether a test exercises a clause's verb, because that means reading a test and a sentence and
# comparing their meaning. So these tests do NOT judge the rule; they check the STRUCTURE that
# carries it — that RUBRIC.md carries the rule and its verb→evidence table and its worked
# anti-example, that task-audit/SKILL.md instructs the comparison, and that TRACE-006 stays OUT of
# the machine floor. Where a check is a structural proxy for the unmechanizable clause, its header
# says so plainly and names what it does NOT cover.
#
# HONEST BOUND (read before trusting a green): t01-t04 assert the rule TEXT is present and names its
# parts. They CANNOT verify a model applies TRACE-006 correctly — that is the rule's own
# unmechanizability, and it is why the rule is judgment-family and why AC6 (re-auditing TASK-IMP-108
# §1.7) is `verify: MANUAL` and is NOT in this suite: a shell test asserting "a recorded audit says
# FAIL" would be weaker than the clause's verb — the exact TASK-IMP-118 defect committed inside 118.
# t05's absence-from-task-lint arm is the one fully-mechanizable check here and the strongest.
#
# WHY scripts/tests/ AND NOT modules/skill/task-audit/tests/: this is the only home that is (a)
# gated — scripts/tests/run_all.sh globs scripts/tests/, tools/docs-site/tests/, tools/install/tests/
# and the pre-commit hook runs it; a suite anywhere else is never executed — and (b) not inside a
# forbidden cone (tools/install/ is TASK-IMP-117's; modules/cuo/ is off-limits). The sibling
# rubric/skill suites (test_template_schema.sh, test_check_doc_anchors.sh) live here for the same
# reason. The 118 spec's `new_files:(none)` + bare test names is a spec gap; flagged for the gate.
#
#   bash scripts/tests/test_trace_006.sh
set -uo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  \033[32mok\033[0m   %s\n' "$1"; }
fail() { FAIL=$((FAIL+1)); printf '  \033[31mFAIL\033[0m %s: %s\n' "$1" "$2"; }

RUBRIC="$repo/modules/skill/task-audit/RUBRIC.md"
SKILL="$repo/modules/skill/task-audit/SKILL.md"
# The machine floor, in the platform repo (SKILL.md §3: .cyberos/docs-tools/task-lint.mjs in an
# installed repo, tools/install/docs-tools/task-lint.mjs here). READ-ONLY here — editing it is
# TASK-IMP-117's cone and would violate §1.5. TRACE006_LINT_OVERRIDE lets the load-bearing proof
# point t05 at a mutated /tmp copy (task-lint-with-TRACE-006-wired-in) WITHOUT touching the real
# floor; unset in every normal and CI run.
LINT="${TRACE006_LINT_OVERRIDE:-$repo/tools/install/docs-tools/task-lint.mjs}"

# The TRACE-006 subsection of RUBRIC.md §9: from its `### TRACE-006` heading to the §9 terminator
# (`---`) or the next H2. Empty output means the subsection is gone — which is a failure, not a pass.
trace006_subsection() {
  awk '/^### TRACE-006/{f=1} f&&/^---$/{exit} f&&/^## /{exit} f{print}' "$RUBRIC"
}
# The TRACE-006 instruction block of SKILL.md: from its `### TRACE-006` heading to the next heading.
skill_trace006_block() {
  awk '/^### TRACE-006/{f=1; print; next} f&&/^#{2,3} /{exit} f{print}' "$SKILL"
}

# --- t01 / AC1 / §1.1 -------------------------------------------------------------------
# §1.1: RUBRIC.md MUST carry TRACE-006 — for each clause citing a test, the audit states the
# clause's VERB, states what the cited test ASSERTS, and FAILS when the assertion is weaker.
# STRUCTURAL PROXY: asserts the rule ROW and its DEFINING substance are present (verb / assertion /
# weaker-fails), not merely the token "TRACE-006" (present-of-token would itself be the weaker-than-
# the-clause defect this rule forbids). Does NOT verify a model applies the rule (unmechanizable).
t01_rubric_carries_trace_006() {
  [ -f "$RUBRIC" ] || { fail t01_rubric_carries_trace_006 "RUBRIC.md missing at $RUBRIC"; return; }
  local why=""
  grep -qE '^\| `TRACE-006`' "$RUBRIC"        || why="$why no-§9-table-row"
  local blk; blk="$(trace006_subsection)"
  [ -n "$blk" ]                               || why="$why no-TRACE-006-subsection"
  grep -qi 'verb'   <<<"$blk"                 || why="$why no-verb-half"
  grep -qi 'assert' <<<"$blk"                 || why="$why no-assertion-half"
  grep -qi 'weaker' <<<"$blk"                 || why="$why no-weaker-fails-rule"
  [ -z "$why" ] && ok t01_rubric_carries_trace_006 || fail t01_rubric_carries_trace_006 "$why"
}

# --- t02 / AC2 / §1.2 -------------------------------------------------------------------
# §1.2: TRACE-006 MUST carry a verb→evidence table covering >= render, reject, refuse, halt, emit,
# preserve — naming, for each, what discharges it AND what does NOT.
# STRUCTURAL PROXY: asserts all six verbs are listed and the "does NOT discharge" half is named. A
# table with only the discharging column is half a table (edge rows 6/7/8: name-not-evidence,
# log-line-not-refusal, happy-path-not-rejection live in that column). Does NOT verify the model
# applies the table.
t02_verb_table_is_complete() {
  local blk; blk="$(trace006_subsection)"
  local why="" v
  [ -n "$blk" ] || { fail t02_verb_table_is_complete "no TRACE-006 subsection"; return; }
  for v in render reject refuse halt emit preserve; do
    grep -qiw "$v" <<<"$blk" || why="$why missing-verb:$v"
  done
  grep -qi 'not discharge' <<<"$blk" || why="$why no-non-discharging-column"
  [ -z "$why" ] && ok t02_verb_table_is_complete || fail t02_verb_table_is_complete "$why"
}

# --- t03 / AC3 / §1.3 -------------------------------------------------------------------
# §1.3: TRACE-006 MUST use 108 §1.7 as its worked anti-example, quoting the clause, the original
# assertion, and why the assertion was weaker.
# Verified against the real 108 §1.7 (spec line 114) and the test's own history comment
# (test_render_status_hub.sh::t11) before authoring. STRUCTURAL PROXY for "quotes ... specifically".
t03_anti_example_is_present_and_specific() {
  local blk; blk="$(trace006_subsection)"
  local why=""
  [ -n "$blk" ] || { fail t03_anti_example_is_present_and_specific "no TRACE-006 subsection"; return; }
  grep -qE '108' <<<"$blk"                                  || why="$why no-108-ref"
  grep -qE '1\.7' <<<"$blk"                                 || why="$why no-clause-1.7-ref"
  grep -qi 'render' <<<"$blk"                               || why="$why no-clause-verb-quoted"
  grep -q 'draft_staleness' <<<"$blk"                       || why="$why no-original-assertion-quoted"
  grep -qiE 'payload|no code reads|present-in-payload' <<<"$blk" || why="$why no-weakness-rationale"
  [ -z "$why" ] && ok t03_anti_example_is_present_and_specific || fail t03_anti_example_is_present_and_specific "$why"
}

# --- t04 / AC4 / §1.4 -------------------------------------------------------------------
# §1.4: the single-source task-audit/SKILL.md MUST instruct the auditor to perform the comparison
# per clause and to record BOTH halves in the audit body. (build.sh vendors this source to the
# payload; the source is the thing under test.)
t04_skill_instructs_the_comparison() {
  [ -f "$SKILL" ] || { fail t04_skill_instructs_the_comparison "SKILL.md missing at $SKILL"; return; }
  local blk; blk="$(skill_trace006_block)"
  local why=""
  [ -n "$blk" ]                                    || why="$why no-TRACE-006-instruction"
  grep -qi 'verb'   <<<"$blk"                       || why="$why no-verb"
  grep -qi 'assert' <<<"$blk"                       || why="$why no-assertion"
  grep -qiE 'per-clause|per clause'   <<<"$blk"     || why="$why no-per-clause"
  grep -qiE 'both halves|record both' <<<"$blk"     || why="$why no-record-both-halves"
  grep -qiE 'audit body|issue block'  <<<"$blk"     || why="$why no-audit-body"
  [ -z "$why" ] && ok t04_skill_instructs_the_comparison || fail t04_skill_instructs_the_comparison "$why"
}

# --- t05 / AC5 / §1.5 -------------------------------------------------------------------
# §1.5: TRACE-006 MUST be judgment-family and MUST NOT be added to task-lint. A structural check that
# appeared to enforce it would pass 108 §1.7's original test and restore the false assurance.
# (a) is the fully-mechanizable arm: the machine floor has ZERO TRACE-006 — it fails the moment
# anyone wires it in. (b) documents the boundary so it is not merely currently-true.
t05_not_in_the_machine_floor() {
  local why=""
  if [ -f "$LINT" ]; then
    [ "$(grep -c 'TRACE-006' "$LINT")" -eq 0 ] || why="$why TRACE-006-present-in-task-lint"
  else
    why="$why task-lint-not-found:$LINT"   # unreadable is not satisfied (edge row 10)
  fi
  local blk; blk="$(trace006_subsection)"
  grep -qi 'judgment' <<<"$blk"                 || why="$why not-tagged-judgment-family"
  grep -qiE 'task-lint|machine floor' <<<"$blk" || why="$why no-absent-from-lint-statement"
  [ -z "$why" ] && ok t05_not_in_the_machine_floor || fail t05_not_in_the_machine_floor "$why"
}

t01_rubric_carries_trace_006
t02_verb_table_is_complete
t03_anti_example_is_present_and_specific
t04_skill_instructs_the_comparison
t05_not_in_the_machine_floor
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
