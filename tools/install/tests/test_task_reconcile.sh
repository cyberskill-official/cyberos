#!/usr/bin/env bash
# test_task_reconcile.sh - TASK-IMP-100 §2 suite (t01-t05 -> AC 1-5; AC 6 is a recorded
# SKILL.md grep in the gate log).
#
# The tool judges the state ship-tasks cannot vouch for: work it did not perform. Each
# scenario builds a scratch git repo whose task folder is shaped like a real corpus entry
# (spec + audit + artefacts + committed deliverables) and bends exactly one thing.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TR="$repo/tools/install/docs-tools/task-reconcile.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
rec_of() { node "$1" "$2" --repo "$3" ${4:-} --json 2>"$TMP/err" | python3 -c 'import json,sys; print(json.load(sys.stdin)["recommendation"])'; }
rung_of() { node "$1" "$2" --repo "$3" ${4:-} --json 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['rungs']['$5'])"; }

# fixture <dir> <status> <artefacts:yes|no> <deliverable:committed|dirty|none> <suite:pass|fail|missing>
fixture() {
  local d="$1" status="$2" arte="$3" deliv="$4" suite="$5"
  local t="$d/docs/tasks/demo/TASK-DEMO-001-thing"
  mkdir -p "$t" "$d/docs/tasks/.workflow" "$d/src" "$d/tests"
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t )
  # deliverable
  case "$deliv" in
    committed|dirty) echo "export const x = 1;" > "$d/src/thing.ts" ;;
  esac
  # cited suite
  case "$suite" in
    pass) printf '#!/usr/bin/env bash\nexit 0\n' > "$d/tests/thing.sh"; chmod +x "$d/tests/thing.sh" ;;
    fail) printf '#!/usr/bin/env bash\nexit 1\n' > "$d/tests/thing.sh"; chmod +x "$d/tests/thing.sh" ;;
  esac
  local cited="tests/thing.sh"; [ "$suite" = "missing" ] && cited="tests/gone.sh"
  cat > "$t/spec.md" <<SPEC
---
id: TASK-DEMO-001
title: demo thing
template: task@1
type: improvement
status: $status
priority: p2
new_files:
  - src/thing.ts
modified_files: []
---

# TASK-DEMO-001: demo thing

## 1. Description (normative)

