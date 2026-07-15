---
# ── Identity ─────────────────────────────────────────────────────────
name: debugging-cycle-author
description: >-
  Run a multi-vector debugging pass when the coverage-gate trips (tests_failed > 0 OR files_below_90pct non-empty OR ecm_rows_uncovered non-empty). For each failed iteration: (1) classify the failure vector — state / network / memory / logic / flake; (2) state the targeted hypothesis + exact file:line change; (3) re-run the test suite; (4) emit one `debug-trace@1` attempt-row per cycle. Trips the workflow circuit breaker after 5 consecutive failures. Used by chief-technology-officer/ship-tasks as step 15, conditional on `coverage_report.tests_failed > 0`. Use when user asks to "draft a debugging cycle" or "create the debugging cycle". Do NOT use for "audit existing debugging cycle" (use debugging-cycle-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: debug-trace@1
  cyberos-rubric-target: debugging_cycle_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/debug-trace
audit:
  row_kind: debug_cycle_authored
  required_fields: [task_id, attempts, failure_vectors_seen, consecutive_failures, circuit_breaker_tripped]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,               format: task@1, required: true }
  - { name: coverage_report,  format: coverage-gate@1,   required: true }
outputs:
  - { name: debug_trace, format: debug-trace@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - workflow `chief-technology-officer/ship-tasks` step 15 when coverage_report.tests_failed > 0
blockers:
  - "test framework itself is broken (no test process can start) — diagnose tooling before this skill runs"
  - "circuit breaker already tripped on this task this session — escalate, do not retry"
---

# debugging-cycle-author

## 1. Purpose

Replace the unbounded "try things until it works" loop with a bounded,
auditable five-attempt budget. Each attempt is classified, hypothesised,
and tested; the outer workflow uses the attempt count to trip the
circuit breaker.

## 2. Output schema

```yaml
# debug-trace@1
task_id: task-<MODULE>-<NNN>
generated_at: <ISO-8601>
trigger: "tests_failed > 0 | files_below_90pct | ecm_rows_uncovered"
budget_max_attempts: 5

attempts:
  - id: ATTEMPT-001
    started_at: <ISO-8601>
    failure_vector: state | network | memory | logic | flake | env
    hypothesis: "<one-paragraph hypothesis tying observed failure to a root cause>"
    change:
      file: "<absolute>"
      lines: "<L1-L2 | new>"
      diff_summary: "<one-liner>"
    rerun_result:
      tests_failed: <int>
      files_below_90pct: <int>
      ecm_rows_uncovered: <int>
    outcome: passed | partial | regressed | no-progress

consecutive_failures: <int>
circuit_breaker_tripped: false | true
resolution: passed | tripped-circuit-breaker
on_trip_actions:
  - "git restore on touched paths"
  - "mark task [FAILED: UNRESOLVABLE ERROR] in BACKLOG via backlog-state-update"
  - "emit workflow_failed memory audit row with this debug-trace as the artefact"
```

## 3. Quality gates

- Attempts are bounded by `budget_max_attempts` (default 5).
- Each attempt has a non-vacuous `hypothesis` (≥ one sentence specifying
  a root cause, not "try X again").
- `change.file:lines` resolves to a real diff (audit checks via `git diff`).
- `consecutive_failures` increments on `regressed | no-progress`, resets
  on `partial`.
- `circuit_breaker_tripped` is true iff `consecutive_failures ≥ 5`.

## 4. Chains to

`debugging-cycle-audit` then `backlog-state-update-author`.

---

*End of debugging-cycle-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
