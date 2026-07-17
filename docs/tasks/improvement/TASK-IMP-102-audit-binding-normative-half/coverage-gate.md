---
artefact: coverage-gate@1
task: TASK-IMP-102
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-102 coverage gate

Raw terminal: `bash tools/install/tests/test_task_reconcile.sh` -> `test_task_reconcile: pass=6 fail=0`
(t06_body_binding_preferred: flip-proof, drift-caught, legacy-honest, legacy-dishonest).

| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs-tools/task-reconcile.mjs (R1 preference) | t06 x4 arms | 4/4 binding paths |
| modules/skill/task-audit/SKILL.md (§12 + 3 field re-statements) | AC 4 recorded greps | n/a - prose contract |
| tools/install/tests/test_task_reconcile.sh | is the coverage | n/a |

TRACE-004: 1.4/1.5 -> t06 passed (AC 1-3); 1.1/1.2/1.3 -> AC 4 recorded greps (gate log E2,
`grep -c audited_body_sha256` = 4 across payload_hash_field, fixity_notes, re_entrancy, §12).

ECM rows uncovered: none. Rows 7 (future lifecycle field) and 9 (security) are documented
judgments, not testable states; row 8 rides t04's existing unverifiable-binding arm.

Self-witness: this task's own audit.md carries `audited_body_sha256_prefix: 5c530084993c87d5`
and stayed R1-green through the batch's own draft -> ... -> testing flips - the property the
whole-file field could never have.
