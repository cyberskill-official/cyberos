---
task_id: TASK-IMP-108
artefact: coverage-gate@1
phase: testing
generated: 2026-07-17
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
coverage_measurable: partial
coverage_reason: "Touched files span JS (task-lint.mjs, render-status-hub.mjs) and prose contracts (STATUS-REFERENCE, ship-tasks, SKILL.md). The JS is exercised by its suites but this repo runs no c8/lcov over docs-tools, so no percentage is reported. Prose contracts are pinned structurally instead - the same discipline TASK-IMP-104's t05 uses."
trace_004_closed: true
---

# TASK-IMP-108 - coverage gate

## TRACE-004 closure - every clause's cited test PASSED

| AC | traces_to | cited test | result |
|---|---|---|---|
| AC 1 | #1.1, #1.2 | `test_task_lint.sh::t09_optional_status_reason_enums` | passed |
| AC 2 | #1.3, #1.4 | `test_workflow_helpers.sh::t18_entered_via_contract` | passed |
| AC 3 | #1.5 | `test_workflow_helpers.sh::t19_spec_rejected_lands_draft` | passed |
| AC 4 | #1.6 | `test_workflow_evolution.py::test_routeback_ceiling_halts` | passed |
| AC 5 | #1.6 | `test_workflow_evolution.py::test_under_ceiling_reenters` | passed |
| AC 6 | #1.7 | `test_render_status_hub.sh::t11_draft_staleness_report` | passed |

All 7 §1 clauses cited; 6/6 ACs closed.

**AC 6's citation was AMENDED during implementation** (`t08_draft_staleness_report` -> `t11_draft_staleness_report`): `t08` is already `t08_spec_chunks` in that suite, so the spec named an occupied slot. Recorded here rather than silently renamed, and accepted by the operator at the review gate (2026-07-17).

## Why some ACs are structural tests, not behavioural ones

AC 2/3/4/5 pin prose contracts (a skill envelope, a status-reference row, a workflow section). `backlog-mutate` deliberately never writes frontmatter - it writes rows - so `entered_via` is set by the agent in the same edit that moves the status cell, and the contract is the artefact. The ceiling is a HALT for a human; a suite cannot simulate the human without becoming a test of its own fixture.

Structural does not mean weak: t18/t19 and the four ceiling arms fail if the doctrine is deleted, reworded past recognition, or left out of the payload. That is the property that matters, and it is the same discipline TASK-IMP-104's `t05_single_comparator` uses to pin `ver_lt` to exactly one definition repo-wide.

## Edge-case matrix coverage (§3)

| Row | Arm | result |
|---|---|---|
| 336 existing drafts carry no `draft_reason` | t09's absent-field arm; the page renders `unknown 336` | covered |
| `routed_back_count: 3` from ONE cause still halts | `test_routeback_ceiling_halts` (counts cycles, not causes) | covered |
| Manual operator flip with no `entered_via` | absent is legal (FM-116 optional); the ceiling still reads the counter | covered |
| `spec_rejected` on mis-cited ACs -> still `draft` | STATUS-REFERENCE §1.3 row; `test_spec_rejected_pairs_with_the_ceiling` | covered |
| Ceiling reached mid-swarm -> the PARENT halts | `test_routeback_ceiling_halts` asserts "halt belongs to the parent" | covered |
| An `authoring` draft 200 days old is reported, never touched | t11 asserts the report changes no status | covered |
| Security-class: two closed enums + a counter comparison | no execution surface; a crafted value reds via FM-115/116 rather than routing | covered by inspection |

## Live evidence

The report on the real corpus: **336 drafts, all `unknown`, oldest 2026-05-16.** Correct and honest - no draft carries a reason, because backfilling one for tasks this run did not author is an explicit Non-Goal. The number nobody believed is now the number with its ignorance stated.

## Suite evidence

```
test_task_lint            9/9   (t09 new - incl. the absent-field arm protecting 115 specs)
test_workflow_helpers    16/16  (t18/t19 new; 9 version pins moved 2.7.0 -> 2.8.0, disclosed)
test_workflow_evolution  12/12  (4 ceiling arms new)
test_render_status_hub   11/11  (t11 new)
test_chain_coverage       7/7   test_full_sdp_payload   9/9
test_pair_parity          6/6   test_rubrics_vendored   2/2
test_install_lock         7/7
build OK; sync OK 1.0.0 across 7 artifacts; page byte-identical on re-render (082 holds)
```
