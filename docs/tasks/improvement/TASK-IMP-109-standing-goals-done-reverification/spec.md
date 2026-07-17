---
id: TASK-IMP-109
title: Standing goals - re-verify done, forever
template: task@1
type: improvement
module: improvement
status: testing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-086, TASK-IMP-100, TASK-IMP-102]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 8
service: modules/cuo
new_files:
  - tools/install/docs-tools/verify-goals.mjs
  - tools/install/tests/test_verify_goals.sh
modified_files:
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - modules/skill/contracts/task/STATUS-REFERENCE.md
source_pages:
  - "IMPROVEMENT_HANDOFF.md §11 IMP-29 (the strongest idea in the four articles)"
  - "How to Build An Agentic OS using Fable 5 (Avid, 2026-07-06) BUILD 5: a goal you only verify once is an assumption with a timestamp; the sentinel detects, the pipeline fixes"
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md (done is terminal; nothing re-checks)"
  - "IMPROVEMENT_HANDOFF.md §1 TASK-IMP-086 row (acceptance evidence recorded for rows not in the committed file - caught by an external reviewer, not by us)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-109: Standing goals - re-verify done, forever

## Summary

`done` is terminal and never re-checked. TRACE-004 proves every clause had a passing test on the day it shipped and nothing looks again, so a task shipped in batch 1 could be broken today and the corpus would still show it green. At the `done` flip, enrol the task's cited tests as a standing goal, and re-run the goals on demand. Detection only: a violated goal becomes a `type: bug` task through the normal loop, never an auto-fix.

## Problem

A goal verified once is an assumption with a timestamp. Every acceptance this run recorded is exactly that: true on 2026-07-17, unexamined on 2026-07-18.

`task-reconcile` (v2.7.0) does not close this. It measures drift when a task RE-ENTERS the workflow - it is a turnstile, not a sentinel. A `done` task that never comes back is never looked at again by anything.

The corpus already demonstrates the failure mode: TASK-IMP-086 recorded acceptance evidence for rows that were not in the committed file, and nothing re-checked until an external reviewer asked. That was caught by luck and a bot. The 35 done improvement tasks and 176 done tasks overall currently rest on the same footing.

## Proposed Solution

At the `done` flip, ship-tasks writes `docs/goals/<task-id>.md` carrying the task's §1 cited tests as its predicate - free, because TRACE-004 already collected exactly that list. A `verify-goals.mjs` runner re-runs each predicate, flips a failing goal to `violated`, and appends to a goal ledger. A violated goal produces a finding, and the fix goes through create-tasks -> ship-tasks like any other bug.

Only mechanically re-runnable predicates are enrolled. ACs carrying a justified `verify:` rather than `test:` cannot be re-run by a script, and the goal file MUST say so rather than pretend - an honest gap beats a fake predicate.

## Alternatives Considered

- Re-run the whole suite instead of per-task goals. Rejected: it answers "is the repo green", not "is TASK-X still true", and it cannot name which acceptance decayed.
- Auto-fix on violation. Rejected outright: the sentinel detects, the pipeline fixes. An auto-fix on a violated acceptance is the machine grading its own homework at the one moment nobody is watching.
- Re-open the original task on violation. Rejected: `done` is terminal for a reason, and reviving a shipped task destroys the record of what was accepted. A new `type: bug` task preserves both.
- Cron the runner. Rejected in scope: scheduling is a host decision and CyberOS is invoked, not daemonised. The runner is a command; when it runs is the operator's business.

## Success Metrics

- Primary: a task flipped to `done` gains a goal file whose predicate is its §1 cited tests, and breaking one of those tests flips the goal to `violated` on the next run - suite-asserted. Baseline: `done` is never re-checked.
- Guardrail: a violated goal changes no task status and writes no code - it emits a finding and nothing else.

## Scope

In scope: goal emission at the `done` flip, `verify-goals.mjs`, the goal ledger, the report, suite arms.

### Out of scope / Non-Goals

- Any auto-fix, auto-revert, or status change on violation.
- Scheduling (cron, CI wiring) - the runner is a command.
- Enrolling `verify:`-only ACs as predicates (they are not mechanically re-runnable; the goal names the gap).
- Retiring goals automatically - retirement is a human decision, logged.

## Dependencies

None mechanically. Consumes the §1 cited tests TRACE-004 already collects.

