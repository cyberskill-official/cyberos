---
id: TASK-IMP-115
title: Effort tiering - advisory judgment metadata on skill_chain steps
template: task@1
type: improvement
module: improvement
status: ready_to_test
priority: p3
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-084]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 3
service: modules/cuo
new_files:
  - (none)
modified_files:
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - modules/cuo/tests/test_workflow_evolution.py
source_pages:
  - "IMPROVEMENT_HANDOFF.md §11 IMP-31"
  - "How to Build An Agentic OS using Fable 5 (Avid, 2026-07-06) BUILD 3: the conductor emits 10-20 % of tokens while making 100 % of decisions; the model reading the quiet ticks decides the bill"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-115: Effort tiering - advisory judgment metadata

## Summary

Every workflow step runs at whatever the host gives it. Nothing says which steps deserve expensive reasoning (task-audit's judgment half, spike options) and which are near-mechanical (backlog-mutate, coverage-scope - already scripts, correctly). Annotate each `skill_chain` step with `judgment: high | medium | mechanical` as ADVISORY metadata a host may use to route. No model strings, ever.

## Problem

CyberOS is host-agnostic and gives a host no information to route with. The article's numbers are the argument: the same outcome costs $2.56/day or $34/day depending on which model reads the quiet ticks. This run's two spend cutoffs are the same failure with a different face.

The constraint is equally clear: a `claude-fable-5` literal in a skill is a rule that expires. The payload cannot name models, prices, or effort levels - those are the host's facts, accurate the day they are written and wrong soon after.

## Proposed Solution

One optional field per `skill_chain` step: `judgment: high | medium | mechanical`. `mechanical` means a deterministic helper does the work and a model is not deciding anything. `high` means the step's output is a judgment the workflow depends on. A host MAY route on it; nothing in the payload reads it. That is the whole change: information, not instruction.

## Alternatives Considered

- Model strings or effort names per step. Rejected: host-specific and stale on arrival. The payload describes the work, the host picks the worker.
- A cost budget in the workflow. Rejected: a payload that refuses to run because of a number it cannot verify is a payload that gets edited.
- Infer from the step's skill name. Rejected: it is exactly the kind of implicit rule this run keeps finding wrong; if it matters, write it down.

## Success Metrics

- Primary: every `skill_chain` step carries a valid `judgment`, and the mechanical ones are precisely the steps whose work is done by a docs-tools helper - suite-asserted. Baseline: no such information exists.
- Guardrail: no model string, price, or effort name appears anywhere in the payload - suite-asserted as a negative.

## Scope

In scope: the `judgment` field on ship-tasks' skill_chain steps, its documentation, suite arms.

### Out of scope / Non-Goals

- Any routing, model selection, or effort setting - the payload informs, the host decides.
- Model names, prices, or effort levels in the payload.
- Applying the field to create-tasks or plan (extend once this proves useful).

## Dependencies

None. Additive and optional.

**Serialisation note:** touches `ship-tasks.md` (shared with 108, 109, 113, 114) and `test_workflow_evolution.py` (shared with 108). Parent-serialised per §11a.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §11 IMP-31, adopting the guide's insight while rejecting its host-specific encoding.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 Each `skill_chain` step in ship-tasks MUST carry `judgment: high | medium | mechanical`.
- 1.2 `mechanical` MUST mean the step's work is performed by a deterministic helper with no model judgment in the result.
- 1.3 The field MUST be documented as ADVISORY: a host MAY route on it, and nothing in the payload may read it to decide anything.
- 1.4 No model string, price, or effort name may appear in the payload as a result of this task.
- 1.5 A step whose judgment level is genuinely ambiguous MUST be `medium` rather than guessed high - overstating a step's need is how the expensive default returns.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - every skill_chain step carries a value from the enum - test: `modules/cuo/tests/test_workflow_evolution.py::test_every_step_has_judgment`
- [ ] AC 2 (traces_to: #1.2) - every step marked mechanical is one whose skill delegates to a docs-tools helper - test: `modules/cuo/tests/test_workflow_evolution.py::test_mechanical_steps_are_helper_backed`
- [ ] AC 3 (traces_to: #1.4) - no model string, price, or effort literal in the payload - test: `modules/cuo/tests/test_workflow_evolution.py::test_no_host_specific_literals`
- [ ] AC 4 (traces_to: #1.3) - the field is documented as advisory and nothing in the payload reads it - verify: recorded grep in the gate log (a negative structural claim; same rationale as TASK-IMP-090 AC 1).
- [ ] AC 5 (traces_to: #1.5) - no step is marked `high` without a named reason in the review; ambiguous steps carry `medium` - verify: recorded reviewer walk of the assigned levels in the gate log (a judgment claim about a prose table - no suite can decide whether a level was guessed; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- A step that is mechanical today and judgment tomorrow (a helper replaced by a model): the field is wrong until someone updates it, and AC 2's test is what catches the drift.
- The conditional steps (0, 4, 7, 15): they carry the field like any other; a step that may not run still has a judgment level when it does.
- A host that ignores the field entirely: correct, and the default. Advisory means ignorable.
- A future workflow (plan, per TASK-IMP-111) adding steps: out of scope here, and the field's documentation says where it would extend.
- Security-class: one enum field in a markdown table. No execution surface.
