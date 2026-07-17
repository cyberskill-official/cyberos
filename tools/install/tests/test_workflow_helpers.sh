#!/usr/bin/env bash
# test_workflow_helpers.sh - the doc-driven workflow helpers (TASK-IMP-085):
#   ship-manifest.mjs (ship-manifest@1 executor) + backlog-mutate.mjs
#   (backlog-state-update@2 byte-discipline executor).
#
#   t01  manifest lifecycle: init pins, record hashes at write time, verify
#        exit 0 naming the first non-done step, resume-line echoes the workflow's
#        mandated line EXACTLY, delete is terminal + idempotent.
#   t02  two-phase atomicity: a planted stale .tmp.<nonce> never corrupts reads
#        or writes (readers ignore tmp); pins (task_sha256 + workflow_version)
#        recorded at init; writes leave no tmp litter behind.
#   t03  verify staleness order with distinct exits: workflow_version mismatch
#        -> 3, task_sha256 mismatch -> 4, EARLIEST stale artefact -> 5 naming the
#        step, intact -> 0; resume-line inherits the same codes.
#   t04  flip rewrites exactly one cell; refuses with exit 6 on missing row,
#        duplicate rows (naming both lines), drifted status cell, and drifted
#        --old-line pre-image.
#   t05  insert: uniqueness gate (exit 7 anywhere in the file), regenerator-
#        identical grammar, stem-ascending placement (titles never affect it),
#        placeholder replacement in an empty section, no-counts header untouched.
#   t06  section-header counts stay true across flips and inserts — the counted
#        header is retallied from the section's ACTUAL rows after every mutation
#        (zero counts dropped, lifecycle order; TASK-IMP-092 replaced the
#        incremental +1/-1 adjust, so even the fixture's inherited header drift
#        is corrected by the first mutation).
#   t07  --json envelopes parse; --help documents the exit codes; byte-identical
#        reruns (both tools; ship-manifest under a pinned CYBEROS_NOW); whole-file
#        discipline proven by diff (= 1 row + at most 1 header line); CRLF
#        preserved outside the mutated line.
#   t08  the assembled payload carries both tools byte-identically and a scratch
#        install lays them into .cyberos/docs-tools/ where they run.
#   t09  doctrine wiring: ship-tasks.md (source + payload cuo/ + plugin copy)
#        names ship-manifest.mjs in Resume semantics and backlog-mutate.mjs in
#        the backlog-layout/state-engine area; workflow_version current (2.8.0
#        since TASK-IMP-099).
#   t10  a LYING counted header (counts disagreeing with the section's rows —
#        the 086 incident's shape) is rewritten to the true tally by any flip
#        AND any insert; statuses the header never listed join in lifecycle
#        order; a placeholder insert under phantom counts drops the header to
#        the sole real status (TASK-IMP-092).
#   t11  the retally stays inside the whole-file discipline: a mutation's diff
#        is exactly 1 row + at most 1 header line even when the header
#        correction is large (asserted line-by-line on the lying fixture).
#   t12  doctrine: ship-tasks.md carries the one-writer-one-view rule (§11a)
#        and the committed-object evidence rule (§9), in the source AND the
#        scratch payload's cuo/ copy, at workflow_version 2.8.0.
#   t13  queue selection ranks p0 before p1 before p2 before p3 (FM-105 scale)
#        in the source AND the scratch payload's cuo/ copy, with NO bare MoSCoW
#        ordering rule surviving (the FM-105 legacy-mapping parenthetical is the
#        one allowed mention), payload at workflow_version 2.8.0 (TASK-IMP-099, bumped by TASK-IMP-101).
#
# Origin: 2026-07-16 sachviet + cyberos batch-1 runs (IMPROVEMENT_HANDOFF.md
# IMP-04) - manifests were skipped and every backlog flip was hand-sed; the two
# strongest disciplines in the ship loop existed only as prose. This suite gates
# the tools that execute them.
#
# Usage: bash test_workflow_helpers.sh [t01 t02 ...]   (no args = all scenarios)
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
SM="$repo/tools/install/docs-tools/ship-manifest.mjs"
BM="$repo/tools/install/docs-tools/backlog-mutate.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }

NOW="2026-07-16T00:00:00Z"   # the injectable clock: CYBEROS_NOW pins every timestamp
sha() { node -e 'const{createHash}=require("node:crypto");const{readFileSync}=require("node:fs");process.stdout.write(createHash("sha256").update(readFileSync(process.argv[1])).digest("hex"))' "$1"; }

# One manifest fixture repo: a task spec, two artefacts, and a mini workflow doc
# whose frontmatter carries the version + skill_chain the tool parses.
emit_manifest_repo() { # $1 = repo dir
  mkdir -p "$1/docs/tasks/improvement/TASK-T-001-fixture" "$1/artefacts"
  printf 'id: TASK-T-001\nbody: fixture spec\n' > "$1/docs/tasks/improvement/TASK-T-001-fixture/spec.md"
  printf 'artefact one\n' > "$1/artefacts/a1.md"
  printf 'artefact two\n' > "$1/artefacts/a2.md"
  cat > "$1/wf.md" <<'EOF'
---
workflow_id: chief-technology-officer/ship-tasks
workflow_version: 9.9.9
skill_chain:
  - { step: 1,  skill: repo-context-map-author }
  - { step: 4,  skill: edge-case-matrix-audit }
---
EOF
}

sm() { CYBEROS_NOW="$NOW" node "$SM" "$@" > "$TMP/out" 2> "$TMP/err"; }

# Regenerator-identical backlog fixture: counted sections (alpha, beta), an
# empty section with the placeholder (beta), and a bare no-counts header (gamma).
emit_backlog() { # $1 = file
  mkdir -p "$(dirname "$1")"
  cat > "$1" <<'EOF'
# CyberOS task backlog (regenerated 2026-07-09)

Source of truth = task frontmatter.

Totals: 1 draft, 2 ready_to_implement, 1 implementing, 19 done

## alpha  (1 draft, 2 ready_to_implement, 2 done)

- [ready_to_implement] TASK-ALPHA-001-login-rate-limit - Login rate limiting
- [ready_to_implement] TASK-ALPHA-003-token-rotation - Token rotation (improvement)
- [draft] TASK-ALPHA-005-cache-layer - Cache layer

## beta  (17 done)

- (nothing remaining)

## gamma

- [implementing] TASK-GAMMA-001-no-counts - Header without counts

Totals are never touched by mutations.
EOF
}

