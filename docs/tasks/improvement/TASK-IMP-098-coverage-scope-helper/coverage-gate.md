---
artefact: coverage-gate@1
task: TASK-IMP-098
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-098 coverage gate

Raw terminal: `bash tools/install/tests/test_coverage_scope.sh` -> t01_base_resolution ok, t02_skeleton_from_fixture ok, t03_unknown_report_refused ok, t04_payload_vendored ok - `test_coverage_scope: pass=4 fail=0`.

| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs-tools/coverage-scope.mjs | t01-t04 (base wins/scan/fail, both shapes, 90-boundary, no-data row, deletion note, refusal, payload lifecycle) | all outcome branches |
| tools/install/build.sh (2 guarded lines) | t04 scratch build | 2/2 |

TRACE-004: 1.1 -> t01 passed; 1.2+1.3+1.4 -> t02 passed; 1.3 refusal -> t03 passed; 1.5+1.6 -> t04 passed + runner glob recorded. AC 5 guardrail: the PAYLOAD copy reproduced sachviet batch-1's recorded per-file tables (money.ts 100, primary-vendor.ts 100, files_below_90pct []) from a throwaway worktree, base auto-resolved to the real entry-flip commit 5a647cf with the ambiguity note; consumer repo left byte-untouched (gate-log AC 5 section carries the full run). ECM rows uncovered: none.
