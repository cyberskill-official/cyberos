#!/usr/bin/env bash
# test_task_lint.sh - the deterministic machine floor under audit_rubric@2.0 (TASK-IMP-084):
#   t01  CLI shape: file + dir recursion to */spec.md, --json, exit 0/2, and
#        byte-identical output across two runs on identical input.
#   t02  FM family: representative fixtures each yield exactly their rule_id
#        (exotic-YAML FM-001 naming the line, FM-102, FM-105, FM-101, FM-112, FM-114).
#   t03  SEC family: missing '## Summary' -> SEC-001; empty '## Problem' -> SEC-008.
#   t04  COND family: ai_authorship without disclosure -> COND-004;
#        client_visible without Customer Quotes -> COND-001.
#   t05  TRACE structural halves: uncited MUST clause -> TRACE-001; AC without
#        test/verify -> TRACE-002; dangling test path -> TRACE-003; a verify: AC
#        and a new_files-listed test path both pass.
#   t06  green corpus: the three batch-1 specs (TASK-IMP-082/083/084) lint clean.
#   t07  the assembled payload carries docs-tools/task-lint.mjs and a scratch
#        install lays it into .cyberos/docs-tools/.
#   t08  the task-audit skill carries the lint-first machine-floor wiring, in
#        modules/ AND in the payload's cuo/skills + plugin/skills copies.
#   t09  FM-115/FM-116: draft_reason + entered_via are OPTIONAL enums - absent is legal
#        (the 336 untriaged drafts must not red), present-and-wrong names its rule.
#
# Origin: 2026-07-16 sachviet run (IMPROVEMENT_HANDOFF.md IMP-03) - six spec audits
# re-derived every mechanical rubric rule by model. The rubric calls itself
# machine-checkable; this suite gates the machine that finally checks it.
#
# Usage: bash test_task_lint.sh [t01 t02 ...]   (no args = all scenarios)
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
LINT="$repo/tools/install/docs-tools/task-lint.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }

# One fully green task@1 fixture; every red fixture is a single mutation of it,
# so each scenario isolates exactly one rule.
emit_green() {
  mkdir -p "$(dirname "$1")"
  cat > "$1" <<'EOF'
---
template: task@1
title: Green fixture task
author: "@tester"
department: engineering
status: draft
priority: p2
created_at: 2026-07-16T00:00:00Z
ai_authorship: none
type: chore
eu_ai_act_risk_class: not_ai
client_visible: false
new_files: []
---

# Green fixture task

## Summary

A fixture.

## Problem

Something.

## Proposed Solution

Do a thing.

## Alternatives Considered

- Do nothing.
- Do it later.

## Success Metrics

- Zero findings, on every run.

## Scope

In scope: this fixture.

## Dependencies

- None.

## 1. Description (normative)

- 1.1 The fixture MUST lint clean.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1) - fixture lints clean - verify: manual ops check (fixture; nothing to automate)
EOF
}

run_lint() { node "$LINT" "$@" > "$TMP/out" 2> "$TMP/err"; }
err_count() { grep -c '^error ' "$TMP/out" || true; }

# expect_one_rule <name> <fixture> <rule_id>: exit 2, exactly one error, right id
expect_one_rule() {
  run_lint "$2"; local rc=$?
  [ "$rc" -eq 2 ] || { fail "$1" "expected exit 2, got $rc ($(cat "$TMP/out"))"; return 1; }
  [ "$(err_count)" -eq 1 ] || { fail "$1" "expected exactly 1 error, got: $(cat "$TMP/out")"; return 1; }
  grep -q "^error $3 " "$TMP/out" || { fail "$1" "expected $3, got: $(cat "$TMP/out")"; return 1; }
  return 0
}