bm() { node "$BM" "$@" > "$TMP/out" 2> "$TMP/err"; }

t01_manifest_lifecycle() {
  local d="$TMP/t01/repo"; emit_manifest_repo "$d"
  sm --root "$d" init TASK-T-001 --task-file docs/tasks/improvement/TASK-T-001-fixture/spec.md --workflow-version 9.9.9 \
    || { fail t01 "init failed: $(cat "$TMP/err")"; return; }
  local mf="$d/docs/tasks/.workflow/TASK-T-001.ship.json"
  [ -s "$mf" ] || { fail t01 "manifest not created"; return; }
  # re-init refuses without --force
  sm --root "$d" init TASK-T-001 --task-file docs/tasks/improvement/TASK-T-001-fixture/spec.md --workflow-version 9.9.9
  [ $? -eq 2 ] || { fail t01 "re-init without --force did not exit 2"; return; }
  # record: two done steps with artefacts, one conditional skip
  sm --root "$d" record TASK-T-001 1 repo-context-map-author done --artefact artefacts/a1.md --verdict pass \
    || { fail t01 "record step 1 failed: $(cat "$TMP/err")"; return; }
  sm --root "$d" record TASK-T-001 2 repo-context-map-audit done --artefact artefacts/a2.md --verdict pass \
    || { fail t01 "record step 2 failed"; return; }
  sm --root "$d" record TASK-T-001 3 architecture-decision-record-author skipped-conditional \
    || { fail t01 "record step 3 failed"; return; }
  # the recorded entry carries the full shape, artefact hashed at record time
  node -e '
    const m=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8"));
    const s1=m.steps.find(s=>s.index===1);
    const want=["index","skill","status","artefact_path","artefact_sha256","verdict","completed_at"];
    for (const k of want) if (!(k in s1)) { console.error("missing "+k); process.exit(1); }
    if (s1.artefact_sha256===null || s1.completed_at!=="2026-07-16T00:00:00Z") process.exit(1);
    if (m.steps.find(s=>s.index===3).artefact_sha256!==null) process.exit(1);
  ' "$mf" || { fail t01 "step entry shape/hash/clock wrong: $(cat "$mf")"; return; }
  # verify -> exit 0 naming the first non-done step (4)
  sm --root "$d" verify TASK-T-001 --workflow-version 9.9.9; local rc=$?
  { [ "$rc" -eq 0 ] && grep -q "step 4/31" "$TMP/out"; } || { fail t01 "verify rc=$rc out=$(cat "$TMP/out")"; return; }
  # resume-line echoes the workflow's mandated format EXACTLY (skill from the doc's chain)
  sm --root "$d" resume-line TASK-T-001 --workflow-version 9.9.9 --workflow-doc wf.md; rc=$?
  local wantline="resume TASK-T-001: steps 1-3 verified (2 artefacts, hashes OK), continuing at step 4/31 (edge-case-matrix-audit). routed_back_count=0"
  { [ "$rc" -eq 0 ] && [ "$(cat "$TMP/out")" = "$wantline" ]; } \
    || { fail t01 "resume-line rc=$rc got: $(cat "$TMP/out")"; return; }
  # route-back bumps the count and the line reflects it
  sm --root "$d" record TASK-T-001 4 edge-case-matrix-author failed --routed-back \
    || { fail t01 "record --routed-back failed"; return; }
  sm --root "$d" resume-line TASK-T-001 --workflow-version 9.9.9 --workflow-doc wf.md
  grep -q "routed_back_count=1" "$TMP/out" || { fail t01 "routed_back_count not echoed: $(cat "$TMP/out")"; return; }
  # a task without a manifest is a loud exit 2, never a guess or a crash
  sm --root "$d" verify TASK-NOPE --workflow-version 9.9.9
  [ $? -eq 2 ] || { fail t01 "verify on missing manifest did not exit 2"; return; }
  # delete: terminal + idempotent
  sm --root "$d" delete TASK-T-001 || { fail t01 "delete failed"; return; }
  [ ! -e "$mf" ] || { fail t01 "manifest survived delete"; return; }
  sm --root "$d" delete TASK-T-001 || { fail t01 "second delete not exit 0"; return; }
  ok t01
}

t02_two_phase_atomic() {
  local d="$TMP/t02/repo"; emit_manifest_repo "$d"
  sm --root "$d" init TASK-T-001 --task-file docs/tasks/improvement/TASK-T-001-fixture/spec.md --workflow-version 9.9.9 \
    || { fail t02 "init failed"; return; }
  local mf="$d/docs/tasks/.workflow/TASK-T-001.ship.json"
  # pins recorded at init: task_sha256 == sha256 of the spec, workflow_version pinned
  local specsha; specsha="$(sha "$d/docs/tasks/improvement/TASK-T-001-fixture/spec.md")"
  grep -q "\"task_sha256\": \"$specsha\"" "$mf" || { fail t02 "task_sha256 pin wrong"; return; }
  grep -q '"workflow_version": "9.9.9"' "$mf" || { fail t02 "workflow_version pin missing"; return; }
  # plant a stale tmp from a killed write: readers must ignore it, writers must survive it
  printf 'GARBAGE{{{not json' > "$mf.tmp.deadbeef"
  sm --root "$d" verify TASK-T-001 --workflow-version 9.9.9; local rc=$?
  [ "$rc" -eq 0 ] || { fail t02 "verify with planted tmp rc=$rc: $(cat "$TMP/out") $(cat "$TMP/err")"; return; }
  sm --root "$d" record TASK-T-001 1 repo-context-map-author done --artefact artefacts/a1.md \
    || { fail t02 "record with planted tmp failed"; return; }
  node -e 'JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8"))' "$mf" \
    || { fail t02 "manifest corrupted by planted tmp"; return; }
  grep -q '"index": 1' "$mf" || { fail t02 "record did not land"; return; }
  # the planted tmp is untouched (never consumed as state) and the write left no litter
  [ -f "$mf.tmp.deadbeef" ] || { fail t02 "planted tmp was consumed"; return; }
  local litter; litter="$(ls "$d/docs/tasks/.workflow" | grep -c '\.tmp\.' )"
  [ "$litter" -eq 1 ] || { fail t02 "expected only the planted tmp, found $litter tmp files"; return; }
  # record hashes AT RECORD TIME: a missing artefact is a loud exit 2, never a null hash
  sm --root "$d" record TASK-T-001 2 repo-context-map-audit done --artefact artefacts/absent.md; local rc=$?
  { [ "$rc" -eq 2 ] && grep -q "unreadable at record time" "$TMP/err"; } \
    || { fail t02 "missing artefact rc=$rc: $(cat "$TMP/err")"; return; }
  ok t02
}

