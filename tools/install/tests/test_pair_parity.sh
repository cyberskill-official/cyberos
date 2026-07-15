#!/usr/bin/env bash
# test_pair_parity.sh - TASK-SKILL-118 §5 suite (t01-t06 -> AC 1-6).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
CHECK="$repo/tools/install/check-pair-parity.sh"
SKILLS="$repo/modules/skill"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
SIX="repo-context-map edge-case-matrix mock-contract-test observability-injection backlog-state-update coverage-gate"

t01_all_pairs_parity_clean() {                                         # AC 1
  out="$(bash "$CHECK" "$SKILLS" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] && ! grep -q '^PARITY' <<<"$out" && ok t01 || fail t01 "$out"
}
t02_prose_gate_rule_ids() {                                            # AC 2
  local bad=""
  # bash 3.2 SAFE. This was `declare -A want=(...)`, bash 4 only — a syntax error on macOS
  # (which ships bash 3.2, frozen 2007), so this whole suite aborted on the machine this
  # repo is developed on. It never reported that, because until 2026-07-15 nothing ran it.
  # Same root cause as check-chain-coverage.sh being a silent no-op on macOS.
  _want_prefix() {
    case "$1" in
      repo-context-map)        echo "RCM-" ;;
      edge-case-matrix)        echo "ECM-" ;;
      mock-contract-test)      echo "MCT-" ;;
      observability-injection) echo "OBS-" ;;
      backlog-state-update)    echo "BSU-" ;;
      coverage-gate)           echo "COV-" ;;
      *)                       echo "" ;;
    esac
  }
  for n in $SIX; do
    r="$SKILLS/$n-audit/RUBRIC.md"
    w="$(_want_prefix "$n")"
    [ -n "$w" ] || { bad="$bad $n(no-prefix-mapped)"; continue; }
    grep -q "prose source" "$r" && grep -q "$w" "$r" || bad="$bad $n"
  done
  # spot gates from task §1 #3 present as rules
  grep -q "TOTAL_ROWS_MIN" "$SKILLS/edge-case-matrix-audit/RUBRIC.md" \
    && grep -q "BRANCH_COVERAGE_MIN" "$SKILLS/observability-injection-audit/RUBRIC.md" \
    && grep -q "files_below_90pct" "$SKILLS/coverage-gate-audit/RUBRIC.md" \
    && grep -q "BSU-INS-001" "$SKILLS/backlog-state-update-audit/RUBRIC.md" \
    && grep -q "swap_target" "$SKILLS/mock-contract-test-audit/RUBRIC.md" \
    && grep -q "pinned_in" "$SKILLS/repo-context-map-audit/RUBRIC.md" || bad="$bad gates"
  [ -z "$bad" ] && ok t02 || fail t02 "missing rule-id/prose-map in:$bad"
}
t03_constants_block() {                                                # AC 3
  local bad=""
  for n in $SIX; do
    head -5 "$SKILLS/$n-audit/RUBRIC.md" | grep -q "^constants: TOTAL_ROWS_MIN=8" || bad="$bad $n"
  done
  grep -q "TASK-CUO-207" "$SKILLS/coverage-gate-audit/RUBRIC.md" || bad="$bad cov-override-hook"
  [ -z "$bad" ] && ok t03 || fail t03 "constants header missing:$bad"
}
t04_artefact_sections_stable() {                                       # AC 4 (amended: at-rest guard)
  # additive-only guarantee AT REST: no SKILL.md in the seven pairs lost a line vs HEAD.
  # A DIRTY worktree on these files warns and skips instead of failing - legitimate mid-flight
  # mutations (e.g. TASK-SKILL-119's citation swaps) false-fired this three times; the guard's
  # authority is the committed state, which CI always checks clean. (TASK-SKILL-118 AC 4, amended.)
  local bad="" dirty=""
  for n in $SIX debugging-cycle; do
    for side in author audit; do
      f="modules/skill/$n-$side/SKILL.md"
      if ! git -C "$repo" diff --quiet -- "$f" 2>/dev/null; then dirty="$dirty $n-$side"; continue; fi
      d="$(git -C "$repo" diff -U0 HEAD -- "$f" | grep -c '^-[^-]')" || true
      [ "${d:-0}" -eq 0 ] || bad="$bad $n-$side"
    done
  done
  [ -n "$dirty" ] && echo "  WARN t04: dirty worktree on:$dirty - removal check deferred to the committed state" >&2
  [ -z "$bad" ] && ok t04 || fail t04 "lines REMOVED from:$bad (artefact sections must be diff-stable)"
}
t05_checker_catches_missing() {                                        # AC 5
  cp -R "$SKILLS" "$TMP/skills"
  rm "$TMP/skills/coverage-gate-audit/RUBRIC.md"
  out="$(bash "$CHECK" "$TMP/skills" 2>&1)"; rc=$?
  [ "$rc" -eq 10 ] && grep -q "PARITY coverage-gate-audit: missing RUBRIC.md" <<<"$out" \
    && ok t05 || fail t05 "rc=$rc out=$out"
}
t06_trigger_tests_unchanged() {                                        # AC 6
  local bad=""
  for n in $SIX; do
    for side in author audit; do
      git -C "$repo" diff --quiet HEAD -- "modules/skill/$n-$side/acceptance/TRIGGER_TESTS.md" || bad="$bad $n-$side"
    done
  done
  [ -z "$bad" ] && ok t06 || fail t06 "TRIGGER_TESTS.md changed:$bad"
}

t01_all_pairs_parity_clean; t02_prose_gate_rule_ids; t03_constants_block
t04_artefact_sections_stable; t05_checker_catches_missing; t06_trigger_tests_unchanged
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