t01_cli_and_determinism() {
  local d="$TMP/t01"; emit_green "$d/green/spec.md"
  sed 's/^priority: p2$/priority: p9/' "$d/green/spec.md" > "$d/green/tmp" \
    && mkdir -p "$d/red" && mv "$d/green/tmp" "$d/red/spec.md"
  # green file -> exit 0, no findings
  run_lint "$d/green/spec.md"; local rc=$?
  { [ "$rc" -eq 0 ] && [ ! -s "$TMP/out" ]; } || { fail t01 "green fixture: rc=$rc out=$(cat "$TMP/out")"; return; }
  # red file -> exit 2, findings present
  run_lint "$d/red/spec.md"; rc=$?
  { [ "$rc" -eq 2 ] && [ -s "$TMP/out" ]; } || { fail t01 "red fixture: rc=$rc"; return; }
  # no args -> usage error, exit 2
  node "$LINT" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 2 ] || { fail t01 "no-args exit: $rc"; return; }
  # unreadable input -> template_ambiguous error, exit 2 (never a guess, never a crash)
  run_lint "$d/absent/spec.md"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "template_ambiguous" "$TMP/out"; } \
    || { fail t01 "missing path: rc=$rc out=$(cat "$TMP/out")"; return; }
  # dir recursion finds */spec.md; two runs byte-identical (text + json)
  run_lint "$d"; rc=$?
  [ "$rc" -eq 2 ] || { fail t01 "dir run rc=$rc"; return; }
  grep -q "red/spec.md" "$TMP/out" || { fail t01 "dir recursion missed red/spec.md"; return; }
  cp "$TMP/out" "$TMP/run1"
  run_lint "$d"; cp "$TMP/out" "$TMP/run2"
  cmp -s "$TMP/run1" "$TMP/run2" || { fail t01 "two text runs differ"; return; }
  node "$LINT" --json "$d" > "$TMP/j1" 2>/dev/null
  node "$LINT" --json "$d" > "$TMP/j2" 2>/dev/null
  cmp -s "$TMP/j1" "$TMP/j2" || { fail t01 "two --json runs differ"; return; }
  node -e "JSON.parse(require('fs').readFileSync('$TMP/j1','utf8'))" 2>/dev/null \
    || { fail t01 "--json output is not valid JSON"; return; }
  ok t01
}

t02_fm_family() {
  local d="$TMP/t02"; emit_green "$d/base/spec.md"; local base="$d/base/spec.md"
  # FM-001: exotic YAML (anchor) fails loudly, naming the line
  mkdir -p "$d/f001"; sed 's/^new_files: \[\]$/new_files: \&keep [x]/' "$base" > "$d/f001/spec.md"
  expect_one_rule "t02 FM-001" "$d/f001/spec.md" FM-001 || return
  local ln; ln="$(grep -n '&keep' "$d/f001/spec.md" | cut -d: -f1)"
  grep -q ":$ln " "$TMP/out" || { fail t02 "FM-001 does not name line $ln: $(cat "$TMP/out")"; return; }
  # FM-105: bad priority
  mkdir -p "$d/f105"; sed 's/^priority: p2$/priority: p9/' "$base" > "$d/f105/spec.md"
  expect_one_rule "t02 FM-105" "$d/f105/spec.md" FM-105 || return
  # FM-112: '# UNREVIEWED' marker in frontmatter
  mkdir -p "$d/f112"; awk '{print} /^ai_authorship: none$/{print "# UNREVIEWED"}' "$base" > "$d/f112/spec.md"
  expect_one_rule "t02 FM-112" "$d/f112/spec.md" FM-112 || return
  # FM-114: severity without type: bug
  mkdir -p "$d/f114"; awk '{print} /^type: chore$/{print "severity: sev2"}' "$base" > "$d/f114/spec.md"
  expect_one_rule "t02 FM-114" "$d/f114/spec.md" FM-114 || return
  # FM-101: title over 72 code points
  mkdir -p "$d/f101"
  sed "s/^title: .*/title: $(printf 'x%.0s' {1..80})/" "$base" > "$d/f101/spec.md"
  expect_one_rule "t02 FM-101" "$d/f101/spec.md" FM-101 || return
  # FM-102: author without the @ handle shape
  mkdir -p "$d/f102"; sed 's/^author: .*/author: tester/' "$base" > "$d/f102/spec.md"
  expect_one_rule "t02 FM-102" "$d/f102/spec.md" FM-102 || return
  # FM-004: template key absent -> template_ambiguous, file stopped (no downstream findings)
  mkdir -p "$d/f004"; sed '/^template: task@1$/d' "$base" > "$d/f004/spec.md"
  expect_one_rule "t02 FM-004" "$d/f004/spec.md" FM-004 || return
  grep -q "template_ambiguous" "$TMP/out" || { fail t02 "FM-004 lacks template_ambiguous: $(cat "$TMP/out")"; return; }
  ok t02
}

t03_sec_family() {
  local d="$TMP/t03"; emit_green "$d/base/spec.md"; local base="$d/base/spec.md"
  # SEC-001: '## Summary' missing entirely
  mkdir -p "$d/s001"; awk '/^## Summary$/{skip=1} /^## Problem$/{skip=0} !skip{print}' "$base" > "$d/s001/spec.md"
  expect_one_rule "t03 SEC-001" "$d/s001/spec.md" SEC-001 || return
  grep -q "Summary" "$TMP/out" || { fail t03 "SEC-001 finding does not name Summary"; return; }
  # SEC-008: '## Problem' present but empty
  mkdir -p "$d/s008"; sed '/^Something\.$/d' "$base" > "$d/s008/spec.md"
  expect_one_rule "t03 SEC-008" "$d/s008/spec.md" SEC-008 || return
  grep -q "Problem" "$TMP/out" || { fail t03 "SEC-008 finding does not name Problem"; return; }
  ok t03
}