t03_verify_staleness_exits() {
  local d="$TMP/t03/repo"; emit_manifest_repo "$d"
  local spec="$d/docs/tasks/improvement/TASK-T-001-fixture/spec.md"
  cp "$spec" "$TMP/t03/spec.bak"
  sm --root "$d" init TASK-T-001 --task-file docs/tasks/improvement/TASK-T-001-fixture/spec.md --workflow-version 1.0.0 \
    || { fail t03 "init failed"; return; }
  sm --root "$d" record TASK-T-001 1 repo-context-map-author done --artefact artefacts/a1.md || { fail t03 "record 1 failed"; return; }
  sm --root "$d" record TASK-T-001 2 repo-context-map-audit done --artefact artefacts/a2.md || { fail t03 "record 2 failed"; return; }
  # exit 3: workflow_version mismatch -> needs_human
  sm --root "$d" verify TASK-T-001 --workflow-version 2.0.0; local rc=$?
  { [ "$rc" -eq 3 ] && grep -q "needs_human" "$TMP/out"; } || { fail t03 "version mismatch rc=$rc: $(cat "$TMP/out")"; return; }
  # exit 4: task spec edited since run start
  printf 'edited\n' >> "$spec"
  sm --root "$d" verify TASK-T-001 --workflow-version 1.0.0; rc=$?
  { [ "$rc" -eq 4 ] && grep -q "task_sha256 mismatch" "$TMP/out"; } || { fail t03 "task-hash mismatch rc=$rc: $(cat "$TMP/out")"; return; }
  cp "$TMP/t03/spec.bak" "$spec"
  # exit 5: EARLIEST stale artefact named (tamper both, step 1 must win)
  printf 'tampered\n' >> "$d/artefacts/a1.md"
  printf 'tampered\n' >> "$d/artefacts/a2.md"
  sm --root "$d" verify TASK-T-001 --workflow-version 1.0.0; rc=$?
  { [ "$rc" -eq 5 ] && grep -q "step 1 (repo-context-map-author)" "$TMP/out"; } \
    || { fail t03 "stale artefact rc=$rc: $(cat "$TMP/out")"; return; }
  grep -q "step 2" "$TMP/out" && { fail t03 "verify named step 2, not the earliest"; return; }
  # resume-line inherits the staleness exit (it never claims hashes OK it did not prove)
  sm --root "$d" resume-line TASK-T-001 --workflow-version 1.0.0; rc=$?
  [ "$rc" -eq 5 ] || { fail t03 "resume-line on stale artefact rc=$rc"; return; }
  # intact again -> exit 0, and the exact resume line
  printf 'artefact one\n' > "$d/artefacts/a1.md"
  printf 'artefact two\n' > "$d/artefacts/a2.md"
  sm --root "$d" verify TASK-T-001 --workflow-version 1.0.0; rc=$?
  { [ "$rc" -eq 0 ] && grep -q "step 3/31" "$TMP/out"; } || { fail t03 "intact verify rc=$rc: $(cat "$TMP/out")"; return; }
  sm --root "$d" resume-line TASK-T-001 --workflow-version 1.0.0; rc=$?
  local wantline="resume TASK-T-001: steps 1-2 verified (2 artefacts, hashes OK), continuing at step 3/31 (unknown). routed_back_count=0"
  { [ "$rc" -eq 0 ] && [ "$(cat "$TMP/out")" = "$wantline" ]; } \
    || { fail t03 "exact resume line rc=$rc got: $(cat "$TMP/out")"; return; }
  ok t03
}

t04_flip_and_drift_refusal() {
  local f="$TMP/t04/BACKLOG.md"; emit_backlog "$f"
  # green flip rewrites exactly the located cell; line count unchanged
  local before; before="$(wc -l < "$f")"
  bm flip TASK-ALPHA-001 ready_to_implement implementing --backlog "$f" \
    || { fail t04 "green flip failed: $(cat "$TMP/err")"; return; }
  grep -q '^- \[implementing\] TASK-ALPHA-001-login-rate-limit - Login rate limiting$' "$f" \
    || { fail t04 "flipped row wrong"; return; }
  grep -q '^- \[ready_to_implement\] TASK-ALPHA-001' "$f" && { fail t04 "old row survived"; return; }
  [ "$(wc -l < "$f")" -eq "$before" ] || { fail t04 "line count changed on flip"; return; }
  # the (improvement) suffix is preserved bytes, not grammar the flip re-renders
  bm flip TASK-ALPHA-003 ready_to_implement implementing --backlog "$f" || { fail t04 "flip 003 failed"; return; }
  grep -q '^- \[implementing\] TASK-ALPHA-003-token-rotation - Token rotation (improvement)$' "$f" \
    || { fail t04 "improvement tag not preserved"; return; }
  # drift: status cell no longer matches the pre-image -> exit 6
  bm flip TASK-ALPHA-001 ready_to_implement implementing --backlog "$f"; local rc=$?
  { [ "$rc" -eq 6 ] && grep -q "drifted" "$TMP/err"; } || { fail t04 "cell drift rc=$rc: $(cat "$TMP/err")"; return; }
  # drift: full --old-line differs byte-for-byte -> exit 6
  bm flip TASK-ALPHA-005 draft ready_to_implement --backlog "$f" \
    --old-line '- [draft] TASK-ALPHA-005-cache-layer - Cache Layer'; rc=$?
  { [ "$rc" -eq 6 ] && grep -q "byte-for-byte" "$TMP/err"; } || { fail t04 "old-line drift rc=$rc"; return; }
  # the matching --old-line passes
  bm flip TASK-ALPHA-005 draft ready_to_implement --backlog "$f" \
    --old-line '- [draft] TASK-ALPHA-005-cache-layer - Cache layer' \
    || { fail t04 "matching old-line refused: $(cat "$TMP/err")"; return; }
  # missing row -> exit 6
  bm flip TASK-ALPHA-099 draft done --backlog "$f"; rc=$?
  { [ "$rc" -eq 6 ] && grep -q "missing row" "$TMP/err"; } || { fail t04 "missing row rc=$rc"; return; }
  # duplicate rows (corrupted backlog) -> exit 6 naming both lines
  local g="$TMP/t04/dup.md"; emit_backlog "$g"
  printf -- '- [draft] TASK-ALPHA-005-cache-layer - Cache layer\n' >> "$g"
  bm flip TASK-ALPHA-005 draft done --backlog "$g"; rc=$?
  { [ "$rc" -eq 6 ] && grep -q "2 rows match" "$TMP/err" && grep -Eq 'lines [0-9]+ and [0-9]+' "$TMP/err"; } \
    || { fail t04 "duplicate rows rc=$rc: $(cat "$TMP/err")"; return; }
  ok t04
}

