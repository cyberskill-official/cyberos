---
id: TASK-IMP-108
title: Split the statuses doing two jobs
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-092, TASK-IMP-099, TASK-IMP-101]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
shipped: 2026-07-17
effort_hours: 6
service: modules/skill
new_files:
  - (none)
modified_files:
  - modules/skill/contracts/task/STATUS-REFERENCE.md
  - tools/install/templates/TASK-TEMPLATE.md
  - tools/install/docs-tools/task-lint.mjs
  - tools/install/docs-tools/backlog-mutate.mjs
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/docs-site/render-status-hub.mjs
  - modules/cuo/tests/test_workflow_evolution.py
  - tools/install/tests/test_task_lint.sh
  - tools/docs-site/tests/test_render_status_hub.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §13.2 (draft_reason + staleness + spec_rejected), §14.4 (entered_via - the finding that survived the rejected rename), §10 IMP-25 (route-back ceiling) - the handoff asks for these as ONE batch"
  - "modules/skill/contracts/task/STATUS-REFERENCE.md §1.1 (ready_to_implement carries both meanings), §1.3 (every failure routes there)"
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md (18 routed_back_count references, 0 read it as a limit; the 5-fail breaker at line 210 bounds only the debugging cycle)"
  - "docs/tasks/BACKLOG.md on main bb231900: 336 draft rows vs 176 done"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-108: Split the statuses doing two jobs

## Summary

Three findings are one defect: `draft` means four different things, `ready_to_implement` means two, and `routed_back_count` is written eighteen times and read as a limit zero times. Each is a word doing more than one job, and the fix in every case is to record the reason rather than mint a status. Add `draft_reason`, add `entered_via`, and give the route-back a ceiling that halts for the operator instead of grinding.

## Problem

Verified on merged main:

- **`draft` (336 rows, the largest population in the corpus).** A spec being written this minute, a 2026-07-08 migration stub whose clauses were never authored, a re-opened `cannot_reproduce` bug, and a deliberately parked idea all wear one word. Nothing distinguishes them, nothing ages them, and the status page reports a percentage against a denominator nobody believes.
- **`ready_to_implement`.** Per §1.1 it is "audited, never built"; per §1.3 it is also the route-back target with `routed_back_count += 1` - a task with code, tests, a review packet, and a history of failure. The queue treats both identically and a reader cannot tell them apart without opening the frontmatter.
- **`routed_back_count`.** 18 references in ship-tasks, every one an increment or a definition. No rule reads it as a limit. The 5-fail circuit breaker bounds the debugging cycle inside one testing phase; nothing bounds the number of times a task circles the whole loop.

The consequence is shared: work that keeps failing cannot be told from work that has not started, and nothing escalates.

## Proposed Solution

Record reasons, do not mint statuses. `draft_reason` (enum: `authoring`, `migrated_stub`, `needs_spec`, `parked_idea`) says which kind of draft this is. `entered_via` (enum: `audit`, `rework`, `spec_rejected`) says which kind of `ready_to_implement` this is - and `spec_rejected` gives a wrong SPEC somewhere to go other than back to an implementer who will build the same wrong thing. At `routed_back_count >= 3` ship-tasks HALTS at an operator gate presenting the route-back reasons side by side, with the verdicts re-enter / split / `on_hold` / `closed`.

Three is a judgment call and the spec says so: it is the point at which "the same task failed three different ways" is evidence about the spec, not the implementation.

## Alternatives Considered

- Remove `draft` (the original proposal). Rejected: it is the only status where a spec may be wrong. Every other status carries a warranty; delete it and `draft -> ready_to_implement` collapses into one atomic act, which deletes the audit gate - the transition IS the gate. 336 rows would have to masquerade as something they are not.
- Mint `draft_migrated`, `draft_parked`, `ready_to_rework`. Rejected: the enum's own doctrine (§1.2) is that `closed` meaning six things is a backlog you cannot learn from - the answer there was a field (`duplicate_of`), not more statuses. Same answer here.
- Auto-close stale drafts. Rejected: a staleness report is information; auto-closing is the machine making a scope decision that belongs to the operator.
- Read `routed_back_count > 0` instead of adding `entered_via`. Retained as a fallback if the field proves redundant in review - the counter already carries the signal, and `entered_via` is only worth it for `spec_rejected`, which the counter cannot express.

## Success Metrics

- Primary: the queue can tell fresh work from thrash, and a third route-back halts for a human instead of re-entering - suite-asserted. Baseline: both are invisible today.
- Guardrail: the status enum is unchanged (12 values), no task file changes status as a result of this task, and the existing route-back path still works for counts under the ceiling.

## Scope

In scope: `draft_reason` and `entered_via` frontmatter fields + their FM rules, the route-back ceiling in ship-tasks, the staleness report on the status page, suite arms.

### Out of scope / Non-Goals

