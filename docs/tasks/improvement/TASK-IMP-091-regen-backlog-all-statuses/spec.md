---
id: TASK-IMP-091
title: regen_backlog emits every status and recomputes Totals from frontmatter
template: task@1
type: improvement
module: improvement
status: implementing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T17:25:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-086]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 3
service: scripts
new_files:
  - scripts/tests/test_regen_backlog.sh
modified_files:
  - scripts/migrate_improvement_to_task.py
source_pages:
  - "scripts/migrate_improvement_to_task.py:19-20 and :201 (ACTIVE-status filter in regen_backlog - emits no row for done tasks and deletes existing done rows on regen)"
  - "TASK-IMP-086 gate-log E1 (the recorded regenerator trial that proved it: zero rows for the fourteen done tasks, batch-1 done rows deleted, Totals drifting 155 vs 158)"
  - "IMPROVEMENT_HANDOFF.md IMP-17"
source_decisions:
  - "2026-07-16 Stephen: batch 3 PLAN approved with this item at p1."
---

# TASK-IMP-091: regen_backlog emits every status and recomputes Totals from frontmatter

## Summary

The function every insert contract cites as the byte authority cannot regenerate a truthful index: its ACTIVE-status filter drops done rows entirely - the proven root cause of the fourteen-task drift TASK-IMP-086 repaired. Make regen_backlog emit one row per task folder for every status, recompute the repo-wide Totals line from a frontmatter tally, and gate it with a parity test that regenerates today's corpus and byte-compares the improvement section against the committed file.

## Problem

TASK-IMP-086's mandated trial recorded the failure precisely: regen deleted the three committed done rows, emitted nothing for 068-081 (all done), and left a Totals line three short of the corpus. An index regenerator that silently drops terminal statuses turns the audit trail into fiction the moment tasks ship.

## Proposed Solution

Remove the ACTIVE filter from row emission (the index lists the whole corpus; the queue's eligibility logic stays elsewhere), keep grammar and stem ordering byte-identical to the committed convention, and compute Totals from a per-status frontmatter tally at regen time. A new suite regenerates against the live corpus into a scratch copy and asserts (a) byte-parity of the improvement section with the committed file and (b) Totals equal to the tally.

## Alternatives Considered

- Re-point the byte-authority citation at backlog-mutate.mjs and retire regen. Rejected for now: full-section regeneration from frontmatter remains the recovery tool for exactly the drift class 086 hit; fixing it is one filter.
- Emit done rows into a separate archive section. Rejected: the committed convention (086's repair, reviewed and merged) lists them in place with the status tag.

## Success Metrics

- Primary: regen on today's corpus reproduces the committed improvement section byte-for-byte and a true Totals line, asserted by the suite on every run. Baseline: regen output deletes 17 rows and undercounts Totals by 3 (recorded trial). Deadline: final acceptance.
- Guardrail: a fixture corpus with one task per status yields one row per status (the filter cannot silently return).

## Scope

In scope: the emission filter, the Totals computation, the parity suite.

### Out of scope / Non-Goals

- Other sections' drift repair (regen fixes the mechanism; running it repo-wide is an operator choice).
- The migrate/adopt phases of the script (row regeneration only).

## Dependencies

- None; cone is the python script plus a new scripts/tests suite (disjoint from all batch members).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded TASK-IMP-086 trial evidence; implementation under ship-tasks supervision.
- **Human review:** batch-3 PLAN approved 2026-07-16; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 regen_backlog MUST emit exactly one row per task folder for EVERY frontmatter status (the ACTIVE filter is removed from emission), preserving the committed row grammar and stem-ascending order.
- 1.2 The Totals line MUST be recomputed from a per-status tally of the corpus frontmatter at regen time.
- 1.3 Regenerating today's corpus MUST reproduce the committed improvement section byte-for-byte (rows and header), proven in the suite against a scratch copy.
- 1.4 A fixture corpus containing one task in each lifecycle status MUST yield one row per status.
- 1.5 The suite MUST land at scripts/tests/test_regen_backlog.sh, discovered by the run_all glob.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - live-corpus regen is byte-identical to the committed section - test: `scripts/tests/test_regen_backlog.sh::t01_live_corpus_parity`
- [ ] AC 2 (traces_to: #1.2) - Totals equals the frontmatter tally - test: `scripts/tests/test_regen_backlog.sh::t02_totals_true`
- [ ] AC 3 (traces_to: #1.4) - per-status fixture emits every row - test: `scripts/tests/test_regen_backlog.sh::t03_every_status_emitted`
- [ ] AC 4 (traces_to: #1.5) - suite discovered by run_all - verify: suite listed in the runner output (ops check in the gate log; glob discovery is the runner's contract).

## 3. Edge cases

- Titles containing the row separator or other task ids (081's title quotes TASK-IMP-080): emitted verbatim; parity is byte-level so any mangling fails t01.
- A folder with unparseable frontmatter: regen MUST fail loudly naming the file, never emit a guessed row (asserted inside t03's fixture set).
- Sections other than improvement: untouched by this task's assertions; the mechanism fix applies wherever the script is pointed.
- Security-class: none - a read-compute-write script over tracked markdown, gated by byte-parity.