t05_insert_uniqueness_and_grammar() {
  local f="$TMP/t05/BACKLOG.md"; emit_backlog "$f"
  # stem-ascending placement: 002 lands between 001 and 003 (auto-detected section)
  bm insert TASK-ALPHA-002 TASK-ALPHA-002-token-scope "Token scoping" ready_to_implement --backlog "$f" \
    || { fail t05 "insert failed: $(cat "$TMP/err")"; return; }
  grep -q '^- \[ready_to_implement\] TASK-ALPHA-002-token-scope - Token scoping$' "$f" \
    || { fail t05 "row grammar wrong"; return; }
  local block; block="$(grep -n 'TASK-ALPHA-00[123]' "$f" | cut -d: -f2- | sed 's/^[0-9]*://')"
  awk '/TASK-ALPHA-001/{a=NR} /TASK-ALPHA-002/{b=NR} /TASK-ALPHA-003/{c=NR} END{exit !(a<b && b<c)}' "$f" \
    || { fail t05 "stem placement wrong: $(grep -n TASK-ALPHA "$f")"; return; }
  # uniqueness: same id anywhere in the file -> exit 7 (even in another section, other slug)
  bm insert TASK-ALPHA-002 TASK-ALPHA-002-other-slug "Other" draft --backlog "$f" --section gamma; local rc=$?
  { [ "$rc" -eq 7 ] && grep -q "uniqueness" "$TMP/err"; } || { fail t05 "uniqueness rc=$rc: $(cat "$TMP/err")"; return; }
  # unicode title: placement is bytewise on the STEM token only; improvement tag renders
  bm insert TASK-ALPHA-004 TASK-ALPHA-004-viet-title "Tiêu đề tiếng Việt ✓" draft --backlog "$f" --class improvement \
    || { fail t05 "unicode insert failed"; return; }
  grep -q '^- \[draft\] TASK-ALPHA-004-viet-title - Tiêu đề tiếng Việt ✓ (improvement)$' "$f" \
    || { fail t05 "unicode/improvement row wrong"; return; }
  awk '/TASK-ALPHA-003/{c=NR} /TASK-ALPHA-004/{d=NR} /TASK-ALPHA-005/{e=NR} END{exit !(c<d && d<e)}' "$f" \
    || { fail t05 "unicode row placement wrong"; return; }
  # empty section: the placeholder becomes the first row of the block
  bm insert TASK-BETA-001 TASK-BETA-001-first "First beta task" ready_to_implement --backlog "$f" --section beta \
    || { fail t05 "placeholder insert failed: $(cat "$TMP/err")"; return; }
  grep -q '(nothing remaining)' "$f" && { fail t05 "placeholder survived the insert"; return; }
  grep -q '^- \[ready_to_implement\] TASK-BETA-001-first - First beta task$' "$f" || { fail t05 "beta row missing"; return; }
  # no-counts header (gamma): row lands, header line untouched
  bm insert TASK-GAMMA-002 TASK-GAMMA-002-second "Second gamma" draft --backlog "$f" --section gamma \
    || { fail t05 "gamma insert failed"; return; }
  grep -q '^## gamma$' "$f" || { fail t05 "no-counts header was edited"; return; }
  # ambiguous target without --section -> exit 2, never a guess
  bm insert TASK-DELTA-001 TASK-DELTA-001-x "X" draft --backlog "$f"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q -- "--section" "$TMP/err"; } || { fail t05 "no-candidate insert rc=$rc"; return; }
  # row-injection guard: a title carrying a newline can never smuggle a second row
  bm insert TASK-EPS-001 TASK-EPS-001-evil "$(printf 'evil\n- [done] TASK-X - smuggled')" draft --backlog "$f" --section gamma; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "row-injection" "$TMP/err"; } || { fail t05 "newline title rc=$rc: $(cat "$TMP/err")"; return; }
  grep -q "smuggled" "$f" && { fail t05 "injected row landed"; return; }
  ok t05
}

t06_counts_maintained() {
  local f="$TMP/t06/BACKLOG.md"; emit_backlog "$f"
  # flip: the header is retallied from the section's rows (TASK-IMP-092) — the fixture's
  # alpha header claims '2 done' over zero done rows, so the first mutation corrects it away
  bm flip TASK-ALPHA-001 ready_to_implement implementing --backlog "$f" || { fail t06 "flip 1 failed"; return; }
  grep -q '^## alpha  (1 draft, 1 ready_to_implement, 1 implementing)$' "$f" \
    || { fail t06 "header after flip 1: $(grep '^## alpha' "$f")"; return; }
  # flip the last ready_to_implement away: the zero count is dropped from the header
  bm flip TASK-ALPHA-003 ready_to_implement implementing --backlog "$f" || { fail t06 "flip 2 failed"; return; }
  grep -q '^## alpha  (1 draft, 2 implementing)$' "$f" \
    || { fail t06 "zero count not dropped: $(grep '^## alpha' "$f")"; return; }
  # insert: the retally counts the new row
  bm insert TASK-ALPHA-002 TASK-ALPHA-002-token-scope "Token scoping" draft --backlog "$f" \
    || { fail t06 "insert failed"; return; }
  grep -q '^## alpha  (2 draft, 2 implementing)$' "$f" \
    || { fail t06 "header after insert: $(grep '^## alpha' "$f")"; return; }
  # placeholder insert into beta: the phantom '17 done' (a count with no rows) drops to
  # the sole real status — never a zero entry, never an inherited ghost
  bm insert TASK-BETA-001 TASK-BETA-001-first "First" ready_to_implement --backlog "$f" --section beta \
    || { fail t06 "beta insert failed"; return; }
  grep -q '^## beta  (1 ready_to_implement)$' "$f" \
    || { fail t06 "beta header: $(grep '^## beta' "$f")"; return; }
  # the file-top Totals line is NEVER touched (not part of the declared mutation)
  grep -q '^Totals: 1 draft, 2 ready_to_implement, 1 implementing, 19 done$' "$f" \
    || { fail t06 "Totals line was touched"; return; }
  ok t06
}

