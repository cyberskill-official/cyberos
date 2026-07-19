#!/usr/bin/env bash
# test_e2e_skeleton.sh — TASK-IMP-107: end-to-end MECHANICAL smoke test of the ship-tasks spine.
#
# WHAT THIS IS. Twenty-five suites test the helpers in isolation and the workflow doctrine as
# prose; none drives the whole MECHANICAL spine — install → lint → insert → every lifecycle
# flip → coverage-scope → reconcile → uninstall — on one throwaway repo. This is the seam test.
# It runs with NO model, NO network, NO credentials: only repo-tracked helpers with fixed args.
#
#   t01_spine_green                       (AC1, traces §1.2 + §1.5) the full ordered spine runs
#        green on a scratch repo, offline, and the spine finishes under the 30s target.
#   t02_reconcile_recommendation_asserted (AC2, traces §1.3) a GREEN reconcile state recommends
#        resume_at_phase(23); ONE controlled drift (an uncommitted claimed file — the
#        TASK-IMP-086 class) flips it to route_back. Two assertions differing by one committed
#        object: reconcile CANNOT satisfy both unless it reads the state, so this is not
#        "merely exit 0" (the defect §1.3 forbids).
#   t03_corpus_survives_uninstall         (AC3, traces §1.4) the fixture spec + BACKLOG row are
#        present AND byte-identical (sha256) after uninstall. Construction check: uninstall
#        removed the MACHINE (default keeps the BRAIN) — otherwise "survives" is vacuous.
#   t04_scratch_isolation                 (AC4, traces §1.1) a full mutating mini-spine in a
#        scratch leaves the WORKING repo's tracked docs/tasks content byte-identical.
#   t05_index_first_flip_refuses          (TASK-IMP-120 clause 1.6 / AC7 — this suite's cone) an
#        index-first flip (the truth still lagging) REFUSES with exit 6 and does NOT move the row;
#        the same flip proceeds once the truth is written first. Positively verifies the
#        truth-precedes-index contract the spine's own flips (t01/t04) now depend on.
#   (AC5 §1.6: run_all.sh discovers this file via its tools/install/tests/test_*.sh glob.)
#
# WHY BUILD A FRESH PAYLOAD (not dist/cyberos/install.sh). dist/ is gitignored
# (`git check-ignore dist/cyberos` succeeds) so a fresh CI checkout is not guaranteed to carry
# it, and the committed dist can lag source (its uninstall.sh predated TASK-IMP-106 while
# manifest.rules_sha still matched, because rules_sha fingerprints only cuo/plugin/mcp/cli/
# memory — not the install scripts). Building from source into $TMP is what the sibling suite
# test_install_hygiene.sh does, tests CURRENT source every run, and never touches dist/.
#
# THE FLIP IS TWO WRITES, TRUTH FIRST (the crux). `backlog-mutate flip` rewrites ONLY the
# BACKLOG.md row — it never touches the spec.md frontmatter status cell (index-only; the truth is
# the separate write, and having flip write the spec was explicitly rejected — see the
# backlog-state-update-author SKILL and TASK-IMP-120's Alternatives). A real lifecycle transition
# is TWO writes: the truth (frontmatter) AND the index (backlog-mutate). Since TASK-IMP-120 the
# ORDER is enforced — flip reads the spec frontmatter FIRST and REFUSES (exit 6) unless it ALREADY
# carries <to>: the index may only catch up to a truth that already moved, never lead it. So this
# suite writes the truth FIRST (write_spec) then flips the index, per status, and asserts the two
# AGREE after each flip and at `done`; t05 proves the forbidden order (index first, truth lagging)
# refuses. Driving the truth first is also what lets t01/t02's reconcile read the status intended.
#
# SEAM DISCIPLINE. If a helper legitimately changes output shape, or STATUS-REFERENCE §1.1 grows
# a lifecycle status, this suite WILL fail — deliberately (spec §3). The fix is to update the
# assertion here, never to loosen it. git/node absent → SKIP with a named reason, never a fail.
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"