- 1.1 The thing MUST exist.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - the thing exists - test: \`$cited::t01\`
SPEC
  if [ "$arte" = "yes" ]; then
    for a in context-map.md edge-case-matrix.md impl-plan.md obs-injection.md code-review.md coverage-gate.md; do
      echo "# $a for TASK-DEMO-001" > "$t/$a"
    done
  fi
  # audit bound to the spec's CURRENT bytes, committed together (the honest binding)
  local sha; sha="$(sha256sum "$t/spec.md" | cut -c1-16)"
  cat > "$t/audit.md" <<AUDIT
---
audited_file: "docs/tasks/demo/TASK-DEMO-001-thing/spec.md"
audited_file_sha256_prefix: "$sha"
overall_status: "pass"
---
# audit
AUDIT
  ( cd "$d" && git add -A && git commit -qm "fixture: TASK-DEMO-001 at $status" )
  [ "$deliv" = "dirty" ] && ( cd "$d" && git rm -q --cached src/thing.ts && git commit -qm "uncommit the deliverable" && echo "export const x = 1;" > src/thing.ts )
  [ "$deliv" = "none" ] && rm -f "$d/src/thing.ts"
  return 0
}

# ── t01: every rung supports the claim -> resume_at_phase ────────────────────
t01_clean_resume() {
  local d="$TMP/t01"; fixture "$d" reviewing yes committed pass
  local r; r="$(rec_of "$TR" TASK-DEMO-001 "$d" --run-tests)"
  case "$r" in resume_at_phase*) ;; *) fail t01_clean_resume "recommendation=$r ($(cat "$TMP/err"))"; return;; esac
  for g in r1 r2 r4 r5; do
    [ "$(rung_of "$TR" TASK-DEMO-001 "$d" --run-tests "$g")" = "pass" ] || { fail t01_clean_resume "$g not pass"; return; }
  done
  ok t01_clean_resume
}

# ── t02: claims unsupported (no artefacts + failing cited suite) -> route_back ─
t02_route_back() {
  local d="$TMP/t02"; fixture "$d" testing no committed fail
  local r; r="$(rec_of "$TR" TASK-DEMO-001 "$d" --run-tests)"
  [ "$r" = "route_back" ] || { fail t02_route_back "recommendation=$r"; return; }
  local why; why="$(node "$TR" TASK-DEMO-001 --repo "$d" --run-tests 2>/dev/null)"
  grep -q "R5 cited tests" <<<"$why" || { fail t02_route_back "route_back did not name the failing-suite rung"; return; }
  # the uncommitted-claim arm: deliverable on disk, no commit carries it (TASK-IMP-086 class)
  local e="$TMP/t02b"; fixture "$e" reviewing yes dirty pass
  [ "$(rec_of "$TR" TASK-DEMO-001 "$e" --run-tests)" = "route_back" ] || { fail t02_route_back "uncommitted claim did not route back"; return; }
  node "$TR" TASK-DEMO-001 --repo "$e" 2>/dev/null | grep -q "UNCOMMITTED CLAIM" \
    || { fail t02_route_back "uncommitted claim not named in the report"; return; }
  ok t02_route_back
}

# ── t03: sound at HEAD, artefacts absent -> adopt_candidate ──────────────────
t03_adopt_candidate() {
  local d="$TMP/t03"; fixture "$d" reviewing no committed pass
  local r; r="$(rec_of "$TR" TASK-DEMO-001 "$d" --run-tests)"
  [ "$r" = "adopt_candidate" ] || { fail t03_adopt_candidate "recommendation=$r"; return; }
  # the other artefact home satisfies the set: a .workflow bundle naming the phase artefacts
  mkdir -p "$d/docs/tasks/.workflow/TASK-DEMO-001"
  printf '# phase-bundle\ncontext map, edge case matrix, impl plan, obs injection, code review\n' \
    > "$d/docs/tasks/.workflow/TASK-DEMO-001/phase-bundle.md"
  ( cd "$d" && git add -A && git commit -qm "bundle in the .workflow home" )
  case "$(rec_of "$TR" TASK-DEMO-001 "$d" --run-tests)" in
    resume_at_phase*) ok t03_adopt_candidate ;;
    *) fail t03_adopt_candidate "the .workflow bundle home was not accepted" ;;
  esac
}

# ── t04: read-only + spec drift + not_applicable ─────────────────────────────
t04_read_only_and_spec_drift() {
  local d="$TMP/t04"; fixture "$d" reviewing yes committed pass
  local before; before="$(cd "$d" && find . -path ./.git -prune -o -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum)"
  node "$TR" TASK-DEMO-001 --repo "$d" --run-tests >/dev/null 2>&1
  node "$TR" TASK-DEMO-001 --repo "$d" --json >/dev/null 2>&1
  local after; after="$(cd "$d" && find . -path ./.git -prune -o -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum)"
  [ "$before" = "$after" ] || { fail t04_read_only_and_spec_drift "the tool mutated the tree"; return; }
  # normative drift: a clause edited AFTER the audit commit
  local t="$d/docs/tasks/demo/TASK-DEMO-001-thing"
  sed -i 's/- 1.1 The thing MUST exist./- 1.1 The thing MUST exist and MUST be blue./' "$t/spec.md"
  ( cd "$d" && git add -A && git commit -qm "edit a clause after the audit" )
  [ "$(rec_of "$TR" TASK-DEMO-001 "$d")" = "route_back" ] || { fail t04_read_only_and_spec_drift "normative drift did not route back"; return; }
  node "$TR" TASK-DEMO-001 --repo "$d" 2>/dev/null | grep -q "SPEC DRIFT" || { fail t04_read_only_and_spec_drift "drift not named"; return; }
  # lifecycle-only churn is NOT drift (the flip the workflow itself performs)
  local e="$TMP/t04b"; fixture "$e" reviewing yes committed pass
  sed -i 's/^status: reviewing$/status: done/' "$e/docs/tasks/demo/TASK-DEMO-001-thing/spec.md"
  ( cd "$e" && git add -A && git commit -qm "lifecycle flip only" )
  [ "$(rung_of "$TR" TASK-DEMO-001 "$e" "" r1)" = "pass" ] || { fail t04_read_only_and_spec_drift "lifecycle churn was misread as drift"; return; }
  # not_applicable
  local f="$TMP/t04c"; fixture "$f" ready_to_implement no none missing
  [ "$(rec_of "$TR" TASK-DEMO-001 "$f")" = "not_applicable" ] || { fail t04_read_only_and_spec_drift "ready_to_implement was not not_applicable"; return; }
  ok t04_read_only_and_spec_drift
}

# ── t05: payload vendoring ───────────────────────────────────────────────────
t05_payload_vendored() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 \
    || { fail t05_payload_vendored "build.sh failed"; return; }
  local p="$TMP/payload/docs-tools/task-reconcile.mjs"
  [ -s "$p" ] || { fail t05_payload_vendored "not vendored"; return; }
  cmp -s "$p" "$TR" || { fail t05_payload_vendored "payload copy differs from source"; return; }
  local d="$TMP/t05"; fixture "$d" reviewing yes committed pass
  case "$(rec_of "$p" TASK-DEMO-001 "$d" --run-tests)" in
    resume_at_phase*) ok t05_payload_vendored ;;
    *) fail t05_payload_vendored "the payload copy does not run" ;;
  esac
}

# ── t06: the body binding is preferred and lifecycle-proof (TASK-IMP-102) ───
# The convention this arm guards exists because the ladder red-flagged a correctly-shipped
# task: audits bound whole-file bytes that ship-tasks rewrites (and that no commit carried).
body_sha() {   # normative half: body + frontmatter minus the lifecycle-mutable fields
  python3 - "$1" <<'PYB'
import hashlib, sys
L = open(sys.argv[1]).read().split("\n")
end = [i for i, l in enumerate(L) if i > 0 and l.strip() == "---"][0]
keep = [l for l in L[1:end] if not any(l.startswith(f + ":") for f in ("status", "shipped", "routed_back_count", "memory_chain_hash"))]
print(hashlib.sha256("\n".join(keep + L[end + 1:]).encode()).hexdigest()[:16])
PYB
}
t06_body_binding_preferred() {
  local d="$TMP/t06"; fixture "$d" reviewing yes committed pass
  local t="$d/docs/tasks/demo/TASK-DEMO-001-thing"
  # re-audit with the body field, then flip status like the workflow does
  local b; b="$(body_sha "$t/spec.md")"
  printf -- '---\naudited_file_sha256_prefix: "deadbeefdeadbeef"\naudited_body_sha256_prefix: "%s"\noverall_status: "pass"\n---\n# audit\n' "$b" > "$t/audit.md"
  sed -i 's/^status: reviewing$/status: done/' "$t/spec.md"
  ( cd "$d" && git add -A && git commit -qm "body-bound audit + lifecycle flip" )
  [ "$(rung_of "$TR" TASK-DEMO-001 "$d" "" r1)" = "pass" ] \
    || { fail t06 "body binding did not survive a lifecycle flip"; return; }
  node "$TR" TASK-DEMO-001 --repo "$d" 2>/dev/null | grep -q "binding gap" \
    && { fail t06 "body-bound audit still reported a binding gap"; return; }
  # a clause edit is real drift
  sed -i 's/- 1.1 The thing MUST exist./- 1.1 The thing MUST exist and MUST be blue./' "$t/spec.md"
  ( cd "$d" && git add -A && git commit -qm "clause edit after the audit" )
  [ "$(rung_of "$TR" TASK-DEMO-001 "$d" "" r1)" = "red" ] || { fail t06 "normative edit was not caught"; return; }
  node "$TR" TASK-DEMO-001 --repo "$d" 2>/dev/null | grep -q "SPEC DRIFT" || { fail t06 "drift not named"; return; }
  # legacy audit (no body field) still resolves via the audit commit, gap named as legacy
  local e="$TMP/t06b"; fixture "$e" reviewing yes committed pass
  sed -i 's/^status: reviewing$/status: done/' "$e/docs/tasks/demo/TASK-DEMO-001-thing/spec.md"
  ( cd "$e" && git add -A && git commit -qm "legacy audit + flip" )
  node "$TR" TASK-DEMO-001 --repo "$e" 2>/dev/null | grep -q "via the audit commit" \
    || { fail t06 "legacy audit did not resolve through the audit-commit path"; return; }
  [ "$(rung_of "$TR" TASK-DEMO-001 "$e" "" r1)" = "pass" ] || { fail t06 "legacy lifecycle churn misread as drift"; return; }
  # legacy AND dishonest (the corpus's real shape: sha recorded pre-flip, so no commit carries
  # those bytes) -> the gap is named as legacy, and the substantive check still runs
  local f="$TMP/t06c"; fixture "$f" reviewing yes committed pass
  local ft="$f/docs/tasks/demo/TASK-DEMO-001-thing"
  printf -- '---\naudited_file_sha256_prefix: "beefbeefbeefbeef"\noverall_status: "pass"\n---\n# audit\n' > "$ft/audit.md"
  ( cd "$f" && git add -A && git commit -qm "legacy audit bound to bytes no commit carries" )
  node "$TR" TASK-DEMO-001 --repo "$f" 2>/dev/null | grep -q "legacy audit: no audited_body_sha256_prefix" \
    || { fail t06 "the legacy binding gap did not name itself"; return; }
  [ "$(rung_of "$TR" TASK-DEMO-001 "$f" "" r1)" = "pass" ] \
    || { fail t06 "a binding gap was upgraded into a verdict (it is a note)"; return; }
  ok t06_body_binding_preferred
}

echo "task-reconcile suite (TASK-IMP-100):"
t01_clean_resume
t02_route_back
t03_adopt_candidate
t04_read_only_and_spec_drift
t05_payload_vendored
t06_body_binding_preferred
echo "test_task_reconcile: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