t07_json_and_determinism() {
  local d="$TMP/t07"
  # --help documents the exit codes (both tools)
  node "$SM" --help > "$d.smhelp" 2>&1 || { fail t07 "ship-manifest --help failed"; return; }
  grep -q '^exit codes' "$d.smhelp" && grep -q '3  workflow_version mismatch' "$d.smhelp" \
    && grep -q '4  task_sha256 mismatch' "$d.smhelp" && grep -q '5  artefact hash mismatch' "$d.smhelp" \
    || { fail t07 "ship-manifest --help lacks exit-code docs"; return; }
  grep -q 'CYBEROS_NOW' "$d.smhelp" || { fail t07 "injectable clock undocumented"; return; }
  node "$BM" --help > "$d.bmhelp" 2>&1 || { fail t07 "backlog-mutate --help failed"; return; }
  grep -q '^exit codes' "$d.bmhelp" && grep -q '6  flip refusal' "$d.bmhelp" && grep -q '7  insert refusal' "$d.bmhelp" \
    || { fail t07 "backlog-mutate --help lacks exit-code docs"; return; }
  # ship-manifest determinism: two fresh runs under the same pinned clock are byte-identical
  local a="$d/a" b="$d/b"
  emit_manifest_repo "$a"; emit_manifest_repo "$b"
  local args="init TASK-T-001 --task-file docs/tasks/improvement/TASK-T-001-fixture/spec.md --workflow-version 9.9.9"
  sm --root "$a" $args && sm --root "$b" $args || { fail t07 "det init failed"; return; }
  sm --root "$a" record TASK-T-001 1 repo-context-map-author done --artefact artefacts/a1.md
  sm --root "$b" record TASK-T-001 1 repo-context-map-author done --artefact artefacts/a1.md
  cmp -s "$a/docs/tasks/.workflow/TASK-T-001.ship.json" "$b/docs/tasks/.workflow/TASK-T-001.ship.json" \
    || { fail t07 "manifests differ across identical runs"; return; }
  CYBEROS_NOW="$NOW" node "$SM" --root "$a" --json verify TASK-T-001 --workflow-version 9.9.9 > "$d/v1.json" 2>/dev/null
  CYBEROS_NOW="$NOW" node "$SM" --root "$a" --json verify TASK-T-001 --workflow-version 9.9.9 > "$d/v2.json" 2>/dev/null
  cmp -s "$d/v1.json" "$d/v2.json" || { fail t07 "verify --json reruns differ"; return; }
  node -e 'const j=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")); if (j.ok!==true||j.exit_code!==0) process.exit(1)' "$d/v1.json" \
    || { fail t07 "verify --json envelope invalid"; return; }
  # backlog-mutate determinism + whole-file discipline: diff = 1 row + 1 header line
  emit_backlog "$d/c1/BACKLOG.md"; emit_backlog "$d/c2/BACKLOG.md"; emit_backlog "$d/pre.md"
  (cd "$d/c1" && node "$BM" --json flip TASK-ALPHA-001 ready_to_implement implementing --backlog BACKLOG.md > "$d/f1.json" 2>&1) \
    || { fail t07 "json flip failed: $(cat "$d/f1.json")"; return; }
  (cd "$d/c2" && node "$BM" --json flip TASK-ALPHA-001 ready_to_implement implementing --backlog BACKLOG.md > "$d/f2.json" 2>&1)
  cmp -s "$d/f1.json" "$d/f2.json" || { fail t07 "flip --json reruns differ"; return; }
  cmp -s "$d/c1/BACKLOG.md" "$d/c2/BACKLOG.md" || { fail t07 "mutated backlogs differ across identical runs"; return; }
  node -e 'const j=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")); if (j.ok!==true||!j.old_line||!j.new_line||!j.new_header) process.exit(1)' "$d/f1.json" \
    || { fail t07 "flip envelope lacks old_line/new_line/new_header"; return; }
  local removed added
  removed="$(diff "$d/pre.md" "$d/c1/BACKLOG.md" | grep -c '^<')"; added="$(diff "$d/pre.md" "$d/c1/BACKLOG.md" | grep -c '^>')"
  { [ "$removed" -eq 2 ] && [ "$added" -eq 2 ]; } \
    || { fail t07 "flip footprint not 1 row + 1 header (removed=$removed added=$added)"; return; }
  # insert footprint: 1 added row + 1 changed header, nothing else
  emit_backlog "$d/c3/BACKLOG.md"
  bm insert TASK-ALPHA-002 TASK-ALPHA-002-token-scope "Token scoping" draft --backlog "$d/c3/BACKLOG.md" \
    || { fail t07 "insert for footprint failed"; return; }
  removed="$(diff "$d/pre.md" "$d/c3/BACKLOG.md" | grep -c '^<')"; added="$(diff "$d/pre.md" "$d/c3/BACKLOG.md" | grep -c '^>')"
  { [ "$removed" -eq 1 ] && [ "$added" -eq 2 ]; } \
    || { fail t07 "insert footprint not 1 row + 1 header (removed=$removed added=$added)"; return; }
  # no-counts flip footprint: exactly the one row
  emit_backlog "$d/c4/BACKLOG.md"
  bm flip TASK-GAMMA-001 implementing ready_to_review --backlog "$d/c4/BACKLOG.md" || { fail t07 "gamma flip failed"; return; }
  removed="$(diff "$d/pre.md" "$d/c4/BACKLOG.md" | grep -c '^<')"; added="$(diff "$d/pre.md" "$d/c4/BACKLOG.md" | grep -c '^>')"
  { [ "$removed" -eq 1 ] && [ "$added" -eq 1 ]; } || { fail t07 "no-counts flip footprint (removed=$removed added=$added)"; return; }
  # CRLF: bytes preserved outside the mutated line; no line-ending drift anywhere
  emit_backlog "$d/crlf.md"; sed 's/$/\r/' "$d/crlf.md" > "$d/crlf2.md"; mv "$d/crlf2.md" "$d/crlf.md"
  local crlf_before; crlf_before="$(grep -c $'\r$' "$d/crlf.md")"
  bm flip TASK-ALPHA-001 ready_to_implement implementing --backlog "$d/crlf.md" || { fail t07 "CRLF flip failed"; return; }
  [ "$(grep -c $'\r$' "$d/crlf.md")" -eq "$crlf_before" ] || { fail t07 "CRLF endings drifted"; return; }
  grep -q $'^- \\[implementing\\] TASK-ALPHA-001-login-rate-limit - Login rate limiting\r$' "$d/crlf.md" \
    || { fail t07 "CRLF flipped row lost its ending"; return; }
  # an inserted row inherits the section's CRLF ending
  bm insert TASK-ALPHA-002 TASK-ALPHA-002-token-scope "Token scoping" draft --backlog "$d/crlf.md" \
    || { fail t07 "CRLF insert failed"; return; }
  grep -q $'^- \\[draft\\] TASK-ALPHA-002-token-scope - Token scoping\r$' "$d/crlf.md" \
    || { fail t07 "CRLF inserted row has no CR"; return; }
  # --json error envelopes carry ok:false + the documented exit code
  node "$BM" --json flip TASK-ALPHA-099 draft done --backlog "$d/pre.md" > "$d/e.json" 2>/dev/null; local rc=$?
  { [ "$rc" -eq 6 ] && node -e 'const j=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")); if (j.ok!==false||j.exit_code!==6) process.exit(1)' "$d/e.json"; } \
    || { fail t07 "error envelope rc=$rc: $(cat "$d/e.json")"; return; }
  ok t07
}