t04_cond_family() {
  local d="$TMP/t04"; emit_green "$d/base/spec.md"; local base="$d/base/spec.md"
  # COND-004: ai_authorship: assisted with no '## AI Authorship Disclosure'
  mkdir -p "$d/c004"; sed 's/^ai_authorship: none$/ai_authorship: assisted/' "$base" > "$d/c004/spec.md"
  expect_one_rule "t04 COND-004" "$d/c004/spec.md" COND-004 || return
  # COND-001: client_visible: true with Sales/CS Summary present but no Customer Quotes
  mkdir -p "$d/c001"
  { sed 's/^client_visible: false$/client_visible: true/' "$base"
    printf '\n## Sales/CS Summary\n\nPlain words a seller can repeat.\n'; } > "$d/c001/spec.md"
  expect_one_rule "t04 COND-001" "$d/c001/spec.md" COND-001 || return
  ok t04
}

t05_trace_family() {
  local d="$TMP/t05"; emit_green "$d/base/spec.md"; local base="$d/base/spec.md"
  # TRACE-001: a MUST clause no AC cites
  mkdir -p "$d/r001"
  awk '{print} /^- 1\.1 /{print "- 1.2 The fixture MUST also frob."}' "$base" > "$d/r001/spec.md"
  expect_one_rule "t05 TRACE-001" "$d/r001/spec.md" TRACE-001 || return
  grep -q "1\.2" "$TMP/out" || { fail t05 "TRACE-001 does not name clause 1.2"; return; }
  # TRACE-002: AC line with neither test: nor verify:
  mkdir -p "$d/r002"
  sed 's/^- \[ \] AC 1 .*/- [ ] AC 1 (traces_to: §1 #1.1) - fixture lints clean/' "$base" > "$d/r002/spec.md"
  expect_one_rule "t05 TRACE-002" "$d/r002/spec.md" TRACE-002 || return
  # TRACE-003: test path neither in new_files nor on disk
  mkdir -p "$d/r003"
  sed 's|^- \[ \] AC 1 .*|- [ ] AC 1 (traces_to: §1 #1.1) - dangling - test: `no/such/dir/test_x.sh::t01`|' "$base" > "$d/r003/spec.md"
  expect_one_rule "t05 TRACE-003" "$d/r003/spec.md" TRACE-003 || return
  grep -q "no/such/dir/test_x.sh" "$TMP/out" || { fail t05 "TRACE-003 does not name the path"; return; }
  # a verify: AC passes the structural half (the untouched green fixture)
  run_lint "$base"; local rc=$?
  { [ "$rc" -eq 0 ] && [ ! -s "$TMP/out" ]; } || { fail t05 "verify: AC did not pass: rc=$rc $(cat "$TMP/out")"; return; }
  # a test: path listed in frontmatter new_files passes without existing on disk
  mkdir -p "$d/nf"
  sed -e 's|^new_files: \[\]$|new_files: [tools/t.sh]|' \
      -e 's|^- \[ \] AC 1 .*|- [ ] AC 1 (traces_to: §1 #1.1) - planned - test: `tools/t.sh::t01_planned`|' \
      "$base" > "$d/nf/spec.md"
  run_lint "$d/nf/spec.md"; rc=$?
  { [ "$rc" -eq 0 ] && [ ! -s "$TMP/out" ]; } || { fail t05 "new_files-listed test path did not pass: rc=$rc $(cat "$TMP/out")"; return; }
  # zero numbered clauses -> info note, not an error (pure PRD shape stays exit 0)
  mkdir -p "$d/zero"; sed '/^- 1\.1 /d' "$base" > "$d/zero/spec.md"
  run_lint "$d/zero/spec.md"; rc=$?
  { [ "$rc" -eq 0 ] && grep -q "^info TRACE-001" "$TMP/out"; } \
    || { fail t05 "zero-clause spec: rc=$rc out=$(cat "$TMP/out")"; return; }
  ok t05
}

t06_green_corpus() {
  local bad=0 s
  for s in \
    "$repo/docs/tasks/improvement/TASK-IMP-082-status-stamp-byte-stable/spec.md" \
    "$repo/docs/tasks/improvement/TASK-IMP-083-hookspath-aware-status-hook/spec.md" \
    "$repo/docs/tasks/improvement/TASK-IMP-084-task-lint-machine-floor/spec.md"; do
    run_lint "$s"; local rc=$?
    if [ "$rc" -ne 0 ]; then
      bad=1; echo "  --- findings for $s (exit $rc):"; sed 's/^/      /' "$TMP/out"
    fi
  done
  [ "$bad" -eq 0 ] && ok t06 || fail t06 "a batch-1 spec did not lint clean (findings above)"
}

