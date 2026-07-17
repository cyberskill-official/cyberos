---
id: TASK-IMP-111
title: The plan workflow
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-100, TASK-IMP-093]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 12
service: modules/cuo
new_files:
  - modules/skill/plan-author/SKILL.md
  - modules/skill/plan-audit/SKILL.md
  - modules/skill/rubrics/plan_rubric.md
  - tools/install/plugin/commands/plan.md
  - tools/install/tests/test_plan_workflow.sh
modified_files:
  - modules/skill/repo-context-map-author/SKILL.md
  - tools/install/build.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §12 (findings + proposed implementation, written after checking create-tasks for reuse as instructed)"
  - "modules/skill/task-author/STANDALONE_INTERVIEW.md (required field source_files: 'Which source file(s) should I read' - file plumbing, not elicitation)"
  - "tools/install/plugin/commands/create-tasks.md (promises the interview elicits scope from an idea - it cannot)"
  - "modules/skill/repo-context-map-author/SKILL.md + ship-tasks.md step 1 (inputs {repo_root, task_id}; runs after tasks exist)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-111: The plan workflow

## Summary

`create-tasks` cannot take an idea. Its command promises a standalone interview for the no-document case, but the interview's required field is `source_files` - hand it an idea and it asks for a document. Add `plan`: a workflow that turns an idea (greenfield) or an idea plus a deep repo scan (brownfield) into a `plan@1` document whose proposed task set is exactly what create-tasks already consumes.

## Problem

Two verified gaps.

**The command promises what the skill cannot do.** `create-tasks.md` says: "If given only an idea (no document), use the skill's standalone interview to elicit scope." `task-author/STANDALONE_INTERVIEW.md`'s required field is `source_files`: "Which source file(s) should I read to draft the task? ... The skill confirms each path resolves." It is file plumbing, not elicitation. The brand-new-project case is unreachable today. This is the fourth instance this run of a spec promise with nothing under it.

**The deep scan runs too late and too narrow.** `repo-context-map-author` extracts patterns, schemas, blast radius, and a module-misplacement flag - but its inputs are `{repo_root, task_id}` and it runs at ship-tasks step 1, AFTER tasks exist, scoped to one task. Planning against an existing repo needs a repo-wide scan BEFORE any task exists.

## Proposed Solution

`plan` emits `plan@1` at `docs/plans/PLAN-<slug>-<date>/plan.md`: intent, context, options with checkable evidence, one decision with confidence, scope, a proposed task set, risks, and the BRAIN rows emitted. Section 6 (the proposed task set) is the input contract - a document with a proposed task set is exactly the "PRD or spec" task-author already consumes, so create-tasks needs no new input shape.