ensure_payload() {
  [ -s "$TMP/payload/install.sh" ] && return 0
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1
}

t08_payload_and_install() {
  ensure_payload || { fail t08 "build.sh failed"; return; }
  local t
  for t in ship-manifest.mjs backlog-mutate.mjs; do
    [ -s "$TMP/payload/docs-tools/$t" ] || { fail t08 "payload docs-tools/$t missing or empty"; return; }
    cmp -s "$repo/tools/install/docs-tools/$t" "$TMP/payload/docs-tools/$t" \
      || { fail t08 "payload $t differs from tools/install/docs-tools/$t"; return; }
  done
  local d="$TMP/scratch"; mkdir -p "$d"; (cd "$d" && git init -q . 2>/dev/null || true)
  (cd "$d" && CYBEROS_NO_MIGRATE=1 CYBEROS_NO_HOOK=1 CYBEROS_NO_MEMORY=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1) \
    || { fail t08 "install.sh failed"; return; }
  for t in ship-manifest.mjs backlog-mutate.mjs; do
    [ -s "$d/.cyberos/docs-tools/$t" ] || { fail t08 ".cyberos/docs-tools/$t missing after install"; return; }
  done
  # the vendored copies actually run in the installed repo
  node "$d/.cyberos/docs-tools/ship-manifest.mjs" --help >/dev/null 2>&1 || { fail t08 "installed ship-manifest --help failed"; return; }
  node "$d/.cyberos/docs-tools/backlog-mutate.mjs" --help >/dev/null 2>&1 || { fail t08 "installed backlog-mutate --help failed"; return; }
  # end-to-end inside the scratch repo: init against the vendored workflow doc's version
  mkdir -p "$d/docs/tasks/improvement/TASK-S-001-smoke"
  printf 'smoke\n' > "$d/docs/tasks/improvement/TASK-S-001-smoke/spec.md"
  (cd "$d" && CYBEROS_NOW="$NOW" node .cyberos/docs-tools/ship-manifest.mjs init TASK-S-001 \
      --task-file docs/tasks/improvement/TASK-S-001-smoke/spec.md --workflow-version 0.0.1 >/dev/null 2>&1 \
    && CYBEROS_NOW="$NOW" node .cyberos/docs-tools/ship-manifest.mjs verify TASK-S-001 --workflow-version 0.0.1 >/dev/null 2>&1) \
    || { fail t08 "installed ship-manifest lifecycle failed"; return; }
  ok t08
}

t09_doctrine_wiring() {
  ensure_payload || { fail t09 "build.sh failed"; return; }
  local f
  for f in \
    "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" \
    "$TMP/payload/cuo/ship-tasks.md" \
    "$TMP/payload/plugin/skills/ship-tasks/cuo/ship-tasks.md"; do
    [ -s "$f" ] || { fail t09 "missing $f"; return; }
    # the Resume semantics section names ship-manifest.mjs as the doc-driven reference implementation
    awk '/^## Resume semantics/,/^## Cross-references/' "$f" > "$TMP/resume.sec"
    grep -q 'ship-manifest\.mjs' "$TMP/resume.sec" || { fail t09 "$f: Resume semantics lacks the ship-manifest.mjs pointer"; return; }
    grep -q 'ship_manifest\.py' "$TMP/resume.sec" || { fail t09 "$f: python reference no longer named"; return; }
    # the backlog-layout/state-engine area names backlog-mutate.mjs as the byte-discipline executor
    awk '/^### Backlog layout/,/^### HITL/' "$f" > "$TMP/layout.sec"
    grep -q 'backlog-mutate\.mjs' "$TMP/layout.sec" || { fail t09 "$f: backlog layout lacks the backlog-mutate.mjs pointer"; return; }
    grep -q 'byte-discipline executor' "$TMP/layout.sec" || { fail t09 "$f: pointer does not say byte-discipline executor"; return; }
    # the doc gained normative pointers -> workflow_version bumped
    grep -q '^workflow_version: 2\.8\.0$' "$f" || { fail t09 "$f: workflow_version not current (want 2.8.0)"; return; }
  done
  ok t09
}

# The 086 incident's shape: counted headers that DISAGREE with their rows — alpha
# claims '34 done' over rows tallying 1 draft, 1 implementing, 2 done (the incident's
# inherited 34-vs-true-20 baseline in miniature); omega claims '9 draft' over an empty
# section. The incremental adjust preserved such lies through six mutations; the
# retally (TASK-IMP-092) must correct them on ANY mutation.
emit_lying_backlog() { # $1 = file
  mkdir -p "$(dirname "$1")"
  cat > "$1" <<'EOF'
# CyberOS task backlog (regenerated 2026-07-09)

Totals: 1 draft, 1 implementing, 2 done

## alpha  (34 done)

- [draft] TASK-ALPHA-001-login-rate-limit - Login rate limiting
- [implementing] TASK-ALPHA-002-token-scope - Token scoping
- [done] TASK-ALPHA-003-token-rotation - Token rotation (improvement)
- [done] TASK-ALPHA-005-cache-layer - Cache layer

## omega  (9 draft)

- (nothing remaining)

Totals are never touched by mutations.
EOF
}

