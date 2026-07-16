---
id: TASK-IMP-100
title: task-reconcile, evidence ladder and report for drifted task states
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
depends_on: []
blocks: [TASK-IMP-101]
related_tasks: [TASK-IMP-084, TASK-IMP-085, TASK-IMP-086, TASK-IMP-092, TASK-IMP-098]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 6
service: tools/install/docs-tools
new_files:
  - tools/install/docs-tools/task-reconcile.mjs
  - tools/install/tests/test_task_reconcile.sh
  - modules/skill/task-reconcile/SKILL.md
modified_files:
  - tools/install/build.sh
source_pages:
  - "operator question 2026-07-17: when tasks are already implemented (mid-shipping or shipped) but unaudited or of doubtful quality, how should ship-tasks behave - is there a mechanism to measure that state and put rework-vs-continue to HITL?"
  - "the gap map: ship-manifest@1 resume covers the workflow's OWN runs (hash-verified); the gate ladder + STATUS-REFERENCE §1.3 route-back covers in-loop quality; NOTHING covers out-of-band work whose status claims outrun evidence - the TASK-IMP-086 incident class (status done, no commit carried the deliverable)"
  - "existing rungs to compose: task-lint (084), ship-manifest verify (085), artefact-set-per-phase convention (batch exemplars), coverage-scope (098), committed-object rule (092), audit.md sha binding"
source_decisions:
  - "2026-07-17 Stephen: batch 5 approved - spec + implement now; recommendation set {resume_at_phase, route_back, adopt_candidate}; HITL always holds the verdict."
---

# TASK-IMP-100: task-reconcile, evidence ladder and report for drifted task states

## Summary

ship-tasks trusts two things today: its own manifests (hash-verified resume) and its own gates (route-back). A task that arrives already-implemented from outside the loop - status past ready_to_implement with no manifest, or done with missing gate artefacts - is invisible to both, and the 086 incident showed what an unverified status claim costs. Ship `task-reconcile.mjs`: a read-only evidence ladder over one task that emits `reconcile-report@1` with per-rung verdicts and exactly one recommendation from a closed set, never acting on it - the verdict is HITL's, wired into the workflow by TASK-IMP-101.

## Problem

Status claims beyond ready_to_implement carry no proof obligation. A task marked ready_to_review with no diff, or done with no coverage-gate, sits in the index looking finished; the only detector so far has been an external review bot. The measuring pieces all exist - nothing composes them into a verdict a human can act on.

## Proposed Solution

`node .cyberos/docs-tools/task-reconcile.mjs <task-id> [--repo <root>] [--run-tests] [--json] [--out <file>]`. Rungs, all read-only (rung 5 executes suites only under the explicit flag):
1. **Spec integrity** - task-lint verdict; audit.md present with overall_status pass; audit's audited_file_sha256 prefix matches the CURRENT spec bytes (drift = red).
2. **Artefact completeness vs claimed phase** - the required set per status (implementing+: context-map, edge-case-matrix, impl-plan, obs-injection; reviewing+: code-review; testing/done: coverage-gate), accepted in the task folder or docs/tasks/.workflow/<id>/.
3. **Manifest state** - ship-manifest present? verify via ship-manifest.mjs (version, task sha, artefact hashes); absent manifest is a finding, not a failure (out-of-band work has none).
4. **Committed-object presence** - every frontmatter new_files/modified_files path exists at HEAD (git ls-tree), per the 092 rule: claims are measured on commits, not working views.
5. **Cited tests now** (--run-tests) - the suite files named by §2 `test:` entries run once each; exit codes recorded.
Report: `reconcile-report@1` markdown (frontmatter: task, claimed_status, rung verdicts, drift_score, recommendation, hitl: required) + `--json`. Recommendation is EXACTLY one of `resume_at_phase(<N>)` (claims supported), `route_back` (claims unsupported - reasons per rung), `adopt_candidate` (work sound at HEAD, artefacts missing - backfill then re-enter). The tool never mutates task state, BACKLOG, or specs. modules/skill/task-reconcile/SKILL.md wraps it: machine floor first, judgment guidance for the model half (reading rung reds, drafting the gate question), and the hard rule that the agent NEVER executes a recommendation without the recorded human verdict. build.sh vendors the tool.

## Alternatives Considered

- Extend ship-manifest.mjs. Rejected: manifests describe runs this workflow performed; reconcile judges work it did not perform - conflating them re-opens the claim/evidence confusion this task closes.
- A pure prose procedure (doctrine only). Rejected by the operator decision: the ladder is mechanical and fatigue-prone - exactly the machine-floor class (084 precedent).
- Auto-adopt when everything is green. Rejected: adoption is acceptance of unwitnessed work; the two-gate doctrine puts every acceptance in human hands.

