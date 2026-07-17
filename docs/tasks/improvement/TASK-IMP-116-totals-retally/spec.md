---
id: TASK-IMP-116
title: Every mutation emits the whole file's truth, not just its section's
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T20:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-086, TASK-IMP-092, TASK-IMP-105]
routed_back_count: 0
entered_via: audit
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/docs-tools/backlog-mutate.mjs
  - modules/skill/backlog-state-update-author/SKILL.md
  - tools/install/tests/test_workflow_helpers.sh
source_pages:
  - "Greptile review 2026-07-17 (P1, docs/tasks/BACKLOG.md:10): the header no longer matches the rows below it - line 10 reported 336 draft, 4 ready_to_implement, 176 done while the improvement section alone read 67 draft, 9 ready_to_implement, 39 done. Consumers using the backlog header report incorrect queue counts."
  - "tools/install/docs-tools/backlog-mutate.mjs:146 (TASK-IMP-092's own rationale: 'incremental adjustment faithfully preserves an inherited lie forever (the 086 incident's 34 vs true 20), while the retally makes every mutation emit the section's truth') - the argument that stops one line short of the file's most-read number"
  - "tools/install/tests/test_workflow_helpers.sh t06 ('the file-top Totals line is NEVER touched (not part of the declared mutation)'), t07 + t11 (flip footprint pinned at 1 row + 1 header) - the contract this task amends"
  - "scripts/migrate_improvement_to_task.py:221 (regen_backlog writes Totals) - and it cannot run: FileNotFoundError docs/improvement/memory/backlog.yaml, retired in the 2026-07-08 migration. Nothing maintains the line today."
source_decisions:
  - "2026-07-17 Stephen: extend the contract - mutate retallies Totals; as a task with both gates, not a pin move (recorded HITL answer)."
---

# TASK-IMP-116: Every mutation emits the whole file's truth, not just its section's

## Summary

`backlog-mutate` retallies the section header of the row it touches and leaves the file-top `Totals:` line alone, by an explicit contract three tests pin. That line is the first number any reader sees, and it has no maintainer: the regenerator that owns it cannot run. Widen the declared mutation from two lines to three, so every write emits the whole file's truth - the completion of TASK-IMP-092's own argument.

## Problem

An external review found `Totals: 336 draft, 4 ready_to_implement, 176 done` at line 10 of a file whose improvement section alone read `67 draft, 9 ready_to_implement, 39 done`. The index of truth had a lying header. Consumers reading the header report wrong queue counts.

TASK-IMP-092 replaced incremental +1/-1 header adjustment with a full retally, and its rationale is decisive: *"incremental adjustment faithfully preserves an inherited lie forever (the 086 incident's 34 vs true 20), while the retally makes every mutation emit the section's truth."* It then stopped at section headers. The same lie was free to grow one line higher, and it did.

The line has no owner. `regen_backlog` writes it and cannot run - it reads `docs/improvement/memory/backlog.yaml`, retired in the 2026-07-08 migration. So the only mechanism that maintains `Totals:` is a script that raises FileNotFoundError.

## Proposed Solution

`retallyTotals(lines)`: find `^Totals: `, count every parseable row in the file by status, rewrite the line in the file's own convention. Called by both `flip` and `insert` after the section retally, reported in the result as `totals_line`. Absent or unparseable Totals returns null and is not an error - a file without the line is legal.

The contract widens deliberately: a `backlog-state-update@2` write now declares **one status cell + its section header + the file Totals**. That is the amendment this task exists to make, and it is the reason this is a task rather than an edit: three tests pin the current boundary and the skill's contract states it.

## Alternatives Considered

- **Fix the regenerator instead** (keep the footprint at two lines). Rejected on the operator's call: it leaves the line correct only when someone remembers to regenerate, which is precisely the "nobody ran it" failure that let the 086 rows sit wrong. A number maintained by a ritual is a number that rots between rituals.
- **Leave it; it is correct on disk today.** Rejected: it is correct by accident (one flip ran before the fix was reverted) and re-rots on the next mutation.
- **Move the pins without amending the contract.** Rejected explicitly: t06/t07/t11 encode the byte-discipline boundary of the audited write path. Editing that boundary by rewriting its tests is how a gate quietly stops gating.

## Success Metrics