# Skip discipline (spec §3, same shape as test_release_assets.sh): the spine git-inits a scratch
# and drives node helpers. Absent → SKIP with a reason (run_all.sh counts it as skip, not pass).
command -v git  >/dev/null 2>&1 || { echo "  SKIP test_e2e_skeleton.sh — git not on PATH (the spine git-inits a scratch repo)"; exit 0; }
command -v node >/dev/null 2>&1 || { echo "  SKIP test_e2e_skeleton.sh — node not on PATH (the docs-tools helpers are node)"; exit 0; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# §1.1 scratch discipline: scratch dirs live OUTSIDE the working repo (mktemp → $TMPDIR), so a
# run creates AND removes them (the repo mount denies unlink). If mktemp ever lands under the
# repo the whole premise is broken, so assert it once, loudly, before anything writes.
case "$TMP/" in "$repo"/*) echo "FATAL scratch $TMP is under the repo $repo"; exit 1 ;; esac

echo "building scratch payload (fresh from source; dist/ is gitignored and can lag — see header)..."
PAY="$TMP/payload"
bash "$repo/tools/install/build.sh" "$PAY" >/dev/null 2>&1 || { echo "FATAL build"; exit 1; }

ID=TASK-SMOKE-001
STEM=TASK-SMOKE-001-e2e-fixture
# The linear lifecycle, in order, from STATUS-REFERENCE.md §1.1. THIS LIST IS THE SEAM: if §1.1
# gains a status, this suite must gain it too (a deliberate break, not a silent gap).
LIFECYCLE="ready_to_implement implementing ready_to_review reviewing ready_to_test testing done"

mkrepo()      { mkdir -p "$1"; ( cd "$1" && git init -q . && git config user.email e2e@test && git config user.name e2e ); }
install_into() { bash "$PAY/install.sh" "$1" >/dev/null 2>&1; }

# A task-lint-clean task@1 fixture spec at a given status. Rewriting it IS the truth-half of a
# lifecycle flip (the index-half is backlog-mutate). modified_files names the object reconcile
# R4 measures against HEAD.
write_spec() { # $1=specfile  $2=status
  cat > "$1" <<EOF
---
template: task@1
title: e2e mechanical smoke fixture
author: "@smoke"
department: engineering
status: $2
priority: p1
created_at: 2026-07-17T00:00:00Z
ai_authorship: none
type: chore
eu_ai_act_risk_class: not_ai
client_visible: false
new_files: []
modified_files: [src/foo.txt]
---

# TASK-SMOKE-001: e2e mechanical smoke fixture

## Summary
Fixture task exercising the mechanical spine.

## Problem
The seam between the helpers is untested end to end.

## Proposed Solution
Drive install, lint, insert, flips, coverage-scope, reconcile, uninstall.

## Alternatives Considered
A model-in-the-loop test; rejected as non-deterministic.

## Success Metrics
The spine runs green, offline, under the 30s target.

## Scope
This fixture only; no product code.

## Dependencies
None.
EOF
}

# A minimal BACKLOG.md with a section whose only row is the exact placeholder backlog-mutate
# replaces on first insert — so insert lands deterministically without touching install's own
# template shape.
seed_backlog() { # $1=backlogfile
  cat > "$1" <<'BL'
# scratch backlog

## Smoke

- (nothing remaining)
BL
}

spec_status() { sed -n 's/^status: //p' "$1" | head -1; }               # frontmatter truth
row_status()  { sed -n "s/^- \[\([a-z_]*\)\] $STEM .*/\1/p" "$1" | head -1; }  # BACKLOG index

# hash of every TRACKED file's content under the working repo's docs/tasks (gitignored session
# state — manifests, .DS_Store — is excluded on purpose, so this reflects only what a suite run
# could actually corrupt).
repo_tasks_fp() { ( cd "$repo" && git ls-files -z -- docs/tasks | sort -z | xargs -0 sha256sum 2>/dev/null | sha256sum | awk '{print $1}' ); }

STUB_COVERAGE='{"total":{"lines":{"total":1,"covered":1,"skipped":0,"pct":100}},"src/foo.txt":{"lines":{"total":1,"covered":1,"skipped":0,"pct":100}}}'

# ── AC1 (traces §1.2, §1.5): the full ordered spine, green, offline, under 30s ──────────────
t01_spine_green() {
  local all=1 d="$TMP/spine" t0=$SECONDS; mkrepo "$d"
  # §1.2 step 1 — install the built payload
  install_into "$d" || { fail t01_spine_green "install exited nonzero"; return; }
  { [ -f "$d/.cyberos/docs-tools/task-lint.mjs" ] && [ -f "$d/.cyberos/docs-tools/backlog-mutate.mjs" ] \
    && [ -f "$d/.cyberos/docs-tools/coverage-scope.mjs" ] && [ -f "$d/.cyberos/docs-tools/task-reconcile.mjs" ] \
    && [ -x "$d/.cyberos/uninstall.sh" ]; } || { fail t01_spine_green "payload helpers/uninstall not laid down"; return; }
  local TD="$d/docs/tasks/smoke/$STEM"; mkdir -p "$TD"; local spec="$TD/spec.md"

  # §1.2 step 2 — write a fixture task@1 spec (status draft)
  write_spec "$spec" draft

  # §1.2 step 3 — task-lint clean (exit 0, zero error-severity findings)
  local lint; lint="$(node "$d/.cyberos/docs-tools/task-lint.mjs" --json "$spec" 2>&1)"
  if [ $? -ne 0 ]; then fail t01_spine_green "task-lint did not exit 0: $(echo "$lint" | head -1)"; all=0; fi
  grep -q '"severity": "error"' <<<"$lint" && { fail t01_spine_green "task-lint reported an error finding on the fixture"; all=0; }

  # §1.2 step 4 — backlog-mutate insert
  seed_backlog "$d/docs/tasks/BACKLOG.md"
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" insert "$ID" "$STEM" "e2e mechanical smoke fixture" draft --section Smoke >/dev/null 2>&1 \
    || { fail t01_spine_green "backlog insert exited nonzero"; all=0; }
  [ "$(row_status "$d/docs/tasks/BACKLOG.md")" = draft ] || { fail t01_spine_green "row not present at draft after insert"; all=0; }

  # §1.2 step 5 — a flip through EVERY lifecycle status (STATUS-REFERENCE §1.1), driving BOTH
  # halves in the order TASK-IMP-120 enforces: the truth (frontmatter, via write_spec) FIRST, THEN
  # the index (backlog-mutate flip). The flip now SUCCEEDS precisely because the truth already
  # carries <next> — the index catches up to a truth that already moved. (Index-first is the
  # forbidden order the guard refuses; t05 proves that refusal directly.)
  local prev=draft next
  for next in $LIFECYCLE; do
    write_spec "$spec" "$next"   # truth-half FIRST — the frontmatter now carries <next>
    node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" flip "$ID" "$prev" "$next" >/dev/null 2>&1 \
      || { fail t01_spine_green "flip $prev -> $next exited nonzero (truth-first; the guard should permit it)"; all=0; break; }
    # the crux, proven inline: after a truth-first flip the index has CAUGHT UP to the truth —
    # both halves read <next> (flip moved the row; write_spec set the frontmatter; they agree).
    { [ "$(row_status "$d/docs/tasks/BACKLOG.md")" = "$next" ] && [ "$(spec_status "$spec")" = "$next" ]; } \
      || { fail t01_spine_green "flip $prev->$next left index/truth disagreeing (row=$(row_status "$d/docs/tasks/BACKLOG.md") truth=$(spec_status "$spec"))"; all=0; }
    prev="$next"
  done
  # both halves agree at the terminal status
  { [ "$(row_status "$d/docs/tasks/BACKLOG.md")" = done ] && [ "$(spec_status "$spec")" = done ]; } \
    || { fail t01_spine_green "index/truth disagree at end (row=$(row_status "$d/docs/tasks/BACKLOG.md") truth=$(spec_status "$spec"))"; all=0; }

  # §1.2 step 6 — coverage-scope against a stub report. Commit the corpus (base), then the one
  # claimed file (HEAD), so base...HEAD is exactly the touched file; the stub scores it 100%.
  git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -q --no-verify -m "base state" >/dev/null 2>&1
  local base; base="$(git -C "$d" rev-parse HEAD)"
  mkdir -p "$d/src"; printf 'hello\n' > "$d/src/foo.txt"
  git -C "$d" add src/foo.txt >/dev/null 2>&1; git -C "$d" commit -q --no-verify -m "TASK-SMOKE-001: enter implementing" >/dev/null 2>&1
  mkdir -p "$d/coverage"; printf '%s\n' "$STUB_COVERAGE" > "$d/coverage/coverage-summary.json"
  local cov; cov="$(node "$d/.cyberos/docs-tools/coverage-scope.mjs" "$ID" --base "$base" --coverage coverage/coverage-summary.json --repo "$d" 2>&1)"
  if [ $? -ne 0 ]; then fail t01_spine_green "coverage-scope exited nonzero: $(echo "$cov" | head -1)"; all=0; fi
  grep -qF 'artefact: coverage-gate@1' <<<"$cov" || { fail t01_spine_green "coverage-scope did not emit a coverage-gate@1 skeleton"; all=0; }

  # §1.2 step 7 — task-reconcile. State = done with no audit.md, so R1 (spec integrity) reds and
  # the recommendation is route_back deterministically. Asserting the VALUE (not just exit 0)
  # keeps this step honest; the dedicated recommendation-logic proof is t02.
  local rec; rec="$(node "$d/.cyberos/docs-tools/task-reconcile.mjs" "$ID" --repo "$d" --json 2>&1)"
  if [ $? -ne 0 ]; then fail t01_spine_green "task-reconcile exited nonzero: $(echo "$rec" | head -1)"; all=0; fi
  grep -qF '"artefact": "reconcile-report@1"' <<<"$rec" || { fail t01_spine_green "reconcile did not emit reconcile-report@1"; all=0; }
  grep -qF '"recommendation": "route_back"' <<<"$rec" \
    || { fail t01_spine_green "reconcile recommendation not route_back for a done-with-no-audit state: $(grep recommendation <<<"$rec")"; all=0; }

  # §1.2 step 8 — uninstall (byte-survival of the corpus is t03's job; here just complete green).
  # Default uninstall removes the MACHINE but keeps the BRAIN (.cyberos/memory/store), so assert
  # the machine is gone rather than the whole .cyberos/.
  bash "$d/.cyberos/uninstall.sh" "$d" >/dev/null 2>&1 || { fail t01_spine_green "uninstall exited nonzero"; all=0; }
  { [ ! -e "$d/.cyberos/docs-tools" ] && [ ! -e "$d/.cyberos/uninstall.sh" ]; } \
    || { fail t01_spine_green "uninstall did not remove the machine (.cyberos/docs-tools or uninstall.sh remain)"; all=0; }

  # §1.5 — the spine finished under the 30s sandbox target. Suite-asserted every run.
  local dur=$((SECONDS - t0))
  [ "$dur" -lt 30 ] || { fail t01_spine_green "spine took ${dur}s (>= 30s, spec §1.5)"; all=0; }

  [ "$all" -eq 1 ] && ok "t01_spine_green (spine ${dur}s)"
}

# ── AC2 (traces §1.3): reconcile's RECOMMENDATION matches the constructed state ─────────────
# Build a GREEN state at `testing` (audit.md pass, the six cumulative artefacts, the claimed
# file committed) → resume_at_phase(23). Then make ONE change — uncommit the claimed file — and
# the recommendation MUST become route_back. A reconcile that ignored the state could not pass
# both assertions, which is exactly what "not merely exit 0" means.
t02_reconcile_recommendation_asserted() {
  local all=1 d="$TMP/rec"; mkrepo "$d"
  install_into "$d" || { fail t02_reconcile_recommendation_asserted "install exited nonzero"; return; }
  local TD="$d/docs/tasks/smoke/$STEM"; mkdir -p "$TD"; local spec="$TD/spec.md"
  write_spec "$spec" testing
  # R1: a synthetic audit that passes. overall_status pass + no sha prefix → the binding reads
  # "unverifiable" (a finding, not a red) → R1 passes. This is a legitimate audit.md shape.
  cat > "$TD/audit.md" <<'AUD'
---
artefact: task-audit@2.0
task: TASK-SMOKE-001
overall_status: pass
---
# Synthetic audit (fixture)
overall_status: pass
AUD
  # R2: the cumulative artefact set for `testing`.
  local a; for a in context-map edge-case-matrix impl-plan obs-injection code-review coverage-gate; do
    printf '# %s (fixture stub)\n' "$a" > "$TD/$a.md"
  done
  # R4: commit the claimed modified_files path so HEAD carries it.
  mkdir -p "$d/src"; printf 'hi\n' > "$d/src/foo.txt"
  git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -q --no-verify -m "base with src/foo.txt" >/dev/null 2>&1

  local green; green="$(node "$d/.cyberos/docs-tools/task-reconcile.mjs" "$ID" --repo "$d" --json 2>&1 | sed -n 's/.*"recommendation": "\(.*\)".*/\1/p')"
  [ "$green" = "resume_at_phase(23)" ] \
    || { fail t02_reconcile_recommendation_asserted "green state expected resume_at_phase(23), got '$green'"; all=0; }

  # ONE controlled drift: drop the claimed file from HEAD but leave it on disk (TASK-IMP-086
  # class — a claim no commit carries). R4 reds → route_back.
  git -C "$d" rm --cached src/foo.txt >/dev/null 2>&1; git -C "$d" commit -q --no-verify -m "drop src/foo.txt from index" >/dev/null 2>&1
  local drift; drift="$(node "$d/.cyberos/docs-tools/task-reconcile.mjs" "$ID" --repo "$d" --json 2>&1 | sed -n 's/.*"recommendation": "\(.*\)".*/\1/p')"
  [ "$drift" = "route_back" ] \
    || { fail t02_reconcile_recommendation_asserted "drifted state expected route_back, got '$drift'"; all=0; }

  # construction check: the single change actually MOVED the recommendation. If green == drift
  # the pair proves nothing — the recommendation would be independent of the state under test.
  [ "$green" != "$drift" ] \
    || { fail t02_reconcile_recommendation_asserted "recommendation did not change with the state ('$green' both times)"; all=0; }

  [ "$all" -eq 1 ] && ok t02_reconcile_recommendation_asserted
}

# ── AC3 (traces §1.4): the fixture corpus survives uninstall, byte-identical ────────────────
t03_corpus_survives_uninstall() {
  local all=1 d="$TMP/surv"; mkrepo "$d"
  install_into "$d" || { fail t03_corpus_survives_uninstall "install exited nonzero"; return; }
  local TD="$d/docs/tasks/smoke/$STEM"; mkdir -p "$TD"; local spec="$TD/spec.md" bl="$d/docs/tasks/BACKLOG.md"
  write_spec "$spec" draft
  seed_backlog "$bl"
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" insert "$ID" "$STEM" "e2e mechanical smoke fixture" draft --section Smoke >/dev/null 2>&1 \
    || { fail t03_corpus_survives_uninstall "backlog insert exited nonzero"; return; }
  local before_spec before_bl
  before_spec="$(sha256sum "$spec" | awk '{print $1}')"
  before_bl="$(sha256sum "$bl" | awk '{print $1}')"

  bash "$d/.cyberos/uninstall.sh" "$d" >/dev/null 2>&1 || { fail t03_corpus_survives_uninstall "uninstall exited nonzero"; all=0; }
  # construction check: uninstall REMOVED the machine (default keeps the BRAIN at
  # .cyberos/memory/store, so check the machine's helpers are gone, not the whole .cyberos/), so
  # "corpus survived" is a real result and not the trivial truth that uninstall did nothing.
  [ ! -e "$d/.cyberos/docs-tools" ] || { fail t03_corpus_survives_uninstall "machine not removed (.cyberos/docs-tools remains) — survival would be vacuous"; all=0; }

  { [ -f "$spec" ] && [ -f "$bl" ]; } || { fail t03_corpus_survives_uninstall "corpus file missing after uninstall"; all=0; }
  [ "$(sha256sum "$spec" | awk '{print $1}')" = "$before_spec" ] \
    || { fail t03_corpus_survives_uninstall "spec.md changed by uninstall (not byte-identical)"; all=0; }
  [ "$(sha256sum "$bl" | awk '{print $1}')" = "$before_bl" ] \
    || { fail t03_corpus_survives_uninstall "BACKLOG.md changed by uninstall (not byte-identical)"; all=0; }

  [ "$all" -eq 1 ] && ok t03_corpus_survives_uninstall
}

# ── AC4 (traces §1.1): a suite run does not touch the working repo's docs/tasks ─────────────
t04_scratch_isolation() {
  local all=1 d="$TMP/iso"
  # structural: every scratch this suite creates is outside the repo (also gate-asserted above).
  case "$d/" in "$repo"/*) fail t04_scratch_isolation "scratch $d resolves under the repo"; return ;; esac
  local before after; before="$(repo_tasks_fp)"

  # a representative MUTATING mini-spine, entirely under --root/--repo the scratch — the exact
  # discipline that keeps a helper's root-walk (backlog-mutate/coverage-scope/reconcile findRoot)
  # from ever climbing into the working repo.
  mkrepo "$d"; install_into "$d" || { fail t04_scratch_isolation "install exited nonzero"; return; }
  local TD="$d/docs/tasks/smoke/$STEM"; mkdir -p "$TD"; write_spec "$TD/spec.md" draft
  seed_backlog "$d/docs/tasks/BACKLOG.md"
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" insert "$ID" "$STEM" "e2e mechanical smoke fixture" draft --section Smoke >/dev/null 2>&1
  write_spec "$TD/spec.md" implementing   # truth FIRST (TASK-IMP-120): an index-first flip would refuse
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" flip "$ID" draft implementing >/dev/null 2>&1
  git -C "$d" add -A >/dev/null 2>&1; git -C "$d" commit -q --no-verify -m base >/dev/null 2>&1
  node "$d/.cyberos/docs-tools/task-reconcile.mjs" "$ID" --repo "$d" --json >/dev/null 2>&1
  bash "$d/.cyberos/uninstall.sh" "$d" >/dev/null 2>&1

  after="$(repo_tasks_fp)"
  [ -n "$before" ] || { fail t04_scratch_isolation "could not fingerprint the working repo docs/tasks"; all=0; }
  [ "$before" = "$after" ] \
    || { fail t04_scratch_isolation "the working repo's docs/tasks changed during a suite run"; all=0; }

  [ "$all" -eq 1 ] && ok t04_scratch_isolation
}

# ── TASK-IMP-120 guard (this suite's cone, clause 1.6 / AC7): an index-first flip REFUSES ───────
# The seam's flip is index-only — it never writes the spec frontmatter (that stays the truth-half;
# having flip write the spec was explicitly rejected). Since TASK-IMP-120 the flip ALSO refuses to
# move the index unless the frontmatter ALREADY carries <to>: the index may only catch up to the
# truth, never lead it. This verifies that contract POSITIVELY — an index-first flip (truth still
# lagging) exits 6, names the contract, and leaves the row unmoved; the SAME flip then proceeds once
# the truth is written first. t01/t04 drive their flips truth-first (so they merely rely on the
# guard's permit path); this test is where the suite asserts the guard actually BITES the forbidden
# order — the exact index-first, truth-lagging shape TASK-IMP-120 exists to forbid.
t05_index_first_flip_refuses() {
  local all=1 d="$TMP/guard"; mkrepo "$d"
  install_into "$d" || { fail t05_index_first_flip_refuses "install exited nonzero"; return; }
  local TD="$d/docs/tasks/smoke/$STEM"; mkdir -p "$TD"; local spec="$TD/spec.md" bl="$d/docs/tasks/BACKLOG.md"
  write_spec "$spec" draft                        # the truth sits at draft
  seed_backlog "$bl"
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" insert "$ID" "$STEM" "e2e mechanical smoke fixture" draft --section Smoke >/dev/null 2>&1 \
    || { fail t05_index_first_flip_refuses "backlog insert exited nonzero"; return; }
  [ "$(row_status "$bl")" = draft ] || { fail t05_index_first_flip_refuses "row not at draft after insert"; return; }

  # INDEX-FIRST — the forbidden order: leave the truth at draft and try to move the index to
  # ready_to_implement. The guard MUST refuse (exit 6), name the contract, and NOT move the row.
  local out rc
  out="$(node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" flip "$ID" draft ready_to_implement 2>&1)"; rc=$?
  [ "$rc" -eq 6 ] \
    || { fail t05_index_first_flip_refuses "index-first flip exit was $rc, expected 6 (truth-precedes-index refusal)"; all=0; }
  grep -qE 'truth precedes index|TASK-IMP-120' <<<"$out" \
    || { fail t05_index_first_flip_refuses "refusal did not name the truth-precedes-index contract: $(echo "$out" | head -1)"; all=0; }
  [ "$(row_status "$bl")" = draft ] \
    || { fail t05_index_first_flip_refuses "refused flip still moved the index row (now $(row_status "$bl"), expected draft)"; all=0; }

  # TRUTH-FIRST — the sanctioned order: write the truth to ready_to_implement, THEN the same flip
  # SUCCEEDS and the index catches up. Proves the refusal above was the guard, not a broken flip.
  write_spec "$spec" ready_to_implement
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --root "$d" flip "$ID" draft ready_to_implement >/dev/null 2>&1 \
    || { fail t05_index_first_flip_refuses "truth-first flip refused despite the truth agreeing"; all=0; }
  [ "$(row_status "$bl")" = ready_to_implement ] \
    || { fail t05_index_first_flip_refuses "truth-first flip did not move the index (now $(row_status "$bl"))"; all=0; }

  [ "$all" -eq 1 ] && ok t05_index_first_flip_refuses
}

t01_spine_green
t02_reconcile_recommendation_asserted
t03_corpus_survives_uninstall
t04_scratch_isolation
t05_index_first_flip_refuses

echo "e2e-skeleton: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