- Any change to the status enum itself - the 12 values stay (TASK-IMP-DECIDED 2026-07-17: `implement` keeps its name).
- Backfilling `draft_reason` across the 336 existing drafts - the field is optional and absent means unknown, which is honest. Backfill is an operator decision, possibly per module.
- Auto-closing or auto-escalating anything.
- Changing what the 5-fail debugging circuit breaker does.

## Dependencies

None logically. Fields are additive and optional.

**Serialisation note:** touches `STATUS-REFERENCE.md` (shared with TASK-IMP-109), `ship-tasks.md` (109, 113, 114, 115), `backlog-mutate.mjs` (105), `render-status-hub.mjs` (114), `test_workflow_evolution.py` (115). Different sections in each, but §11a forbids concurrent shared-tree writes: the parent serialises these, and they MUST NOT be swarm members of one round.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §13.2, §14.4, and §10 IMP-25 - three findings the handoff explicitly asks to be treated as one batch; verified against STATUS-REFERENCE.md and ship-tasks.md on merged main.
- **Human review:** the keep-`draft` and keep-`implement` decisions are the operator's recorded 2026-07-17 answers; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `task@1` MUST accept optional `draft_reason` with the closed enum `authoring | migrated_stub | needs_spec | parked_idea`. Absent MUST be legal and MUST mean unknown.
- 1.2 `task-lint` MUST reject a `draft_reason` value outside the enum, and MUST NOT require the field.
- 1.3 `task@1` MUST accept optional `entered_via` with the closed enum `audit | rework | spec_rejected`, carrying the same absent-is-legal rule.
- 1.4 `backlog-state-update-author` MUST set `entered_via: rework` on any §1.3 route-back it writes, alongside the existing `routed_back_count += 1`.
- 1.5 A task routed back because its SPEC is wrong (not its code) MUST be flipped to `draft` with `entered_via: spec_rejected` and re-audited before re-entering - it MUST NOT go to `ready_to_implement`, which would hand an unchanged wrong spec to an implementer.
- 1.6 ship-tasks MUST HALT at an operator gate when a task with `routed_back_count >= 3` would re-enter, presenting every route-back reason on record and the verdicts re-enter / split / `on_hold` / `closed`. It MUST NOT re-enter without a recorded verdict.
- 1.7 The status page MUST render a staleness report: drafts grouped by `draft_reason` with age. It MUST NOT change any task's status.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - a spec with each enum value lints clean; an out-of-enum value reds; an absent field lints clean - test: `tools/install/tests/test_task_lint.sh::t09_draft_reason_enum`
- [ ] AC 2 (traces_to: #1.3, #1.4) - a route-back writes `entered_via: rework` and increments the counter in one mutation - test: `tools/install/tests/test_workflow_helpers.sh::t18_entered_via_on_rework`
- [ ] AC 3 (traces_to: #1.5) - a `spec_rejected` route-back lands at `draft`, not `ready_to_implement` - test: `tools/install/tests/test_workflow_helpers.sh::t19_spec_rejected_lands_draft`
- [ ] AC 4 (traces_to: #1.6) - a task at `routed_back_count: 3` halts with reasons presented and does not re-enter without a verdict - test: `modules/cuo/tests/test_workflow_evolution.py::test_routeback_ceiling_halts`
- [ ] AC 5 (traces_to: #1.6) - a task at `routed_back_count: 2` re-enters normally (the ceiling is 3, not "any") - test: `modules/cuo/tests/test_workflow_evolution.py::test_under_ceiling_reenters`
- [ ] AC 6 (traces_to: #1.7) - the staleness report renders drafts by reason and age, and no status changes - test: `tools/docs-site/tests/test_render_status_hub.sh::t11_draft_staleness_report`

## 3. Edge cases

- The 336 existing drafts carry no `draft_reason`: they render as unknown, which is the truth. This task MUST NOT invent a reason for a task it did not author (that would be the `# UNREVIEWED` mistake again, with better manners).
- `routed_back_count` at 3 with all three route-backs from the same cause (one flaky test): the gate still halts. A human reading three identical reasons decides in seconds, and the alternative is a loop that never asks.
- A task manually flipped by an operator to `ready_to_implement` with no `entered_via`: legal (absent = unknown, 1.3), and the ceiling still applies from the counter.
- `spec_rejected` on a task whose spec is fine but whose ACs were mis-cited: still `draft` - re-authoring the AC block is a spec change, and the audit is what catches it.
- Ceiling reached during a swarm batch: the halt belongs to the parent, per §11a - a sub-agent MUST NOT resolve it, because the verdict is the operator's.
- A draft that is `authoring` and 200 days old: reported, never touched. The report is the finding; the operator is the fix.
- Security-class: two enum fields and a counter comparison. No untrusted content is executed; the enums are closed, so a crafted value reds rather than routes.