**Serialisation note:** touches `STATUS-REFERENCE.md` and `ship-tasks.md`, both shared with TASK-IMP-108 (and ship-tasks with 113, 114, 115). Parent-serialised per §11a; never concurrent.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §11 IMP-29, itself derived from the agentic-OS builder's guide (Avid, 2026-07-06) BUILD 5; verified against ship-tasks.md and task-reconcile.mjs on merged main.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 On the `done` flip, ship-tasks MUST write `docs/goals/<task-id>.md` carrying: the predicate (the task's §1 cited tests), `born`, `source` (the task id), `status: satisfied`, `last_pass`, and `on_violation` (default: report, never auto-fix).
- 1.2 The predicate MUST be derived from the §1 cited tests TRACE-004 already verified - this task MUST NOT invent a predicate the task never claimed.
- 1.3 ACs whose evidence is a justified `verify:` rather than a `test:` MUST NOT be enrolled as predicates, and the goal file MUST name them as not mechanically re-verifiable.
- 1.4 A task reaching `done` with zero mechanically re-runnable predicates MUST still get a goal file, marked `predicate: none` with the reason - the absence is the finding.
- 1.5 `verify-goals.mjs` MUST re-run each non-retired goal's predicate, flip failures to `status: violated`, refresh `last_pass` on success, and append one row per goal to `docs/goals/.ledger.tsv`.
- 1.6 `verify-goals.mjs` MUST exit non-zero when any goal is violated, and MUST print each violated goal with its task id and the failing predicate.
- 1.7 A violated goal MUST NOT change any task's status, MUST NOT modify code, and MUST NOT re-open the source task. The remedy is a new `type: bug` task through create-tasks.
- 1.8 A predicate exceeding its timeout MUST be treated as violated and named as a timeout - an unrunnable predicate is not a passing one.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - a `done` flip writes a goal file whose predicate is the task's §1 cited tests - test: `tools/install/tests/test_verify_goals.sh::t01_done_emits_goal`
- [ ] AC 2 (traces_to: #1.5, #1.6) - breaking a cited test flips the goal to violated, appends a ledger row, and exits non-zero naming the task - test: `tools/install/tests/test_verify_goals.sh::t02_broken_test_violates`
- [ ] AC 3 (traces_to: #1.5) - a passing goal refreshes `last_pass` and stays satisfied - test: `tools/install/tests/test_verify_goals.sh::t03_passing_refreshes`
- [ ] AC 4 (traces_to: #1.3, #1.4) - a `verify:`-only AC is not enrolled and is named; a task with no runnable predicate still gets a `predicate: none` goal - test: `tools/install/tests/test_verify_goals.sh::t04_unrunnable_named_not_faked`
- [ ] AC 5 (traces_to: #1.7) - a violated goal changes no task status and writes no code - test: `tools/install/tests/test_verify_goals.sh::t05_detection_only`
- [ ] AC 6 (traces_to: #1.8) - a predicate that hangs is violated and named as a timeout - test: `tools/install/tests/test_verify_goals.sh::t06_timeout_is_violation`

## 3. Edge cases

- A cited test that is later legitimately renamed: the goal breaks and is violated. Correct - the acceptance now cites a test that does not exist, and that IS a finding. The remedy is an operator retiring or amending the goal, logged.
- A flaky cited test: it violates intermittently and poisons the ledger. Quarantine (`status: retired`, reason recorded), never silent deletion - a goal deleted without a reason is the 086 pattern.
- 176 existing `done` tasks have no goal files: they are not backfilled by this task (out of scope) and the runner MUST NOT claim coverage it does not have. The report states how many `done` tasks have no goal.
- A goal whose predicate needs a service, credential, or network: it fails in a clean checkout. 1.3's rule applies - if it is not cheap, deterministic, and read-only, it is not a predicate.
- Predicate confinement: goals are files in the repo, and their predicates are commands. They MUST be confined and tracked-checked exactly as TASK-IMP-100's rung-5 requires - a goal file is INPUT, and a crafted predicate must never execute. This is the same defect the batch-5 review caught in `task-reconcile`, and it MUST NOT be re-introduced here.
- Security-class: HIGH by construction - this task executes commands read from files. Every predicate MUST resolve under the repo root, MUST be `git ls-tree HEAD`-tracked, and MUST run without a shell where possible. This is the task's central risk and its central test.
