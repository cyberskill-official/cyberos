---
artefact: implementation-plan@1
task_id: TASK-IMP-098
created: 2026-07-17
estimate_pts: 2
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 4)
---
# Implementation plan - TASK-IMP-098

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Read-only git plumbing + base resolution in tools/install/docs-tools/coverage-scope.mjs -
   spawnSync argv-array git (log/diff/ls-tree/rev-parse only), findRoot walk-up,
   `resolveBase`: --base verified via rev-parse wins; else `git log --format=%H %s`
   scanned for subjects naming <task-id> + "implementing", EARLIEST (last emitted line)
   wins with the ambiguity recorded for the range note; else Refusal(3) demanding
   --base - never a guessed range (§1.1; rows 7, 10, 11).
2. Touched-file set - `git diff --name-only <base>...HEAD` split, membership-tested
   against `git ls-tree -r --name-only HEAD`; present -> table (bytewise sort),
   absent -> deletions list for the notes line (§1.2; rows 8, 12).
3. Coverage ingestion - recognition BY NAME (basename): coverage-summary.json ->
   ingestIstanbul (JSON with a 'total' key required; per-file lines.pct; non-number
   pct dropped; absolute keys normalized repo-relative), lcov.info -> ingestLcov
   (SF/LF/LH state machine, pct = LH/LF*100 round-2, LF:0 = 100, zero SF records
   refused); any other name -> Refusal(4) naming the two supported shapes; default
   discovery coverage/coverage-summary.json then coverage/lcov.info; --coverage must
   resolve inside the repo root (§1.3; rows 3, 5, 6, 9).
4. Skeleton emitter - coverage-gate@1 frontmatter (artefact/task/phase: testing,
   tests_failed: TODO, files_below_90pct computed STRICT <90, ecm_rows_uncovered:
   TODO), range line with full base...HEAD shas + resolution provenance + optional
   ambiguity note, report line naming path + shape, per-file table with
   `no-coverage-data` rows, deletions note, no-data count note, author-skill TODO
   line; stdout or --out (inside the repo root, stderr confirmation) (§1.4; rows 1,
   2, 4).
5. Gating suite tools/install/tests/test_coverage_scope.sh (t01-t04 per the spec's AC
   names; scratch git fixture repos built in-test with the entry-flip subject
   convention, isolated from user/system git config; t02 is an expected-BYTES compare
   with sha placeholders filled by the test) + the 2-line guarded vendor copy in
   build.sh (§1.5, §1.6; rows 6, 8, 12).

Pattern conformance (context-map): node stdlib only (node:fs, node:child_process,
node:path), single ESM file, whole-file doc comment, loud refusals with a documented
exit-code table, deterministic output (no clock, no randomness). Out of scope honored:
does not run coverage, does not fill judgment fields, no other report formats, no
run-gates or ship-manifest changes.

Estimate: 2 pts (~4 h) - matches spec effort_hours: 4. Actual landed surface: 2 new
files (coverage-scope.mjs 338 lines, test_coverage_scope.sh 191 lines), 1 modified
(build.sh +2, zero deletions), suite 4/4 in ~3 s including the payload build; sibling
suites (test_memory_append 4/4, test_workflow_helpers 13/13) re-run green after the
shared build.sh edit.
