---
id: TASK-IMP-101
title: ship-tasks reconcile entry phase and depends_on evidence gating
template: task@1
type: improvement
module: improvement
status: implementing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T10:10:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-100]
blocks: []
related_tasks: [TASK-IMP-092, TASK-IMP-099]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 3
service: modules/cuo
new_files: []
modified_files:
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/tests/test_workflow_helpers.sh
source_pages:
  - "operator decisions 2026-07-17 (recorded HITL question): reconcile ships now; depends_on gating = BLOCK with human override"
  - "ship-tasks.md state engine (reads BACKLOG before each iteration; picks at ready_to_implement) and resume semantics § (manifest-verified resume; human gates re-ask) - the two covered states this task joins with the third"
  - "TASK-IMP-100 reconcile-report@1 (the evidence artefact this wiring consumes)"
source_decisions:
  - "2026-07-17 Stephen: batch 5 approved; deps gating yes-with-override."
---

# TASK-IMP-101: ship-tasks reconcile entry phase and depends_on evidence gating

## Summary

TASK-IMP-100 builds the measuring instrument; this task wires it into doctrine. ship-tasks gains a conditional entry phase - encountering a task whose status claims work this workflow cannot vouch for (past ready_to_implement with no manifest, a stale manifest, or missing phase artefacts) triggers task-reconcile and presents the report at a HITL gate with the three-way fork; and a depends_on evidence gate - a task is not started while an upstream done dependency lacks evidence (workflow artefacts or an accepted reconcile verdict), overridable only by the human at the gate. workflow_version 2.6.4 to 2.7.0: a new mechanism, not a wording fix.

## Problem

The state engine's trust model had two legs (own manifests, own gates) and silently extended that trust to any status cell it read. After 086, "the index says done" is known to be a claim, not a fact - the workflow text must say what happens when claims outrun evidence, and must stop new work from stacking on unverified foundations by default.

## Proposed Solution

Two normative additions in ship-tasks.md, plus chain wiring: (a) **Reconcile entry §**: the trigger conditions, the invocation (`node .cyberos/docs-tools/task-reconcile.mjs <id> --run-tests`), the HITL gate presenting the report's single recommendation with the fork {resume_at_phase / route_back / adopt_candidate - backfill artefacts then re-enter}, the rule that the agent NEVER executes any branch without the recorded verdict, and route_back's mechanics (§1.3, routed_back_count += 1). (b) **depends_on evidence gate §**: before starting any task, each depends_on id with status done MUST carry evidence - a coverage-gate artefact in either artefact home, or a reconcile-report with a human-accepted verdict; otherwise the task is BLOCKED and the block is presented at a gate where the human may override (override recorded as memory.status_overridden). (c) skill_chain gains conditional step 0 (skill: task-reconcile, condition: entry state drifted) with reconcile_report in outputs. t14 gates the two passages + 2.7.0 in source and payload; the t12/t09 exact pins move to 2.7.0 (the known pair, disclosed).

## Alternatives Considered

- Warn-only deps gating. Rejected by the recorded decision: propagating onto unverified foundations is the expensive failure; override keeps velocity available deliberately.
- Hard-block without override. Rejected: historical done tasks predate these artefact conventions; a human override with a recorded row is the honest escape hatch.
- Reconcile as a separate operator command only (no chain wiring). Rejected: the state engine is where drifted claims are MET; an unwired tool is advice, not behavior.

## Success Metrics

- Primary: the vendored workflow carries both passages at 2.7.0 with step 0 in the chain, suite-asserted every run. Baseline: trust extended to any status cell; deps are a scheduling hint. Deadline: final acceptance.
- Guardrail: t01-t13 stay green (the additions must not disturb helper or doctrine gates).

## Scope

In scope: the two §§, step 0 chain entry, version bump, pin moves, t14.

### Out of scope / Non-Goals

- The reconcile tool itself (TASK-IMP-100).
- Retroactively reconciling the historical corpus (operator-initiated, per task, when it matters).
- New lifecycle statuses (fork outcomes map onto STATUS-REFERENCE §1 as-is).

## Dependencies

- depends_on TASK-IMP-100 (chain step 0 names the skill; the SKILL dir must exist for chain-coverage gates) - same agent, serial order, per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded operator decisions; implementation under ship-tasks supervision.
- **Human review:** batch-5 PLAN approved 2026-07-17 via recorded HITL decision; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 ship-tasks.md MUST gain the reconcile entry §: trigger conditions (status past ready_to_implement AND (no manifest OR manifest verify fails OR required phase artefacts missing)), the tool invocation, the three-way HITL fork, the no-silent-execution rule, and route_back mechanics per STATUS-REFERENCE §1.3.
- 1.2 ship-tasks.md MUST gain the depends_on evidence gate §: done upstreams require a coverage-gate artefact (either artefact home) or a human-accepted reconcile-report; otherwise the task is blocked, and override is a recorded human act (memory.status_overridden).
- 1.3 skill_chain MUST gain conditional step 0 (skill: task-reconcile) with reconcile_report added to outputs.
- 1.4 workflow_version MUST bump to 2.7.0; the t12 and t09_doctrine_wiring exact pins MUST move with it (disclosed).
- 1.5 The suite MUST gate both passages, step 0, and the version in source AND scratch payload (new t14 in test_workflow_helpers.sh), with t01-t13 green.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2, #1.3, #1.5) - both passages + step 0 present in source and payload at 2.7.0 - test: `tools/install/tests/test_workflow_helpers.sh::t14_reconcile_entry_and_deps_gate`
- [ ] AC 2 (traces_to: #1.4) - version pins moved, suite green - test: `tools/install/tests/test_workflow_helpers.sh::t12_doctrine_view_rules_vendored (pin at 2.7.0)`
- [ ] AC 3 (traces_to: #1.5) - t01-t13 undisturbed - test: `tools/install/tests/test_workflow_helpers.sh::t01_manifest_lifecycle (representative; suite runs as one)`

## 3. Edge cases

- Task with a VALID manifest whose frontmatter status looks past the manifest's step: existing resume semantics own it (manifest outranks the cell); reconcile entry only fires when resume's own preconditions fail - stated in the § to prevent double-handling.
- depends_on naming an off-ramped task (closed/duplicate): not "done" - the gate treats it as unmet dependency and surfaces it (existing eligibility semantics; restated).
- Historical done tasks with artefacts in docs/tasks/.workflow/<id>/ (the corpus norm before per-folder artefacts): the coverage-gate check accepts both homes (1.2), so the gate does not false-block the existing corpus.
- Security-class: none - doctrine prose + test pins; the tool it names is read-only by 100's contract.
