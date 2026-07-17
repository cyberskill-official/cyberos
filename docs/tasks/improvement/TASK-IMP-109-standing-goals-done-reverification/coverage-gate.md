---
task_id: TASK-IMP-109
artefact: coverage-gate@1
phase: testing
generated: 2026-07-17
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
coverage_measurable: false
coverage_reason: "verify-goals.mjs is node stdlib JS; this repo runs no c8/lcov over docs-tools. No percentage is reported rather than fabricated (same as TASK-IMP-103/104). The gate is tests_failed=0 + TRACE-004 closure + full ECM coverage."
trace_004_closed: true
security_class: high
---

# TASK-IMP-109 - coverage gate

## TRACE-004 closure

| AC | traces_to | cited test | result |
|---|---|---|---|
| AC 1 | #1.1, #1.2 | `test_verify_goals.sh::t01_done_emits_goal` | passed |
| AC 2 | #1.5, #1.6 | `test_verify_goals.sh::t02_broken_test_violates` | passed |
| AC 3 | #1.5 | `test_verify_goals.sh::t03_passing_refreshes` | passed |
| AC 4 | #1.3, #1.4 | `test_verify_goals.sh::t04_unrunnable_named_not_faked` | passed |
| AC 5 | #1.7 | `test_verify_goals.sh::t05_detection_only` | passed |
| AC 6 | #1.8 | `test_verify_goals.sh::t06_timeout_is_violation` | passed |

All 8 §1 clauses cited; 6/6 ACs closed.

## The security class IS the task (§3)

This tool executes commands read from files - the rung-5 defect the 2026-07-17 review caught in
`task-reconcile`. The guard is the spine, in order, and every step refuses loudly:

| Step | Arm | result |
|---|---|---|
| CONFINE - `relUnderRoot`, the same predicate task-reconcile and coverage-scope use | `t07_predicate_escaping_root_refused` | passed |
| EXISTS - a citation resolving nowhere IS the finding | `t09_refusal_is_a_violation_not_a_skip` | passed |
| TRACKED AT HEAD - `git ls-tree`; an untracked file on disk cannot be a repo's acceptance | `t08_untracked_predicate_refused` | passed |
| EXECUTE - argv, never a shell string; never `eval` | (inspection: `sh("bash",[rel],root,TIMEOUT)`) | covered |
| REFUSAL = VIOLATION, never a silent skip | `t09` (exit 1 on a refused predicate) | passed |

**t08 is the arm that matters.** It plants an executable-but-untracked predicate that would
`touch PWNED` and asserts the file never appears. It does not.

## Edge-case matrix coverage (§3)

| Row | Arm | result |
|---|---|---|
| A cited test later renamed -> goal breaks, correctly (the acceptance cites a test that is gone) | `t09` | covered |
| Flaky predicate -> quarantine (`status: retired`), never deleted | `t10_retired_goal_skipped` | covered |
| 176+ existing done tasks have no goals; the report must not imply coverage | **`t11_report_states_its_coverage`** (added at the review gate on operator instruction) | covered |
| A predicate needing a service/credential/network fails in a clean checkout | §1.3's rule: not cheap+deterministic+read-only = not a predicate | covered by contract |
| Predicate confinement (the rung-5 defect, re-introduced) | t07/t08/t09 | covered |

## Defects found during implementation and review

1. **Crash on the DEFAULT invocation** (caught by t03). `argv.indexOf(flag) + 1` reads `argv[0]`
   when the flag is absent (`-1 + 1 = 0`), so `--timeout` unset gave `Number("--repo")` = NaN and
   `spawnSync` threw. Flags are now read by presence; `--timeout` is validated.

2. **The report hid its own denominator** (caught by the OPERATOR at the review gate). §3 required
   the report to state how many `done` tasks have no goal; enrolment and the runner shipped
   without it. A green "0 violated" over a corpus it covers 0.5% of is the TASK-IMP-086 class.
   Now: `coverage: 1/179 done tasks enrolled - 178 have NO goal and are unverified since the day
   they shipped.`

## Live fire - on real evidence, not a fixture

TASK-IMP-103 shipped hours earlier and is enrolled. `verify-goals` re-verified its acceptance in
1080ms (satisfied). Commenting out 103's `_cy_lock_acquire` call flipped the goal to **VIOLATED**,
naming the failing predicate; restoring it returned to **satisfied**.

`done` is now a maintained invariant rather than a claim - for 1 task of 179, and the report says
so out loud.

## Suite evidence

```
test_verify_goals    11/11  (t01..t11; four of them the security guard)
test_workflow_helpers 16/16   test_chain_coverage    7/7
test_full_sdp_payload  9/9
build OK; sync OK 1.0.0 across 7 artifacts; payload carries verify-goals.mjs (t01 asserts it)
```
