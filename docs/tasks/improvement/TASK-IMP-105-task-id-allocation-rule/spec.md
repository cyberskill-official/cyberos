---
id: TASK-IMP-105
title: Task-id allocation rule + next-id helper
template: task@1
type: improvement
module: improvement
status: ready_to_test
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-085, TASK-IMP-092]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 2
service: modules/skill
new_files:
  - (none)
modified_files:
  - modules/skill/task-author/SKILL.md
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/tests/test_workflow_helpers.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §10 IMP-26"
  - "modules/skill/task-author/SKILL.md (no allocation rule present on main bb231900)"
  - "tools/install/docs-tools/backlog-mutate.mjs:258 (uniqueness pre-image refusal, exit 7 - the late net)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-105: Task-id allocation rule + next-id helper

## Summary

`task-author` has no rule for choosing a task id - the model picks, and the only safety net is `backlog-mutate`'s uniqueness gate, which fires at INSERT after the spec folder is already on disk. Two authoring runs against one module can therefore both pick the same id, write two folders, and leave an orphan when the second insert refuses. State the allocation rule in the skill and make it executable as `backlog-mutate next-id <module>`.

## Problem

`grep` for an allocation rule in `task-author/SKILL.md` finds nothing: no "scan existing ids", no "next available". The uniqueness pre-image gate in `backlog-mutate.mjs:258` refuses with exit 7 (`row already present ... uniqueness pre-image violated`) - correct, but late: the spec files exist by then, and the operator is left with a half-landed task and no instruction.

Authoring is usually serial, which is why this has not bitten. This run interleaved five batches and got lucky. The pattern this run keeps re-learning is that every mechanical rule left in prose gets re-derived wrongly eventually, and every one moved into a tool stops being a question.

## Proposed Solution

Add the rule to `task-author`: allocate by scanning `docs/tasks/<module>/` for the highest existing stem and taking the next, and re-scan immediately before writing rather than trusting an id chosen at PLAN time. Add `backlog-mutate next-id <module>` so the rule is executable rather than remembered - it reads the same corpus the insert gate reads, so allocation and admission cannot disagree. The gate stays exactly as it is: this narrows the window, it does not replace the check.

## Alternatives Considered

- A lock around authoring. Rejected: heavier than the defect, and it would serialise an operation that is legitimately parallel across different modules.
- Random or timestamp ids. Rejected: the corpus's id-ascending ordering is load-bearing for the regenerator's row grammar and for humans reading the backlog.
- Rely on the insert gate alone (status quo). Rejected: it refuses correctly but only after the files exist, which converts a preventable collision into a cleanup.

## Success Metrics

- Primary: `next-id` returns the correct next stem for a populated module, an empty module, and a module with gaps - suite-asserted. Baseline: no allocation rule exists at all.
- Guardrail: the uniqueness gate still refuses a duplicate insert (this task must not weaken it), asserted by the existing exit-7 arm.

## Scope

In scope: the allocation rule in `task-author/SKILL.md`, `backlog-mutate next-id <module>`, suite arms.

### Out of scope / Non-Goals

- Any change to the uniqueness gate's behavior - it remains the authority.
- Cross-module id coordination (ids are per-module by construction).
- Reserving ids ahead of authoring (a queue this backlog does not need).

## Dependencies

None logically.

**Serialisation note:** touches `backlog-mutate.mjs` (shared with TASK-IMP-108, which adds `entered_via` to the same writer). Parent-serialised per §11a.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md IMP-26, verified against task-author/SKILL.md and backlog-mutate.mjs on merged main; implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `task-author/SKILL.md` MUST state the allocation rule: the next id for a module is the highest existing stem in `docs/tasks/<module>/` plus one, and it MUST be re-scanned immediately before writing files rather than reused from PLAN time.
- 1.2 `backlog-mutate` MUST expose `next-id <module>` printing the next free stem to stdout and exiting 0.
- 1.3 `next-id` MUST derive from the same corpus the insert gate reads, so an id it returns cannot be rejected by the gate for non-uniqueness in the same instant.
- 1.4 `next-id` on a module with no tasks MUST return that module's first stem and exit 0 (an empty module is not an error).
- 1.5 `next-id` MUST ignore gaps: it returns highest+1, never the lowest free number, because reusing a retired id makes two different tasks share a name in the history.
- 1.6 The uniqueness pre-image gate MUST remain unchanged and MUST remain the authority on admission.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.2, #1.3) - `next-id improvement` on the live corpus returns the highest+1 stem and exits 0 - test: `tools/install/tests/test_workflow_helpers.sh::t15_next_id_populated`
- [ ] AC 2 (traces_to: #1.4) - `next-id <empty-module>` returns the first stem and exits 0 - test: `tools/install/tests/test_workflow_helpers.sh::t16_next_id_empty_module`
- [ ] AC 3 (traces_to: #1.5) - a corpus with a gap yields highest+1, not the gap - test: `tools/install/tests/test_workflow_helpers.sh::t17_next_id_ignores_gaps`
- [ ] AC 4 (traces_to: #1.6) - the existing exit-7 uniqueness refusal still fires unchanged - test: `tools/install/tests/test_workflow_helpers.sh::t07_insert_uniqueness_refusal`
- [ ] AC 5 (traces_to: #1.1) - the skill states the rule including the re-scan-before-write requirement - verify: recorded grep in the gate log (prose contract; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- Folder present on disk but no BACKLOG row (a half-landed task from the exact collision this fixes): `next-id` MUST count the folder - the folder is the task, the row is the index, and skipping it would hand out the colliding id again.
- Malformed stem in the module (hand-created folder not matching the grammar): skip it with a note on stderr; one bad folder must not stop allocation.
- Two `next-id` calls racing: both return the same stem. This narrows the window, it does not close it - the gate remains the authority (1.6), and this is why 1.6 exists.
- A module whose name does not yet exist as a directory: treated as empty per 1.4.
- Security-class: reads directory names, prints a stem. No untrusted content is executed; the module argument MUST be confined under `docs/tasks/` on the same `relUnderRoot` rule the other helpers use, so a crafted `../` argument cannot walk out.
