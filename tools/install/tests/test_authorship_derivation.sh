#!/usr/bin/env bash
# test_authorship_derivation.sh — TASK-IMP-124 TRACE-007 / COND-004 authorship rule.
#
# Mechanical halves only: rubric text, skill instructions, anti-example pins.
# AC 10 (judgment re-audit of three documents) is MANUAL by construction.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
RUBRIC="$repo/modules/skill/task-audit/RUBRIC.md"
AUDIT_SKILL="$repo/modules/skill/task-audit/SKILL.md"
AUTHOR_SKILL="$repo/modules/skill/task-author/SKILL.md"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }

rubric_has() { grep -qE "$1" "$RUBRIC"; }

t01_rubric_carries_trace_007() {
  local why=""
  rubric_has 'TRACE-007' || why="$why no-TRACE-007-row"
  # four failure conditions
  for w in ABSENT 'RE-RUN' REPRODUCE 'NARROWER SCOPE'; do
    grep -qiE "TRACE-007|originated" "$RUBRIC" || why="$why no-trace-007-body"
    grep -qi "$w" "$RUBRIC" || why="$why missing-$w"
  done
  # must be judgment family, not mechanical
  grep -A2 '| `TRACE-007`' "$RUBRIC" | grep -qi 'Judgment' || why="$why not-judgment-family"
  # must NOT live only as a mechanical family row without judgment note
  if grep -A2 '| `TRACE-007`' "$RUBRIC" | grep -qi 'Judgment family'; then :; else why="$why no-judgment-family-label"; fi
  [ -z "$why" ] && ok t01 || fail t01 "$why"
}

t02_three_classes_with_both_halves() {
  local why="" blk
  blk="$(awk '/### TRACE-007/,/^---$/' "$RUBRIC")"
  for c in NUMERIC CITATION 'UNIVERSAL NEGATIVE'; do
    grep -q "$c" <<<"$blk" || why="$why missing-class:$c"
  done
  grep -qi 'author.s own candidate\|author'\''s own candidate\|author’s own candidate\|own candidate' <<<"$blk" \
    || why="$why no-own-candidate-non-discharge"
  grep -qi 'REVISION\|revision' <<<"$blk" || why="$why no-citation-revision"
  grep -qi 'What does NOT\|does NOT' <<<"$blk" || why="$why no-non-discharge-column"
  [ -z "$why" ] && ok t02 || fail t02 "$why"
}

t03_originated_defined_and_ordered_first() {
  local why="" blk
  blk="$(awk '/### TRACE-007/,/^---$/' "$RUBRIC")"
  grep -qi 'ORIGINATED' <<<"$blk" || why="$why no-originated"
  grep -qi 'INHERITED\|inherited' <<<"$blk" || why="$why no-inherited"
  grep -qiE 'FIRST|first' <<<"$blk" || why="$why no-ordering"
  [ -z "$why" ] && ok t03 || fail t03 "$why"
}

t04_both_anti_examples_are_worked() {
  local why="" blk
  blk="$(awk '/### TRACE-007/,/^---$/' "$RUBRIC")"
  for v in 1525 1534 102dc507 86cafee8; do
    grep -q "$v" <<<"$blk" || why="$why missing-$v"
  done
  grep -q '63705483' <<<"$blk" || why="$why no-122-rev"
  grep -q '15894b1e' <<<"$blk" || why="$why no-121-rev"
  grep -qiE 'BYTE-EXACT|counter-example|6B' <<<"$blk" || why="$why no-universal-negative-witness"
  git cat-file -e 63705483^{commit} 2>/dev/null || why="$why 63705483-unresolvable"
  git cat-file -e 15894b1e^{commit} 2>/dev/null || why="$why 15894b1e-unresolvable"
  # bare path without revision would be wrong for anti-examples — require sha pins
  grep -qE '`[0-9a-f]{7,}:' <<<"$blk" || why="$why no-sha-pinned-citation"
  [ -z "$why" ] && ok t04 || fail t04 "$why"
}

t05_kinship_to_trace_006_stated() {
  local why="" blk
  blk="$(awk '/### TRACE-007/,/^---$/' "$RUBRIC")"
  grep -q 'TRACE-006' <<<"$blk" || why="$why no-TRACE-006-citation"
  grep -qi 'VERB\|verb' <<<"$blk" || why="$why no-verb-half"
  grep -qi 'SCOPE\|scope' <<<"$blk" || why="$why no-scope-half"
  [ -z "$why" ] && ok t05 || fail t05 "$why"
}

t06_cond_004_requires_the_partition() {
  local why=""
  grep -A3 '| `COND-004`' "$RUBRIC" | grep -q 're-derived and CONFIRMED' || why="$why no-CONFIRMED"
  grep -A3 '| `COND-004`' "$RUBRIC" | grep -q 're-derived and CORRECTED' || why="$why no-CORRECTED"
  grep -A3 '| `COND-004`' "$RUBRIC" | grep -q 'measured and ADDED' || why="$why no-ADDED"
  grep -A3 '| `COND-004`' "$RUBRIC" | grep -qi 'every claim was re-measured' || why="$why no-universal-forbid"
  grep -A3 '| `COND-004`' "$RUBRIC" | grep -q '1525' || why="$why no-1525-reason"
  [ -z "$why" ] && ok t06 || fail t06 "$why"
}

t07_skill_tests_the_disclosure() {
  local why=""
  grep -q 'TRACE-007' "$AUDIT_SKILL" || why="$why no-TRACE-007-in-skill"
  grep -qi 'never as evidence of diligence\|NEVER credit the disclosure' "$AUDIT_SKILL" \
    || why="$why credits-disclosure"
  grep -q 'CONFIRMED' "$AUDIT_SKILL" || why="$why no-CONFIRMED-finding"
  grep -qi 'none of the three' "$AUDIT_SKILL" || why="$why no-unpartitioned-finding"
  [ -z "$why" ] && ok t07 || fail t07 "$why"
}

t08_author_records_at_origination() {
  local why=""
  grep -qi 'AT THE.*MOMENT\|at the moment\|ORIGINATION\|origination' "$AUTHOR_SKILL" \
    || why="$why no-origination-moment"
  grep -q 'source_pages' "$AUTHOR_SKILL" || why="$why no-source_pages"
  # must not defer the instruction solely to review
  if grep -qi 'record.*derivation.*review time' "$AUTHOR_SKILL" \
     && ! grep -qi 'not at review time' "$AUTHOR_SKILL"; then
    why="$why deferred-to-review-only"
  fi
  grep -qi 'not at review time' "$AUTHOR_SKILL" || why="$why missing-not-at-review"
  [ -z "$why" ] && ok t08 || fail t08 "$why"
}

want t01 && t01_rubric_carries_trace_007
want t02 && t02_three_classes_with_both_halves
want t03 && t03_originated_defined_and_ordered_first
want t04 && t04_both_anti_examples_are_worked
want t05 && t05_kinship_to_trace_006_stated
want t06 && t06_cond_004_requires_the_partition
want t07 && t07_skill_tests_the_disclosure
want t08 && t08_author_records_at_origination

echo "test_authorship_derivation: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