t10_retally_corrects_lying_header() {
  # flip: the rewritten header is the section's TRUE tally, not old-counts ± 1 —
  # statuses the header never listed (implementing, ready_to_review) join in
  # lifecycle order and the inherited '34 done' collapses to the real 2
  local f="$TMP/t10/flip.md"; emit_lying_backlog "$f"
  bm flip TASK-ALPHA-001 draft ready_to_review --backlog "$f" || { fail t10 "flip failed: $(cat "$TMP/err")"; return; }
  grep -q '^## alpha  (1 implementing, 1 ready_to_review, 2 done)$' "$f" \
    || { fail t10 "flip did not retally: $(grep '^## alpha' "$f")"; return; }
  # insert corrects the same lie (fresh fixture — ANY mutation, not just flip)
  local g="$TMP/t10/insert.md"; emit_lying_backlog "$g"
  bm insert TASK-ALPHA-004 TASK-ALPHA-004-new-row "New row" draft --backlog "$g" \
    || { fail t10 "insert failed: $(cat "$TMP/err")"; return; }
  grep -q '^## alpha  (2 draft, 1 implementing, 2 done)$' "$g" \
    || { fail t10 "insert did not retally: $(grep '^## alpha' "$g")"; return; }
  # empty section under phantom counts: the placeholder insert drops '9 draft' to the
  # sole real status — never a zero entry, never a negative
  bm insert TASK-OMEGA-001 TASK-OMEGA-001-first "First omega" done --backlog "$g" --section omega \
    || { fail t10 "omega insert failed: $(cat "$TMP/err")"; return; }
  grep -q '^## omega  (1 done)$' "$g" || { fail t10 "omega header: $(grep '^## omega' "$g")"; return; }
  # the --json envelope names the correction, so a caller SEES the lie it just fixed
  node "$BM" --json flip TASK-ALPHA-002 implementing done --backlog "$g" > "$TMP/t10/j.json" 2>&1 \
    || { fail t10 "json flip failed: $(cat "$TMP/t10/j.json")"; return; }
  node -e '
    const j=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8"));
    if (j.old_header!=="## alpha  (2 draft, 1 implementing, 2 done)") process.exit(1);
    if (j.new_header!=="## alpha  (2 draft, 3 done)") process.exit(1);
  ' "$TMP/t10/j.json" || { fail t10 "envelope old/new header wrong: $(cat "$TMP/t10/j.json")"; return; }
  ok t10
}

t11_footprint_holds_with_retally() {
  # even a LARGE header correction stays inside the whole-file discipline: the diff is
  # exactly the mutated row + the one header line the mutation was allowed to touch
  local d="$TMP/t11"; emit_lying_backlog "$d/pre.md"; emit_lying_backlog "$d/flip.md"
  bm flip TASK-ALPHA-001 draft ready_to_review --backlog "$d/flip.md" || { fail t11 "flip failed"; return; }
  diff "$d/pre.md" "$d/flip.md" | sed -n 's/^< //p' > "$d/removed"
  diff "$d/pre.md" "$d/flip.md" | sed -n 's/^> //p' > "$d/added"
  { [ "$(wc -l < "$d/removed")" -eq 2 ] && [ "$(wc -l < "$d/added")" -eq 2 ]; } \
    || { fail t11 "flip footprint not 1 row + 1 header: removed=[$(cat "$d/removed")] added=[$(cat "$d/added")]"; return; }
  { grep -qxF '## alpha  (34 done)' "$d/removed" \
      && grep -qxF -- '- [draft] TASK-ALPHA-001-login-rate-limit - Login rate limiting' "$d/removed"; } \
    || { fail t11 "flip removed lines are not header+row: $(cat "$d/removed")"; return; }
  { grep -qxF '## alpha  (1 implementing, 1 ready_to_review, 2 done)' "$d/added" \
      && grep -qxF -- '- [ready_to_review] TASK-ALPHA-001-login-rate-limit - Login rate limiting' "$d/added"; } \
    || { fail t11 "flip added lines are not header+row: $(cat "$d/added")"; return; }
  # insert: 1 rewritten header + 1 added row, nothing else
  emit_lying_backlog "$d/ins.md"
  bm insert TASK-ALPHA-004 TASK-ALPHA-004-new-row "New row" draft --backlog "$d/ins.md" || { fail t11 "insert failed"; return; }
  local removed added
  removed="$(diff "$d/pre.md" "$d/ins.md" | grep -c '^<')"; added="$(diff "$d/pre.md" "$d/ins.md" | grep -c '^>')"
  { [ "$removed" -eq 1 ] && [ "$added" -eq 2 ]; } \
    || { fail t11 "insert footprint not 1 row + 1 header (removed=$removed added=$added)"; return; }
  # 'at most one': a mutation under a BARE header changes exactly the one row
  emit_backlog "$d/bare-pre.md"; emit_backlog "$d/bare.md"
  bm flip TASK-GAMMA-001 implementing ready_to_review --backlog "$d/bare.md" || { fail t11 "bare flip failed"; return; }
  removed="$(diff "$d/bare-pre.md" "$d/bare.md" | grep -c '^<')"; added="$(diff "$d/bare-pre.md" "$d/bare.md" | grep -c '^>')"
  { [ "$removed" -eq 1 ] && [ "$added" -eq 1 ]; } \
    || { fail t11 "bare-header footprint not 1 row + 0 header (removed=$removed added=$added)"; return; }
  ok t11
}

t12_doctrine_view_rules_vendored() {
  ensure_payload || { fail t12 "build.sh failed"; return; }
  local f
  for f in \
    "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" \
    "$TMP/payload/cuo/ship-tasks.md"; do
    [ -s "$f" ] || { fail t12 "missing $f"; return; }
    # §11a swarm rule: shared files get ONE writer through ONE filesystem view per run
    grep -q 'ONE writer through ONE filesystem view' "$f" \
      || { fail t12 "$f: one-writer-one-view rule missing"; return; }
    grep -q 'cone-independence includes view-independence' "$f" \
      || { fail t12 "$f: view-independence passage missing"; return; }
    # §9 testing-phase rule: committed-object evidence for content deliverables
    grep -q 'git show <commit>:<path>' "$f" \
      || { fail t12 "$f: committed-object evidence command missing"; return; }
    grep -q 'never a working view' "$f" \
      || { fail t12 "$f: never-a-working-view rule missing"; return; }
    # the doc gained normative rules -> workflow_version bumped
    grep -q '^workflow_version: 2\.8\.0$' "$f" \
      || { fail t12 "$f: workflow_version not bumped to 2.8.0"; return; }
  done
  ok t12
}