- Primary: after any flip or insert, `Totals:` equals an independent count of the file's rows - suite-asserted, and true for every status. Baseline: the line is maintained by nothing and was wrong by 4 ready_to_implement and 4 done when the review found it.
- Guardrail: the mutation stays minimal and auditable - exactly 3 lines change (row, section header, Totals) and never a fourth.

## Scope

In scope: `retallyTotals` in backlog-mutate, its wiring into flip + insert, the `backlog-state-update@2` contract amendment, the three pinned tests.

### Out of scope / Non-Goals

- Repairing `regen_backlog` (a real defect, its own task - this task makes the mutation self-sufficient, which is what removes the urgency).
- Any change to row grammar, placement, or the uniqueness gate.
- Retallying anything else in the file (a `Totals:` line is the only global counter; inventing more is not this task's job).

## Dependencies

None. `retallyHeader`, `parseRow`, and `STATUS_ORDER` all exist.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the 2026-07-17 Greptile P1 and the suite's refusal of a first, unamended fix attempt; implementation under ship-tasks supervision.
- **Human review:** the contract-extension decision is the operator's recorded 2026-07-17 answer; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `backlog-mutate` MUST expose `retallyTotals(lines)`, rewriting the `^Totals: ` line from a full count of EVERY parseable row in the file, in the file's own rendering convention (lifecycle order, zero-count statuses omitted).
- 1.2 `flip` MUST call it after its section retally, and MUST report the rewritten line as `totals_line` in its result and message.
- 1.3 `insert` MUST call it on the same terms - a stale global total is stale whichever mutation caused it.
- 1.4 An absent or unparseable `Totals:` line MUST return null and MUST NOT be an error: a backlog without the line is legal and MUST NOT be given one.
- 1.5 Rows whose status token is outside the enum MUST NOT be counted, matching `retallyHeader` and `regen_backlog` - one counting rule, three callers.
- 1.6 The declared mutation footprint MUST be exactly three lines (row, section header, Totals) and MUST NOT grow further; `backlog-state-update-author`'s contract MUST state the new footprint.
- 1.7 A mutation whose Totals line is already correct MUST leave it byte-identical and MUST NOT report `totals_line` - a no-op is not a write.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - after a flip, Totals equals an independent count of the file's rows, and the result reports `totals_line` - test: `tools/install/tests/test_workflow_helpers.sh::t06_counts_maintained`
- [ ] AC 2 (traces_to: #1.3) - after an insert, the same holds - test: `tools/install/tests/test_workflow_helpers.sh::t06_counts_maintained`
- [ ] AC 3 (traces_to: #1.6) - the flip footprint is exactly 3 lines: row + section header + Totals, never a fourth - test: `tools/install/tests/test_workflow_helpers.sh::t07_json_and_determinism`
- [ ] AC 4 (traces_to: #1.4) - a backlog with no Totals line mutates cleanly and gains no Totals line - test: `tools/install/tests/test_workflow_helpers.sh::t11_footprint_holds_with_retally`
- [ ] AC 5 (traces_to: #1.7) - a mutation against an already-correct Totals leaves it byte-identical and reports no `totals_line` - test: `tools/install/tests/test_workflow_helpers.sh::t06_counts_maintained`
- [ ] AC 6 (traces_to: #1.5) - a row carrying an out-of-enum status is excluded from the Totals count - test: `tools/install/tests/test_workflow_helpers.sh::t06_counts_maintained`
- [ ] AC 7 (traces_to: #1.6) - the skill's contract states the three-line footprint - verify: recorded grep in the gate log (a prose contract; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- A file with TWO `Totals:` lines (hand-edited): the FIRST is rewritten and the second left alone - the tool must not guess which one is canonical, and a second is a corpus defect for a human to see, not for a mutation to silently resolve.
- A `Totals:` line inside a fenced code block (a doc example): `^Totals: ` at column 0 inside a fence would match. Accepted risk, named: BACKLOG.md is a generated index, not prose, and a fence at column 0 there is already a corpus defect. Revisit if a real file trips it.
- A backlog whose rows are all out-of-enum: the tally is empty; per 1.4's shape the line is left alone rather than rendered as `Totals: ` with nothing after it.
- Placeholder-only sections (`(none yet)`): parse as no row, count nothing - the same semantics `retallyHeader` already has.
- Concurrency: mutations are serialised by the workflow, and `atomicWrite` already governs the file. Widening the write from 2 lines to 3 does not change that; the whole file is rewritten either way.
- Security-class: counts rows already in the file and rewrites one line. No untrusted content is executed; no path is taken from input.