ensure_payload() {
  [ -s "$TMP/payload/install.sh" ] && return 0
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1
}

t07_payload_and_install() {
  ensure_payload || { fail t07 "build.sh failed"; return; }
  [ -s "$TMP/payload/docs-tools/task-lint.mjs" ] || { fail t07 "payload docs-tools/task-lint.mjs missing or empty"; return; }
  cmp -s "$repo/tools/install/docs-tools/task-lint.mjs" "$TMP/payload/docs-tools/task-lint.mjs" \
    || { fail t07 "payload task-lint.mjs differs from tools/install/docs-tools/task-lint.mjs"; return; }
  local d="$TMP/scratch"; mkdir -p "$d"; (cd "$d" && git init -q . 2>/dev/null || true)
  (cd "$d" && CYBEROS_NO_MIGRATE=1 CYBEROS_NO_HOOK=1 CYBEROS_NO_MEMORY=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1) \
    || { fail t07 "install.sh failed"; return; }
  [ -s "$d/.cyberos/docs-tools/task-lint.mjs" ] || { fail t07 ".cyberos/docs-tools/task-lint.mjs missing after install"; return; }
  # the vendored copy actually runs
  emit_green "$TMP/t07/green/spec.md"
  node "$d/.cyberos/docs-tools/task-lint.mjs" "$TMP/t07/green/spec.md" >/dev/null 2>&1 \
    || { fail t07 "installed lint did not exit 0 on a green fixture"; return; }
  ok t07
}

t08_skill_wiring_present() {
  ensure_payload || { fail t08 "build.sh failed"; return; }
  local f
  for f in \
    "$repo/modules/skill/task-audit/SKILL.md" \
    "$TMP/payload/cuo/skills/task-audit/SKILL.md" \
    "$TMP/payload/plugin/skills/task-audit/SKILL.md"; do
    [ -s "$f" ] || { fail t08 "missing $f"; return; }
    grep -q "task-lint\.mjs" "$f" || { fail t08 "$f lacks the task-lint.mjs wiring"; return; }
    grep -q "MUST run it FIRST" "$f" || { fail t08 "$f lacks the lint-first order"; return; }
    grep -q "judgment families" "$f" || { fail t08 "$f lacks the judgment-family split"; return; }
  done
  ok t08
}

t09_optional_status_reason_enums() {
  # TASK-IMP-108 §1.1/§1.2/§1.3: draft_reason + entered_via are OPTIONAL enums. Absent is legal
  # (336 existing drafts carry neither and MUST NOT red); present-and-wrong reds naming the rule.
  local d="$TMP/t09"; emit_green "$d/base/spec.md"; local base="$d/base/spec.md"

  # absent -> clean. This is the arm that protects the corpus from the rule.
  node "$LINT" "$base" >/dev/null 2>&1 || { fail t09 "green base with neither field red"; return; }

  local v
  for v in authoring migrated_stub needs_spec parked_idea; do
    mkdir -p "$d/dr-$v"; awk -v val="$v" '{print} /^status: /{print "draft_reason: " val}' "$base" > "$d/dr-$v/spec.md"
    node "$LINT" "$d/dr-$v/spec.md" >/dev/null 2>&1 || { fail t09 "legal draft_reason '$v' red"; return; }
  done
  for v in audit rework spec_rejected; do
    mkdir -p "$d/ev-$v"; awk -v val="$v" '{print} /^status: /{print "entered_via: " val}' "$base" > "$d/ev-$v/spec.md"
    node "$LINT" "$d/ev-$v/spec.md" >/dev/null 2>&1 || { fail t09 "legal entered_via '$v' red"; return; }
  done

  mkdir -p "$d/f115"; awk '{print} /^status: /{print "draft_reason: whenever"}' "$base" > "$d/f115/spec.md"
  expect_one_rule "t09 FM-115" "$d/f115/spec.md" FM-115 || return
  mkdir -p "$d/f116"; awk '{print} /^status: /{print "entered_via: vibes"}' "$base" > "$d/f116/spec.md"
  expect_one_rule "t09 FM-116" "$d/f116/spec.md" FM-116 || return
  ok t09_optional_status_reason_enums
}

want t01 && t01_cli_and_determinism
want t02 && t02_fm_family
want t03 && t03_sec_family
want t04 && t04_cond_family
want t05 && t05_trace_family
want t06 && t06_green_corpus
want t07 && t07_payload_and_install
want t08 && t08_skill_wiring_present
want t09 && t09_optional_status_reason_enums

echo "test_task_lint: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