## Success Metrics

- Primary: four fixture drift shapes (clean-resume, claims-unsupported, adopt-shaped, spec-drift) each produce their expected recommendation and rung verdicts, suite-asserted every run. Baseline: no mechanism - operator judgment unaided. Deadline: final acceptance.
- Guardrail: the tool provably mutates nothing (fixture tree byte-fingerprint identical after every run; report goes to stdout/--out only).

## Scope

In scope: the CLI and its five rungs, reconcile-report@1 shape, the SKILL.md contract, fixture suite, build.sh vendor line.

### Out of scope / Non-Goals

- Wiring into ship-tasks' state engine and depends_on gating (TASK-IMP-101, same batch).
- Executing recommendations (flips stay with backlog-mutate under human verdicts).
- New lifecycle statuses (the three outcomes map onto STATUS-REFERENCE §1 as-is).

## Dependencies

- Upstream: none (composes shipped tools 084/085/098). Downstream: blocks TASK-IMP-101 (the chain wiring needs the skill dir) - same agent, serial, per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the operator's doctrine clarification and the recorded gap map; implementation under ship-tasks supervision.
- **Human review:** batch-5 PLAN approved 2026-07-17 via recorded HITL decision; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The CLI MUST evaluate rungs 1-4 read-only in one invocation and rung 5 only under `--run-tests`, and MUST NOT write to any file outside `--out`.
- 1.2 The report MUST carry per-rung verdicts (pass/red/absent with reasons), a drift_score, `hitl: required`, and EXACTLY one recommendation from {resume_at_phase(N), route_back, adopt_candidate}.
- 1.3 Recommendation mapping MUST be: all rungs supporting the claimed status -> resume_at_phase(N of the claimed phase); any load-bearing rung red (spec drift, missing deliverable at HEAD, failing cited suite) -> route_back with the reasons; deliverables present and green at HEAD but phase artefacts missing -> adopt_candidate.
- 1.4 Artefact completeness MUST accept both artefact homes (the task folder and docs/tasks/.workflow/<task-id>/), per the corpus convention.
- 1.5 modules/skill/task-reconcile/SKILL.md MUST state the machine-floor-first loop and the hard rule: the agent never executes a recommendation without a recorded human verdict.
- 1.6 build.sh MUST vendor the tool (guarded copy); the suite MUST land at tools/install/tests/test_task_reconcile.sh (run_all glob) and MUST gate the payload copy.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.2, #1.3) - clean-resume fixture -> resume_at_phase with all rungs pass - test: `tools/install/tests/test_task_reconcile.sh::t01_clean_resume`
- [ ] AC 2 (traces_to: #1.3) - claims-unsupported fixture (missing artefacts + failing cited suite) -> route_back naming each red rung - test: `tools/install/tests/test_task_reconcile.sh::t02_route_back`
- [ ] AC 3 (traces_to: #1.3, #1.4) - adopt-shaped fixture (deliverables at HEAD green, artefacts absent in both homes) -> adopt_candidate - test: `tools/install/tests/test_task_reconcile.sh::t03_adopt_candidate`
- [ ] AC 4 (traces_to: #1.1) - mutation guard: fixture tree fingerprint identical after every run; spec-drift fixture (audit sha mismatch) -> route_back - test: `tools/install/tests/test_task_reconcile.sh::t04_read_only_and_spec_drift`
- [ ] AC 5 (traces_to: #1.6) - payload carries the tool; suite glob-discovered - test: `tools/install/tests/test_task_reconcile.sh::t05_payload_vendored`
- [ ] AC 6 (traces_to: #1.5) - SKILL.md carries the machine-floor loop and the no-silent-execution rule - verify: recorded greps in the gate log (prose contract, same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- Task at ready_to_implement or draft (no drift possible): the tool reports `not_applicable` and exits 0 without a recommendation - reconcile is for claims past the entry state.
- Manifest present AND valid while artefacts are complete: resume path even if the frontmatter status looks odd - the manifest is stronger evidence than the cell (rung 3 outranks the claim; stated in the report).
- Cited test suite absent from disk (renamed since): rung 5 red naming the path, recommendation route_back - a citation that resolves nowhere is TRACE-003 drift at run time.
- done task with every rung green including coverage-gate: resume_at_phase maps to "confirm done" - the report says so explicitly and still demands the human verdict.
- Security-class: read-only by contract (AC 4's fingerprint); --run-tests executes only repo-tracked suite files named by the spec, never constructed commands.
