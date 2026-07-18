---
id: TASK-IMP-107
title: End-to-end mechanical smoke test
template: task@1
type: improvement
module: improvement
status: ready_to_test
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-098, TASK-IMP-100]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 4
service: tools/install
new_files:
  - tools/install/tests/test_e2e_skeleton.sh
modified_files:
  - scripts/tests/run_all.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §10 IMP-28"
  - "25 suites on main bb231900, none spanning install -> lint -> insert -> flips -> reconcile -> uninstall"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-107: End-to-end mechanical smoke test

## Summary

Twenty-five suites cover install hygiene, channels, payload shape, the helper CLIs, the renderer, and workflow doctrine. None runs the three workflows end to end, because the middle two need a model. Add a suite that exercises the MECHANICAL spine - install, lint, insert, every lifecycle flip, coverage-scope, reconcile, uninstall - on a scratch repo with no model in the loop.

## Problem

The proof that the loop works is a session transcript. That is evidence, but it is not a gate: nothing re-runs it, and nothing fails when a helper's contract drifts from the workflow that calls it. Each helper is tested in isolation and the workflow doctrine is tested as prose; the seam between them is tested by hand, once, by whoever happened to run the loop that day.

The batch-4 seed-shape break is the shape of the gap: it was caught by the awh gate, one layer late and only because a goldenset existed. A mechanical spine test would have caught it earlier and without one.

## Proposed Solution

`test_e2e_skeleton.sh`: create a scratch repo, install the built payload, write a fixture `task@1` spec by hand, lint it clean, insert its row via `backlog-mutate`, flip it through every lifecycle status with the real helper, run `coverage-scope` against a stub report, run `task-reconcile` and assert the expected recommendation, uninstall, and assert the corpus survives. No model, no LLM, no network; target under 30 s to stay inside the sandbox cap. It tests the plumbing, not the judgment - which is 100 % more than zero.

## Alternatives Considered

- A model-in-the-loop e2e test. Rejected: non-deterministic, expensive, and it would fail for reasons unrelated to the plumbing - the definition of a flaky gate.
- Extending each helper's own suite. Rejected: they already pass individually; the untested thing is specifically the seam between them.
- Recording a transcript as a golden. Rejected: a transcript of a model's output is not a contract, and pinning it would make every prompt improvement a test failure.

## Success Metrics

- Primary: the spine suite passes on a scratch repo in under 30 s with no network and no model - suite-asserted every run. Baseline: no end-to-end coverage exists.
- Guardrail: it asserts the corpus survives uninstall, closing the loop on the one thing an operator cannot recover from.

## Scope

In scope: the new suite and its wiring into `run_all.sh`.

### Out of scope / Non-Goals

- Testing model judgment, prompt quality, or audit verdict correctness.
- Replacing any existing suite.
- Testing the plugin host or any agent harness.

## Dependencies

None - every helper it drives exists on main.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md IMP-28; implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The suite MUST run on a scratch repo it creates and removes, and MUST NOT touch the working repo's corpus.
- 1.2 It MUST exercise, in order: install the built payload, write a fixture `task@1` spec, `task-lint` clean, `backlog-mutate` insert, a flip through every lifecycle status in `STATUS-REFERENCE.md` §1.1, `coverage-scope` against a stub report, `task-reconcile`, uninstall.
- 1.3 It MUST assert `task-reconcile`'s recommendation matches the state it constructed, not merely that it exits 0.
- 1.4 It MUST assert the corpus survives uninstall.
- 1.5 It MUST require no model, no network, and no credentials, and MUST complete within 30 s on the reference sandbox.
- 1.6 It MUST be wired into `scripts/tests/run_all.sh`.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.2, #1.5) - the full spine runs green on a scratch repo, offline, under 30 s - test: `tools/install/tests/test_e2e_skeleton.sh::t01_spine_green`
- [ ] AC 2 (traces_to: #1.3) - a deliberately drifted state yields the expected reconcile recommendation, not just exit 0 - test: `tools/install/tests/test_e2e_skeleton.sh::t02_reconcile_recommendation_asserted`
- [ ] AC 3 (traces_to: #1.4) - the fixture corpus is present and byte-identical after uninstall - test: `tools/install/tests/test_e2e_skeleton.sh::t03_corpus_survives_uninstall`
- [ ] AC 4 (traces_to: #1.1) - the working repo's `docs/tasks/` is untouched by a suite run - test: `tools/install/tests/test_e2e_skeleton.sh::t04_scratch_isolation`
- [ ] AC 5 (traces_to: #1.6) - `run_all.sh` invokes the suite - test: `scripts/tests/run_all.sh` (the suite appears in its output; asserted by the runner's own count)

## 3. Edge cases

- Scratch repo on a filesystem without `git`: the suite `git init`s its own scratch; if git is absent it MUST skip with a named reason rather than fail (the same skip discipline the existing suites use).
- The 45 s sandbox cap: 1.5's 30 s target leaves headroom; if the spine grows past it, split by phase rather than raise the cap - a suite that cannot finish is a suite that gets disabled.
- A helper that legitimately changes its output shape: this suite WILL fail, and that is the point - it is the seam test, and the fix is to update the assertion deliberately, not to loosen it.
- Leftover scratch dirs from an interrupted run: the suite MUST clean up on trap, and MUST tolerate a stale scratch from a previous kill.
- Security-class: writes only inside its own scratch directory; runs only repo-tracked helpers with fixed arguments; no untrusted input reaches a command.