Roughly 70 % already exists. `repo-context-map-author` gains `scope: repo | task` (task = today's behavior, byte-for-byte). The PLAN gate's state machine, the spike's option grammar, `memory-append`, and the author/audit pair shape are all reused rather than reinvented.

Mode detection: no `.cyberos/` and no git HEAD is greenfield; commits or a corpus is brownfield with a MANDATORY scan; ambiguous ASKS, because guessing greenfield on a live repo plans against a codebase that exists.

## Alternatives Considered

- Fix `STANDALONE_INTERVIEW.md` to elicit ideas, no new workflow. Rejected: it would put option-weighing, repo scanning, and decision-recording inside task-author, which authors tasks. The interview is the symptom; the missing front door is the defect.
- Build IMP-21 (four-label intake triage) separately. Rejected explicitly: `plan` subsumes it. Triage's `park` and `needs_info` become plan outcomes and `needs_spike` is already a step. Two front doors is one too many.
- Let plan write tasks directly. Rejected: create-tasks owns the audited write path, and a second writer to `docs/tasks/**` re-opens the 086 class.

## Success Metrics

- Primary: an idea with no document produces a `plan@1` that create-tasks consumes unmodified - the case that is unreachable today. Baseline: the interview asks for a file.
- Guardrail: brownfield never plans without a scan, and `repo-context-map-author --scope task` behaves byte-identically to today (the ship-tasks path must not move).

## Scope

In scope: `plan-author`, `plan-audit`, `plan_rubric@1.0`, the `scope` input on repo-context-map, the `/cyberos:plan` command, suite arms.

### Out of scope / Non-Goals

- Writing `docs/tasks/**` or any BACKLOG row - create-tasks does that.
- Writing code. The blast radius is one directory of markdown.
- Setting any task status - plan produces no tasks.
- Changing `repo-context-map-author`'s existing task-scoped behavior.
- A second HITL gate: create-tasks already has its own PLAN gate, and two approvals of the same content in five minutes is how a gate becomes a rubber stamp.

## Dependencies

None blocking. Reuses skills that exist.

**Serialisation note:** touches `build.sh` (shared with TASK-IMP-110 - both add a vendored skill to VENDORED_SKILLS, and chain-coverage fails closed if either lands half-applied). Parent-serialised per §11a.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §12, written after checking create-tasks for reusable mechanism as the operator instructed; the two gaps were verified against the live skills on merged main.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `plan` MUST detect mode: greenfield (no `.cyberos/` AND no git HEAD), brownfield (commits and/or `docs/tasks/`), or ambiguous - and ambiguous MUST halt and ask rather than guess.
- 1.2 In brownfield mode the repo-wide scan MUST run before the interview, and `plan` MUST NOT emit a decision without it.
- 1.3 `repo-context-map-author` MUST accept `scope: repo | task`; `task` MUST behave exactly as today (the ship-tasks path is unchanged).
- 1.4 `plan-author` MUST emit `plan@1` carrying: intent, context, >=2 options each with checkable evidence, exactly one decision with a confidence grade, scope with a non-empty out list, a proposed task set, risks, and the BRAIN rows emitted.
- 1.5 `plan` MUST HALT at one operator gate on the decision, before emitting.
- 1.6 `plan-audit` MUST refuse below 10/10 against `plan_rubric@1.0`.
- 1.7 `plan` MUST NOT write `docs/tasks/**`, MUST NOT write BACKLOG rows, MUST NOT write code, and MUST NOT set any task status.
- 1.8 The `plan@1` proposed task set MUST be consumable by create-tasks with no change to task-author's input contract.
- 1.9 `plan` MUST emit its decision and context to BRAIN via `memory-append`.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - greenfield, brownfield, and ambiguous fixtures each route correctly; ambiguous halts - test: `tools/install/tests/test_plan_workflow.sh::t01_mode_detect`
- [ ] AC 2 (traces_to: #1.2, #1.3) - brownfield runs the repo-scoped scan before the interview; `--scope task` output is byte-identical to today's - test: `tools/install/tests/test_plan_workflow.sh::t02_scan_first_task_scope_unchanged`
- [ ] AC 3 (traces_to: #1.4, #1.6) - a plan missing an option, a decision, or the out list reds at audit - test: `tools/install/tests/test_plan_workflow.sh::t03_rubric_refuses_incomplete`
- [ ] AC 4 (traces_to: #1.7) - a plan run leaves `docs/tasks/**` and BACKLOG.md byte-identical - test: `tools/install/tests/test_plan_workflow.sh::t04_never_writes_tasks`
- [ ] AC 5 (traces_to: #1.8) - a `plan@1` from an idea-only greenfield run is consumed by create-tasks unmodified - test: `tools/install/tests/test_plan_workflow.sh::t05_output_feeds_create_tasks`
- [ ] AC 6 (traces_to: #1.9) - the run appends the decision to BRAIN and the chain verifies - test: `tools/install/tests/test_plan_workflow.sh::t06_brain_rows_chain`
- [ ] AC 7 (traces_to: #1.5) - the decision gate halts and no artefact is emitted without a verdict - verify: recorded gate-log transcript (a HITL halt cannot be asserted by a suite without simulating the human; same rationale as the existing gate arms).

## 3. Edge cases

- Brownfield repo with 100k+ lines: the scan must bound itself (module inventory and conventions, not every file) or it exceeds the sandbox cap. The scan reports what it sampled rather than implying exhaustiveness.
- Greenfield where the operator has an idea but no opinion on stack: options carry the evidence and the decision names a confidence of `low` - honest, and a low-confidence decision is what a spike is for.
- A repo with `.cyberos/` but no commits (installed, never committed): brownfield - the machine's presence means someone intends to work here.
- An idea that is already a task in the corpus: the brownfield scan surfaces it and the plan's option set MUST include "this exists" rather than proposing a duplicate.
- Operator points `plan` at a document (not an idea): legal - it plans from the document. `plan` is not restricted to ideas; it is the front door.
- Security-class: the scan reads repo files and the interview reads operator text. Neither is executed. The scan MUST confine under the repo root; a plan document is a proposal and is never a command source.
