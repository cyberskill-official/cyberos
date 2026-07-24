#!/usr/bin/env bash
# test_entrypoint_identity.sh — TASK-IMP-138 Branch A invariants (t01–t06).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
TASKDIR="$repo/docs/tasks/improvement/TASK-IMP-138-entrypoint-identity-fork"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "test_entrypoint_identity.sh (TASK-IMP-138 Branch A)"

t01_decision_recorded() {
  local spec="$TASKDIR/spec.md" dec="$TASKDIR/decision-branch-a.md"
  [ -f "$dec" ] || { fail t01 "missing decision-branch-a.md"; return; }
  grep -qE 'Branch A|thin spine' "$spec" && grep -qE '2026-07-23' "$spec" \
    && ok t01 || fail t01 "Branch A decision not recorded on spec"
}

t02_first_screen_reaches_task_law() {
  local head30
  head30="$(head -30 "$repo/AGENTS.md")"
  echo "$head30" | grep -q '\.cyberos/AGENT-ENTRY\.md' \
    && ok t02 || fail t02 "first 30 lines of AGENTS.md missing AGENT-ENTRY"
}

t03_pointers_truthful() {
  local f all=1
  for f in CLAUDE.md .cursorrules .cursor/rules/cyberos.mdc GEMINI.md \
           .github/copilot-instructions.md .windsurfrules; do
    [ -f "$repo/$f" ] || { fail t03 "missing $f"; all=0; continue; }
    grep -q '\.cyberos/AGENT-ENTRY\.md' "$repo/$f" \
      || { fail t03 "$f does not name AGENT-ENTRY"; all=0; }
  done
  # Branch A: unqualified "Canonical instructions: AGENTS.md (root)" must not survive
  if grep -RInE 'Canonical instructions: AGENTS\.md \(root\)' \
       "$repo/.cursorrules" "$repo/.cursor/rules/cyberos.mdc" "$repo/GEMINI.md" \
       "$repo/.github/copilot-instructions.md" "$repo/.windsurfrules" \
       "$repo/CLAUDE.md" 2>/dev/null | grep -q .; then
    fail t03 "unqualified Canonical instructions: AGENTS.md (root) still present"; all=0
  fi
  [ "$all" -eq 1 ] && ok t03
}

t04_single_normative_source() {
  # Exactly one Layer-1 H1 without a copy/pointer marker among known homes.
  local norm=0
  local f
  for f in "$repo/modules/memory/cyberos/data/AGENTS.md" \
           "$repo/.cyberos/memory/AGENTS.md" \
           "$repo/AGENTS.md" \
           "$repo/CLAUDE.md"; do
    [ -f "$f" ] || continue
    if head -1 "$f" | grep -q 'Layer-1 Memory Protocol'; then
      if grep -qiE 'copy of|pointer|do not treat this file as a second copy|thin (CyberOS )?spine|thin pointer' "$f"; then
        continue
      fi
      # installed copy may be unmarked duplicate of normative — count only module data as normative
      if [[ "$f" == *"modules/memory/cyberos/data/AGENTS.md" ]]; then
        norm=$((norm+1))
      elif [[ "$f" == *".cyberos/memory/AGENTS.md" ]]; then
        # installed copy is allowed as marked OR as byte-copy of normative; require it starts with protocol H1
        : # not counted as second normative
      else
        fail t04 "unexpected unmarked protocol at $f"; return
      fi
    fi
  done
  [ "$norm" -eq 1 ] || { fail t04 "expected exactly 1 normative protocol file, got $norm"; return; }
  # Root + CLAUDE must NOT be protocol dumps
  head -1 "$repo/AGENTS.md" | grep -q 'Layer-1 Memory Protocol' \
    && { fail t04 "root AGENTS.md still Layer-1 protocol"; return; }
  head -1 "$repo/CLAUDE.md" | grep -q 'Layer-1 Memory Protocol' \
    && { fail t04 "CLAUDE.md still Layer-1 protocol"; return; }
  ok t04
}

t05_branch_consistency() {
  # Branch A: no platform AGENTS exception; no "Platform monorepo exception" comment
  if grep -q 'Platform monorepo exception' "$repo/tools/install/install.sh"; then
    fail t05 "install.sh still carries Platform monorepo exception comment"; return
  fi
  if grep -q 'kept platform AGENTS.md' "$repo/tools/install/install.sh"; then
    fail t05 "install.sh still keeps platform AGENTS.md as protocol source"; return
  fi
  # Stale root-protocol assumption sweep (narrow): installer must not claim root is normative protocol
  if grep -qE 'root AGENTS\.md is the normative Layer-1 protocol source' "$repo/tools/install/install.sh"; then
    fail t05 "install.sh still claims root AGENTS.md is normative protocol"; return
  fi
  ok t05
}

t06_registered_and_recorded() {
  [ -f "$here/test_entrypoint_identity.sh" ] || { fail t06 "suite missing"; return; }
  local top
  # Scan every versioned ## […] section — top entry moves with each cut (same class as CUO doctrine pin).
  top="$(awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md")"
  echo "$top" | grep -qiE 'Branch A|IMP-138|entrypoint' \
    && ok t06 || fail t06 "CHANGELOG versioned entry does not name Branch A / IMP-138"
}

t01_decision_recorded
t02_first_screen_reaches_task_law
t03_pointers_truthful
t04_single_normative_source
t05_branch_consistency
t06_registered_and_recorded

echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
