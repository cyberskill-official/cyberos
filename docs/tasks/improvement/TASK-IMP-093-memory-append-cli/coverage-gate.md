---
artefact: coverage-gate@1
task: TASK-IMP-093
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-093 coverage gate

Raw terminal (full suite): `bash tools/install/tests/test_memory_append.sh` -> t01_fresh_store_three_appends ok, t02_verify_and_tamper ok, t03_bad_kind_refused ok, t04_payload_vendored ok - `test_memory_append: pass=4 fail=0`.

| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs-tools/memory-append.mjs | t01-t04 (append, bootstrap, stale-tmp heal, verify, tamper, lease, kind/JSON refusal, payload lifecycle) | all outcome branches |
| tools/install/build.sh (2 guarded lines) | t04 scratch build | 2/2 |
| tools/install/tests/test_memory_append.sh | is the coverage | n/a |

TRACE-004: 1.1+1.3 -> t01 passed; 1.4 -> t02 passed; 1.2 -> t03 passed; 1.5 -> t04 passed; 1.6 -> AC 5 ops (runner glob lists the suite; recorded in gate log). ECM rows uncovered: none (stale tmp, held lease, expired lease, non-JSON, store-escape refusal all asserted). Cross-implementation evidence: an independent Python walk (writer.py logic) recomputed a tool-written store's chain clean (gate-log E4).
