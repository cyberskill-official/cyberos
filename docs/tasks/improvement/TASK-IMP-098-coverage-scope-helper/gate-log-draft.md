# TASK-IMP-098 gate-log evidence (implementing -> ready_to_review)

E1 - gating suite (AC 1-4), full run, verbatim:
```
$ bash tools/install/tests/test_coverage_scope.sh
  ok   t01
  ok   t02
  ok   t03
  ok   t04
test_coverage_scope: pass=4 fail=0
```

E2 - AC 4 glob half (run_all discovery): scripts/tests/run_all.sh:43's
`tools/install/tests/test_*.sh` glob picks up
`tools/install/tests/test_coverage_scope.sh` by name - zero wiring (same ops check
as the batch siblings; the expansion was verified at authoring time).

E3 - vendor line: tools/install/build.sh:180-181 (guarded copy, sibling idiom, directly
under TASK-IMP-093's line in the shared docs-tools block):
```
  # coverage-scope: task diff -> per-file coverage skeleton (TASK-IMP-098)
  [ -f "$here/docs-tools/coverage-scope.mjs" ] && cp "$here/docs-tools/coverage-scope.mjs" "$out/docs-tools/"
```
t04 gates payload presence, byte-parity with the source, --help of the vendored copy,
and a live vendored run against a scratch fixture repo.

E4 - shared-file regression check after the build.sh edit (both suites re-run green):
```
test_memory_append: pass=4 fail=0
test_workflow_helpers: pass=13 fail=0
```

E5 - AC 5 (guardrail: sachviet batch-1 per-file tables reproduced) is PARENT-RUN:
consumer-repo evidence the fixture suite cannot carry. The parent runs the tool
against the shipped sachviet batch (recorded gates in docs/tasks/web/*/coverage-gate.md
of that repo) and appends the recorded output to this gate log. Not claimed here.

## AC 5 - sachviet reproduction (parent-run, 2026-07-17)

Worktree of sachviet `batch/1-web-workspace` (removed after); `npm install` + `vitest run
--coverage --coverage.reporter=json-summary`; then the PAYLOAD copy:

  node dist/cyberos/docs-tools/coverage-scope.mjs TASK-WEB-002 --repo <worktree> \
    --coverage app/web/coverage/coverage-summary.json

Output matched the recorded batch-1 gates: base auto-resolved to the real entry-flip commit
5a647cf ("chore(TASK-WEB-002,003): enter implementing ...") with the two-subject ambiguity
note emitted as specced; `app/web/src/utils/money.ts | 100 | ok` and
`app/web/src/domain/primary-vendor.ts | 100 | ok` reproduce the recorded per-file tables
(money.ts and primary-vendor.ts at 100 across all columns in the committed coverage-gate.md
files); `files_below_90pct: []` matches both recorded verdicts. Docs/test files in the range
surfaced as visible no-coverage-data rows, never dropped. The consumer repo was left
byte-untouched (worktree removed; `git status` clean).
