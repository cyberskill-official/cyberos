---
artefacts: phase bundle + coverage-gate@1 + code-review@1 (bundled)
task_id: TASK-CUO-205
tests_failed: 0
tests_passed: 8
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdicts: all pass - human verdict pending at HITL gate 1
---
# Ship artefact bundle - TASK-CUO-205

## Context map / plan (condensed)
Domain: modules/skill/backlog-state-update-* + the create-tasks command doc. files_outside_immediate_domain: 1 (command doc) -> no ADR. Sequencing note honored: TASK-SKILL-118 has not shipped, so the BSU-INS rules land in the audit SKILL.md rubric table (its live home) and migrate to a standalone RUBRIC.md when 118 deepens the pair - exactly the §7 fallback.

## Verification (CASE-01..08 - executable case table at
modules/skill/backlog-state-update-author/acceptance/INSERT_ROW_CASES.md) CASE-08 executed LIVE against the real BACKLOG.md: all 338 task rows round-trip byte-identical (remove + re-insert per §2b == original file). Doc assertions: command step 3 delegates to the pair, inline-edit wording gone; author description 883 chars (under the 1024 host limit); ship workflow doc diff-clean (AC 8).

## §1 clause -> evidence
| #1 @2 enum closed + @1 transition window | author §3 + audit transition note + CASE-07 | passed |
| #2 insert payload + expected_absent uniqueness gate | schema + BSU-INS-001 + CASE-03 | passed |
| #3 regenerator-identical grammar + placement | §2b + CASE-08 live 338/338 | passed |
| #4 BSU-INS-001..005 rubric rules | audit rubric @2.0 table | passed |
| #5 command doc delegates, no inline edit | step-3 rewrite + negative grep | passed |
| #6 ship path unchanged | AC 8 diff-clean + status-cell rules untouched in semantics | passed |

## Findings fixed DURING implementation (caught by the executable case - the task working as designed)
1. Sort key: regen_backlog() sorts by task STEM tuple, not the rendered row string (the [status] prefix would reorder rows). §2b corrected before first commit.
2. Block bounds: the blank line between a section header and its rows sits OUTSIDE the contiguous row block; the naive walk-to-header algorithm diverged. §2b corrected.
3. Pre-existing defect fixed in passing: the audit's BSU-001 rule still enumerated the RETIRED status vocabulary ("[FAILED: ...]" era); now the 10-value enum (rubric bumped to @2.0).

## Reviewer attention points
1. backlog-state-update@2 + backlog_state_update_rubric@2.0 are contract version bumps; the @1 acceptance window closes one release after this ships (noted in both files).
2. The regenerator (scripts/migrate_improvement_to_task.py regen_backlog) remains the byte authority - §2b cross-cites it in both directions.

## Verdict requested
Review acceptance (HITL gate 1): approve to advance, or reject with findings.