t13_queue_rule_p0_p3() {
  # TASK-IMP-099: the queue-selection prose ranks the FM-105 priority scale. The
  # negative grep targets the rule SHAPE ('<value> before <value>' in MoSCoW
  # terms) - the legacy-mapping parenthetical is the one allowed MoSCoW mention.
  ensure_payload || { fail t13 "build.sh failed"; return; }
  local f
  for f in \
    "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" \
    "$TMP/payload/cuo/ship-tasks.md"; do
    [ -s "$f" ] || { fail t13 "missing $f"; return; }
    grep -q 'order by priority: `p0` before `p1` before `p2` before `p3`' "$f" \
      || { fail t13 "$f: p0-p3 ordering rule missing"; return; }
    grep -q 'legacy MoSCoW values map per FM-105' "$f" \
      || { fail t13 "$f: FM-105 legacy-mapping parenthetical missing"; return; }
    grep -Eiq '(MUST|SHOULD|COULD|WON.?T)[[:space:]]+before[[:space:]]+(MUST|SHOULD|COULD|WON.?T)' "$f" \
      && { fail t13 "$f: bare MoSCoW ordering rule survives: $(grep -Ein '(MUST|SHOULD|COULD|WON.?T)[[:space:]]+before' "$f" | head -3)"; return; }
  done
  # the reword is a normative change: the payload ships it at the bumped version
  grep -q '^workflow_version: 2\.8\.0$' "$TMP/payload/cuo/ship-tasks.md" \
    || { fail t13 "payload cuo/ship-tasks.md workflow_version not 2.8.0"; return; }
  ok t13
}

# ── t14: reconcile entry + depends_on evidence gate are vendored (TASK-IMP-101) ──
# The workflow gained a MECHANISM, not a wording fix: step 0, the conditional third gate,
# and the deps-evidence rule must reach consumers, not just the source tree.
t14_reconcile_entry_and_deps_gate() {
  ensure_payload || { fail t14 "build.sh failed"; return; }
  local f
  for f in "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" \
           "$TMP/payload/cuo/ship-tasks.md"; do
    [ -s "$f" ] || { fail t14 "missing $f"; return; }
    grep -q '^## Reconcile entry' "$f"             || { fail t14 "$f: reconcile entry section missing"; return; }
    grep -q 'no ship-manifest exists OR' "$f"      || { fail t14 "$f: trigger condition missing"; return; }
    grep -q 'NEVER executes a branch' "$f"         || { fail t14 "$f: no-silent-execution rule missing"; return; }
    grep -q '^## depends_on evidence gate' "$f"    || { fail t14 "$f: deps gate section missing"; return; }
    grep -q 'MUST carry evidence' "$f"             || { fail t14 "$f: deps evidence MUST missing"; return; }
    grep -q 'step: 0,  skill: task-reconcile' "$f" || { fail t14 "$f: chain step 0 missing"; return; }
    grep -q '^workflow_version: 2\.8\.0$' "$f"     || { fail t14 "$f: version not 2.8.0"; return; }
  done
  ok t14
}

t18_entered_via_contract() {
  # TASK-IMP-108 §1.4 + §1.5. entered_via is written to FRONTMATTER by the agent in the same edit
  # that moves the status cell - backlog-mutate deliberately never touches frontmatter (it writes
  # rows). So the contract lives in the skill, and this asserts the contract STRUCTURALLY, the way
  # TASK-IMP-104's t05 pins the single comparator. A contract nobody checks is a suggestion.
  ensure_payload || { fail t18 "build.sh failed"; return; }
  local f
  for f in "$repo/modules/skill/backlog-state-update-author/SKILL.md" \
           "$TMP/payload/cuo/skills/backlog-state-update-author/SKILL.md"; do
    [ -s "$f" ] || { fail t18 "missing $f"; return; }
    grep -q 'entered_via: audit | rework | spec_rejected | null' "$f" || { fail t18 "$f: entered_via not in the envelope"; return; }
    grep -q "sets \`entered_via: rework\`" "$f"                      || { fail t18 "$f: rework path does not set entered_via"; return; }
    grep -q "sets \`entered_via: spec_rejected\`" "$f"               || { fail t18 "$f: spec_rejected path missing"; return; }
  done
  ok t18_entered_via_contract
}

t19_spec_rejected_lands_draft() {
  # §1.5: a wrong SPEC returns to draft, NOT ready_to_implement. Routing it to ready_to_implement
  # hands an unchanged wrong spec to an implementer, who builds the same wrong thing.
  ensure_payload || { fail t19 "build.sh failed"; return; }
  local f
  for f in "$repo/modules/skill/contracts/task/STATUS-REFERENCE.md" \
           "$TMP/payload/cuo/skills/contracts/task/STATUS-REFERENCE.md"; do
    [ -s "$f" ] || continue     # the contract is vendored under more than one root; check what exists
    grep -q 'SPEC REJECTED' "$f"      || { fail t19 "$f: spec_rejected route missing"; return; }
    grep -qE '\| \*\*.draft.\*\* \(with .routed_back_count \+= 1., .entered_via: spec_rejected.\)' "$f" \
      || { fail t19 "$f: spec_rejected does not land at draft"; return; }
    grep -q 'entered_via' "$f"        || { fail t19 "$f: entered_via not documented"; return; }
    grep -q 'draft_reason' "$f"       || { fail t19 "$f: draft_reason not documented"; return; }
  done
  # the ceiling itself (§1.6) - 18 increments, zero reads, until now
  local w="$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md"
  grep -q '^## 11b. Route-back ceiling' "$w"          || { fail t19 "$w: no route-back ceiling section"; return; }
  grep -q 'routed_back_count >= 3.*MUST HALT' "$w"    || { fail t19 "$w: ceiling is not a MUST HALT"; return; }
  ok t19_spec_rejected_lands_draft
}

want t01 && t01_manifest_lifecycle
want t02 && t02_two_phase_atomic
want t03 && t03_verify_staleness_exits
want t04 && t04_flip_and_drift_refusal
want t05 && t05_insert_uniqueness_and_grammar
want t06 && t06_counts_maintained
want t07 && t07_json_and_determinism
want t08 && t08_payload_and_install
want t09 && t09_doctrine_wiring
want t10 && t10_retally_corrects_lying_header
want t11 && t11_footprint_holds_with_retally
want t12 && t12_doctrine_view_rules_vendored
want t13 && t13_queue_rule_p0_p3
want t14 && t14_reconcile_entry_and_deps_gate
want t18 && t18_entered_via_contract
want t19 && t19_spec_rejected_lands_draft

echo "test_workflow_helpers: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
