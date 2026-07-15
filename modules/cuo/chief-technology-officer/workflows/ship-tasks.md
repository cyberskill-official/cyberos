---
workflow_id: chief-technology-officer/ship-tasks
workflow_version: 2.6.0
purpose: Drive each eligible task in `docs/tasks/BACKLOG.md` end-to-end through the full lifecycle — from `ready_to_implement` through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1.1). Deep-maps the repo, generates the edge-case matrix, implements with 90 % coverage on touched files, injects observability, self-approves architectural deviations via ADRs, runs the multi-vector debugger with a 5-fail circuit breaker, runs the testing gate (`coverage-gate-author`/`-audit`), and physically updates BACKLOG.md status between every phase transition. Failure or blocker at any downstream phase routes the task back to `ready_to_implement` (STATUS-REFERENCE §1.3) with `routed_back_count += 1`.
persona: chief-technology-officer
cadence: per-task (loops continuously over BACKLOG.md)
status: shipped   # CUO-workflow lifecycle: planned | shipped | retired (distinct from task lifecycle in STATUS-REFERENCE.md)
pattern: linear
hitl: required    # human-acceptance verdict mandatory at reviewing->ready_to_test and testing->done (STATUS-REFERENCE §1.4, EXECUTION-DISCIPLINE §2a); the agent never self-sets done
scope: all implementation work - net-new product tasks and improvement/hardening tasks alike; there is no separate improvement track (see section 1a)

inputs:
  - { name: backlog,                source: docs/tasks/BACKLOG.md,                                       format: markdown }
  - { name: repo_root,              source: workflow-caller,                                                        format: absolute path }
  - { name: stop_signal,            source: operator (Ctrl-C / workflow-stop event),                                format: bool }

outputs:
  - { name: updated_backlog,           format: markdown (BACKLOG.md with status mutations),         recipient: repo HEAD }
  - { name: implementation_diff,       format: git diff (files added/modified),                    recipient: human-reviewer (commit + push manual) }
  - { name: adr_records,               format: architecture-decision-record@1 (zero or more),      recipient: docs/adrs/ }
  - { name: edge_case_matrix,          format: edge-case-matrix@1 (one per task),                    recipient: memory audit chain }
  - { name: coverage_report,           format: coverage-gate@1 (one per task),                       recipient: memory audit chain }
  - { name: debug_trace,               format: debug-trace@1 (one per failed task attempt),          recipient: memory audit chain }
  - { name: task_audit_report,           format: task-audit@2.0 (pre-flight, one per task), recipient: memory audit chain + <task>/audit.md §10 }
  - { name: coverage_gate_report,      format: coverage-gate-audit@1 (one per task),                 recipient: memory audit chain + <task>/audit.md §10.4 }
  - { name: awh_gate_report,           format: awh-eval@1 (one per task, out-of-band rerun),         recipient: memory audit chain (memory.awh_gate_result) + <task>/audit.md §10.5 }
  - { name: caf_gate_report,           format: caf-gate@1 (one per task, code-audit floor),          recipient: memory audit chain (memory.caf_gate_result) + <task>/audit.md §10.6 }

skill_chain:
  # ── Phase: ready_to_implement → implementing (workflow start) ──
  - { step: 1,  skill: repo-context-map-author,                    inputs_from: { repo_root: repo_root, task_id: next_task_id },              outputs_to: context_map_draft,                         phase: "ready_to_implement → implementing" }
  - { step: 2,  skill: repo-context-map-audit,                     inputs_from: context_map_draft,                                        outputs_to: context_map }
  - { step: 3,  skill: architecture-decision-record-author,        inputs_from: { context_map: context_map, task: next_task },                outputs_to: adr_draft,                                 condition: 'context_map.files_outside_immediate_domain > 3' }
  - { step: 4,  skill: architecture-decision-record-audit,         inputs_from: adr_draft,                                                outputs_to: adr,                                       condition: "step 3 ran" }
  - { step: 5,  skill: edge-case-matrix-author,                    inputs_from: { task: next_task, context_map: context_map },                outputs_to: edge_case_matrix_draft }
  - { step: 6,  skill: edge-case-matrix-audit,                     inputs_from: edge_case_matrix_draft,                                   outputs_to: edge_case_matrix }
  - { step: 7,  skill: mock-contract-test-author,                  inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix },      outputs_to: mock_contracts_draft,                      condition: "task.has_external_dependency" }
  - { step: 8,  skill: mock-contract-test-audit,                   inputs_from: mock_contracts_draft,                                     outputs_to: mock_contracts,                            condition: "step 7 ran" }
  - { step: 9,  skill: implementation-plan-author,                 inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix, adr: adr },  outputs_to: impl_plan_draft }
  - { step: 10, skill: implementation-plan-audit,                  inputs_from: impl_plan_draft,                                          outputs_to: impl_plan }
  - { step: 11, skill: observability-injection-author,             inputs_from: { task: next_task, impl_plan: impl_plan },                    outputs_to: obs_injection_plan }
  - { step: 12, skill: observability-injection-audit,              inputs_from: obs_injection_plan,                                       outputs_to: obs_injection }
  # ── Phase transition: implementing → ready_to_review ──
  - { step: 13, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "implementing → ready_to_review", outcome: steps_1_to_12 }, outputs_to: backlog_mutation_phase_1, phase: "implementing → ready_to_review" }
  - { step: 14, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_1,                                 outputs_to: backlog_after_phase_1 }
  # ── Phase: ready_to_review → reviewing → ready_to_test ──
  - { step: 15, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "ready_to_review → reviewing", outcome: reviewer_claim }, outputs_to: backlog_mutation_phase_2 }
  - { step: 16, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_2,                                 outputs_to: backlog_after_phase_2 }
  - { step: 17, skill: code-review-author,                         inputs_from: { task: next_task, impl_diff: implementation_diff, adr: adr, edge_case_matrix: edge_case_matrix }, outputs_to: code_review_draft }
  - { step: 18, skill: code-review-audit,                          inputs_from: code_review_draft,                                        outputs_to: code_review_report }
  - { step: 19, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "reviewing → ready_to_test", outcome: code_review_report }, outputs_to: backlog_mutation_phase_3 }
  - { step: 20, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_3,                                 outputs_to: backlog_after_phase_3 }
  # ── Phase: ready_to_test → testing → done ──
  - { step: 21, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "ready_to_test → testing", outcome: tester_claim }, outputs_to: backlog_mutation_phase_4 }
  - { step: 22, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_4,                                 outputs_to: backlog_after_phase_4 }
  - { step: 23, skill: coverage-gate-author,                       inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix },      outputs_to: coverage_gate_draft }
  - { step: 24, skill: coverage-gate-audit,                        inputs_from: coverage_gate_draft,                                      outputs_to: coverage_gate_report }
  - { step: 25, skill: debugging-cycle-author,                     inputs_from: { task: next_task, coverage_report: coverage_gate_report },   outputs_to: debug_cycle_draft,                         condition: "coverage_gate_report.tests_failed > 0" }
  - { step: 26, skill: debugging-cycle-audit,                      inputs_from: debug_cycle_draft,                                        outputs_to: debug_trace,                               condition: "step 25 ran" }
  - { step: 27, skill: task-audit,                      inputs_from: { task: next_task, coverage_report: coverage_gate_report },   outputs_to: task_audit_report,                           description: "Post-implementation TRACE-004 closure — every §1 clause's cited test MUST be passed in coverage_gate_report. Pre-flight spec audit (`draft → ready_to_implement` transition) ran earlier, BEFORE this workflow; this is the closure check just before marking the task done." }
  - { step: 28, skill: awh-gate,                                   inputs_from: { task: next_task, module: next_task.module, goldenset: "modules/<module>/.awh/goldenset.yaml", baseline: "modules/<module>/.awh/eval-baseline.json" }, outputs_to: awh_gate_report, description: "Out-of-band independent rerun (the check step 27 is NOT). `awh eval <goldenset> --base-dir . --seeds 1 --baseline <baseline> --max-regression 0.0` reruns the task's §1 cited tests plus the module suite against the sealed, read-only baseline. GREEN (no task regressed) is REQUIRED to reach the done-flip; RED routes the task back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Tests sealed via `awh lock modules/<module>/tests`. Emits memory.awh_gate_result." }
  - { step: 29, skill: caf-gate,                                 inputs_from: { task: next_task, module: next_task.module, audit_profile: "modules/<module>/audit-profile.yaml", audit_baseline: "modules/<module>/.caf/" }, outputs_to: caf_gate_report, description: "Code-audit gate (absorbed from CyberSkill/code-audit-framework). Deterministic floor, no LLM: `bash scripts/caf_gate.sh <module>` runs the module's TARGET HEALTH via tools/caf/core/evals/verify-target.sh (the module's own RUN_COMMANDS - build/lint/typecheck/test - from modules/<module>/audit-profile.yaml, fail-closed) AND, when a sealed audit exists at modules/<module>/.caf/, `code-audit-validate --run modules/<module>/.caf --fail-on High` (no new High/Critical finding vs the sealed baseline). CLEAN is REQUIRED alongside the awh gate to reach the done-flip; RED routes the task back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Catches the class awh cannot: build/lint breaks, route 404s, changed data contracts (the CCAF/kymondongiap class). Emits memory.caf_gate_result. See docs/verification/caf-absorption-design.md." }
  - { step: 30, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "testing → done", outcome: { task_audit_report: task_audit_report, awh_gate_report: awh_gate_report, caf_gate_report: caf_gate_report } }, outputs_to: backlog_mutation_phase_5, condition: "awh_gate_report.outcome == GREEN AND caf_gate_report.outcome == CLEAN" }
  - { step: 31, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_5,                                 outputs_to: updated_backlog }

escalates_to:
  - { persona: chief-information-security-officer,                 when: "step 6 edge-case-matrix flags a SECURITY-class entry above warning + no corresponding ADR exists yet" }
  - { persona: chief-product-officer,                              when: "the task's acceptance criteria are ambiguous — step 5 cannot enumerate the boundary cases without product input" }
  - { persona: chief-financial-officer,                            when: "step 10 implementation-plan-audit total_estimate_pts > 25 % of the target-quarter capacity, OR cumulative session cost > $500 in compute" }

consults:
  - { persona: chief-privacy-officer,                              when: "the task touches personal data — verify GDPR / Vietnam Decree 13/2023 coverage in the edge-case matrix" }
  - { persona: chief-ai-officer,                                   when: "the task is AI-driven — verify EU AI Act risk-class + AI-specific test cases in the edge-case matrix" }

audit_hooks:
  - each skill emits one artefact_write row to the memory audit chain per its frontmatter audit.row_kind
  - between every phase transition (steps 13-14, 15-16, 19-20, 21-22, 30-31) backlog-state-update emits a `workflow_phase_complete` memory row
  - on successful `testing → done` transition (step 30) backlog-state-update emits a `workflow_complete` memory row with the full artefact summary
  - on circuit-breaker trip or any in-cycle failure → status reverts to `ready_to_implement` and the writer emits `task_routed_back` with the rework reason
  - HITL pauses (typically at step 4 ADR-self-approval boundary, step 24 coverage < 90 %, step 26 5-fail circuit-breaker trip) halt the chain

circuit_breaker:
  consecutive_test_failures_per_task: 5
  on_trip:
    - revert files to pre-execution state (`git restore` on touched paths)
    - mark task `ready_to_implement` in BACKLOG.md (with `routed_back_count += 1`) via step 30's rework branch
    - emit a `task_routed_back` memory audit row with the last debug_trace + reason `"circuit_breaker_5_consecutive_test_failures"`
    - proceed to the next eligible task (do NOT halt the outer loop)
---
# Ship Tasks — `chief-technology-officer/ship-tasks`

The canonical CTO workflow for **shipping** each `BACKLOG.md` task end-to-end through the full lifecycle. Renamed from `implement-backlog-frs` (v1.x) in v2.0.0 because the workflow doesn't just implement — it drives the task through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1.1). The old name suggested the workflow stopped at code-write; the new name reflects that it covers the full ship.

### One workflow, improvement folds in here

This is the single implementation workflow. There is no separate improvement track any more. Enterprise-hardening and refactoring work (formerly driven by the retired `run-improvement-program` and the `docs/improvement/` backlogs) are tasks too: an improvement item is a task carrying `class: improvement`, and it runs this exact lifecycle with the same mandatory human-acceptance gates. Section 1a covers how improvement tasks are declared, where they live under `docs/tasks/`, and how their gate suite is derived. The retired `run-improvement-program.md` points here; the two `cyberos-improve-*` skills that drove the old separate loop have been removed.

## 1. The state engine

Each task's frontmatter `status` is the record of truth; `docs/tasks/BACKLOG.md` is the index the state engine reads and keeps in lockstep with it (on any mismatch, repair the backlog toward frontmatter). The state engine reads BACKLOG.md before each iteration:

- **Eligible task** = first row whose status is `ready_to_implement` AND whose declared `depends_on` rows are all in `done` status.
- **Skipped statuses**: `draft` (not yet audited — handled by the `draft → ready_to_implement` chain, not this workflow), `implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing` (in-flight under another invocation — possibly the previous session of this workflow; pick those up by re-entering at the matching phase), `done` (terminal success — no work to do), `on_hold` / `closed` (operator-decided off-ramps).
- Pick the first eligible task. Run all 30 steps end-to-end. Between every phase transition the workflow physically updates the BACKLOG.md status cell via `backlog-state-update-author/-audit`. The mutation is atomic — same write that emits the `workflow_phase_complete` (or `workflow_complete` for the final transition) memory row.

### Backlog layout — one file, both classes

There is exactly ONE backlog: `docs/tasks/BACKLOG.md` indexes every task, `class: product` and `class: improvement` alike. Never create a second backlog file for improvement work.

- Row format: `- [status] task-ID-slug - title`, with an `(improvement)` suffix tag on `class: improvement` rows; product rows are untagged. Example: `- [ready_to_implement] TASK-007-rate-limit - login rate limiting (improvement)`.
- Grouping: small repos group rows into lifecycle-status sections (`ready_to_implement` / in flight / done / on_hold-closed — the init template); large monorepos may group by module with the status tag on each row. Both are conforming: frontmatter is the record of truth and every row carries its status either way.
- Task files all live under `docs/tasks/`: flat (`TASK-001-slug.md`) for small repos, module subfolders (`<module>/task-<MOD>-NNN-slug.md`) for monorepos. `improvement/` is a normal subfolder there for cross-cutting hardening tasks — not a separate top-level home.

### HITL — human-in-the-loop is REQUIRED

Human acceptance is mandatory (STATUS-REFERENCE.md §1.4, EXECUTION-DISCIPLINE.md §2a). The workflow drives the machine-verifiable transitions automatically, but two transitions are human-acceptance gates the agent MUST NOT cross by itself:

- **Review acceptance** (`reviewing → ready_to_test`, steps 19-20): the agent produces the code-review packet (steps 17-18) with every §1 clause mapped to a named test, then HALTS. A human records the approval verdict, which advances the cell.
- **Final acceptance** (`testing → done`, steps 30-31): the agent brings every machine gate green (coverage, TRACE-004, awh, caf), then HALTS. A human records the acceptance verdict. The agent NEVER self-sets `done`.

Between the gates the agent runs continuously and self-resolves everything it can verify (compile, lint, a test it broke, a red module gate on its own change); it does not pause for self-resolvable work. The only mandatory stops inside a task are these two verdicts.

An operator keeps the superset power to override any cell to any other cell at any time. Common operations:

- **Re-audit a shipped task** (replaces the v1.2.0 `mode: re_audit`): flip `done → ready_to_review`; on next invocation this workflow picks up at the `reviewing` phase and re-runs steps 15-30.
- **Skip review** for a trivial task: flip `ready_to_review → ready_to_test` directly (an explicit, recorded override).
- **Park an in-flight task**: flip `implementing → on_hold`; this workflow skips it on the next iteration.

Every human verdict or override emits one `memory.status_overridden` aux row capturing `{actor, task_id, prior_status, new_status, reason}`. This workflow detects the persisted state on resume by comparing it against the previous step's expected outcome.

### Failure / blocker semantics — route back to `ready_to_implement`

Any failure in `implementing` (steps 1-12), `reviewing` (steps 17-18), or `testing` (steps 23-28) routes the task back to `ready_to_implement` with `routed_back_count += 1`. The reason is recorded in:

1. A `memory.task_routed_back` aux audit row with the failure context (debug_trace, failing-test-name, or blocker reason).
2. A comment cell on the BACKLOG row (`<!-- routed back: <reason> -->`).
3. A future **Issue Request** artefact that will auto-spawn from the rework signal - future work, unscheduled (no task yet; see STATUS-REFERENCE §1.3 for the rework signal it would consume).

There are NO terminal failure statuses any more. The previous `[FAILED: UNRESOLVABLE ERROR]` and `[BLOCKED: ...]` enums are gone — failures are routing decisions, not states. Operator can still send a doomed task to `closed` manually via HITL.

## 1a. Improvement tasks (the folded-in hardening track)

Enterprise-hardening, refactoring, and audit-remediation work is not a separate track. Each such item is a task that runs this same lifecycle, with the same mandatory human-acceptance gates. It carries `class: improvement` in its frontmatter (a net-new feature carries `class: product`, the default). The class does not change the lifecycle; it records intent and selects the gate profile.

Where improvement tasks live:

- Module-scoped hardening (touches one module, e.g. memory) is an `task-<MODULE>-*` entry under `docs/tasks/<module>/`, exactly like a product task for that module.
- Cross-cutting hardening (spans modules, e.g. a repo-wide audit remediation) lives under `docs/tasks/improvement/` with its own README index — a normal subfolder of `docs/tasks/`, never a separate top-level tree. (In repos migrated from an old `docs/improvement/` backlog, that README also carries the migration record.)
- Backlog: improvement tasks are indexed in the SAME `docs/tasks/BACKLOG.md` as product tasks, tagged `(improvement)` on the row (see "Backlog layout" in section 1). There is no separate improvement backlog.

Gate profile by class:

- The gate suite for any task is derived from the touched module's `audit-profile.yaml` (the RUN_COMMANDS caf runs as target health) plus the coverage, TRACE, and edge-case gates that apply to every task.
- The awh out-of-band rerun (step 28) applies when the touched module has a sealed goldenset at `modules/<module>/.awh/`. An improvement task that touches a module without a goldenset declares awh N/A in its §1 and relies on coverage + caf + the review gate; it does not fabricate an awh pass. Standing up the goldenset can itself be an improvement task.
- No task, product or improvement, may weaken a protected invariant (auth model, tenant RLS, hash-chained audit, consent-gated capture, gateway-only model calls) to make a gate green. That is an operator-decision fork: park it and record why (EXECUTION-DISCIPLINE §2).

Everything else (selection from BACKLOG.md, one task with a commit per phase, the two human gates, route-back on failure) is identical to a product task.

## 2. Deep context mapping (steps 1-2)

Before any code is generated, the `repo-context-map` skill scans the repo for existing patterns for dependency injection, state management, error handling; database schemas + type interfaces in the task's declared module; files outside the task's immediate domain that the implementation would touch.

If more than three "outside-domain" files are flagged, the workflow auto-triggers an ADR (steps 3-4) using the existing `architecture-decision-record-author` + `-audit` pair. The ADR audit must pass at 10/10 against `adr-rubric@1.0` before the chain proceeds.

> **Spec audit was already done.** v2.0.0 drops the pre-flight `task-audit` at step 3 (which was v1.1.0's safety net). The reason: spec correctness is the responsibility of the `draft → ready_to_implement` chain. By the time this workflow picks up a task in `ready_to_implement`, the spec has already passed `audit_rubric@2.0` at 10/10. If the spec drifted afterwards (e.g. an AGENTS.md amendment broke a TRACE-001 citation), the operator either re-audits the spec via HITL (flip status back to `draft` so the spec chain re-runs) or runs `task-audit` standalone. The post-impl audit at step 27 still enforces TRACE-004 (every clause's cited test is `passed`).

## 3. Edge-case matrix (steps 5-6)

The `edge-case-matrix` skill generates a structured matrix covering: null/empty inputs; extreme bounds (off-by-one, integer overflow, time-zone DST, leap second); malformed payloads (truncated, oversized, non-UTF8); concurrent race conditions (double-submit, double-acknowledge, cross-tenant); security-class entries (auth bypass, RLS escape, injection).

The audit enforces the matrix is not vacuous — every category has ≥1 entry — and that SECURITY-class entries are paired to either an existing test or an ADR.

## 4. Mocks + contract tests (steps 7-8)

If `task.has_external_dependency = true` (CAPTCHA / 2FA / paywall / missing API keys / future service), `mock-contract-test-author` defines the **exact** expected Request/Response shape of the missing service plus a Mock Service that **passes** the contract test. The task's frontmatter gets `implementation_kind: mocked` (per STATUS-REFERENCE §3) so the mocked-against-real distinction is preserved without polluting the lifecycle status.

The contract test stays in the suite forever — when the real dependency lands, swapping the mock out is a single import change and the contract guarantees behavioural parity.

## 5. Implementation (steps 9-10)

The `implementation-plan-author` + `-audit` pair drives the actual code. Inputs are the task, the edge-case matrix, and the (optional) ADR. The audit enforces: (a) every edge-case row is addressed in the plan, (b) the plan respects the existing patterns identified in step 1, (c) capacity estimate is reasonable.

## 6. Observability injection (steps 11-12)

`observability-injection-author` walks the critical paths of the new code and emits: structured-log lines at every state transition (incl. `tenant_id`, `subject_id` when present); trace spans wrapping every external IO; counter increments for every error branch.

The audit checks coverage: ≥80 % of branches have a log/metric/trace point.

## 7. Phase transition: `implementing → ready_to_review` (steps 13-14)

`backlog-state-update-author/-audit` flips the BACKLOG status cell from `implementing` to `ready_to_review` and emits a `workflow_phase_complete` memory row carrying the artefact bundle (context_map, adr?, edge_case_matrix, mock_contracts?, impl_plan, obs_injection).

## 8. Review (steps 15-20)

After the implementing artefacts are settled, status flips to `reviewing` (steps 15-16) and `code-review-author` reads the diff against the §1 clauses and the edge-case matrix, flagging gaps and naming the test cases that would prove each clause. The audit confirms every §1 clause has a named test reference and every edge-case-matrix row has either a test or an ADR justification.

Review acceptance is a mandatory human gate (HITL, see the state engine). The agent presents the review packet with every §1 clause mapped to a named test, then halts; on a recorded human approval verdict, status flips to `ready_to_test` (steps 19-20). On rejection (review uncovers a missing clause or an unaddressed edge case) the task routes back to `ready_to_implement` (see §1 failure semantics).

> v2.0.0 introduced `code-review-author` and `code-review-audit` as dedicated skills for the explicit `reviewing` phase (before that, review work was implicit in the post-impl `task-audit` call). Both skills exist at `modules/skill/` and are vendored in the payload (chain-coverage enforces their presence at build time - TASK-SKILL-116).

## 9. Testing phase: coverage gate + post-impl task audit + awh + caf gates (steps 21-29)

Status flips to `testing` (steps 21-22). `coverage-gate-author` runs the test suite, computes coverage on touched files, and fails the gate if per-file coverage on files touched in this task is < 90 %. The audit emits the raw terminal output of the coverage tool as the artefact.

If any test fails, `debugging-cycle-author` runs the multi-vector pass (classify failure vector — state/network/memory/logic/flake; output hypothesis + targeted change; re-run; after 5 consecutive failures revert + trip circuit breaker). The audit emits the full hypothesis-and-attempt log.

After coverage + debugging settle, `task-audit` runs the post-impl pass at step 27 to enforce **TRACE-004** — every §1 clause's cited test MUST appear as `passed` in `coverage_gate_report`. A §1 clause may have an AC and a named test from the pre-flight pass, but if the actual test is failing or absent from the coverage report, the task cannot ship `done`.

## 10. Phase transition: `testing → done` (steps 30-31)

The final phase transition. Outcomes derived by steps 27-29 (post-impl audit + the awh out-of-band test-rerun gate + the caf code-audit gate). Both gates must pass: awh proves the tests still pass; caf proves the module's own build/lint/typecheck/test still run and the audit finds no new High/Critical issue. They are complementary - awh catches test regressions, caf catches the class awh cannot see (a build/lint break, a route that 404s, a changed data contract). Green gates are necessary but not sufficient: this transition is a human-acceptance gate, so the agent halts once the gates are green and a human records the acceptance verdict that sets `done`. The agent never sets `done` itself (HITL required, STATUS-REFERENCE §1.4).

| Step 27 audit + step 28 awh gate + step 29 caf gate + circuit breaker status                                                                                                                          | New status                      | Mutation                                                                                                               |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| All TRACE-001..005 passing + 0 failed tests + awh gate GREEN (independent rerun, no task regressed vs the sealed baseline) + caf gate CLEAN (target health PASS + no new High/Critical audit finding) + recorded human acceptance verdict | `done`                        | `workflow_complete` memory row (written when the human records acceptance), BACKLOG cell `testing → done`         |
| TRACE-004 fails (test exists per spec but isn't passing)                                                                                                                                              | `ready_to_implement` (rework) | `task_routed_back` memory row with `reason: "trace-004: <test_name> not in coverage_gate_report"`                    |
| awh gate RED (a task regressed vs the sealed baseline, or the task's cited test is not passing on independent rerun)                                                                                    | `ready_to_implement` (rework) | `task_routed_back` + `memory.awh_gate_result{outcome: RED}`, `reason: "awh-gate: <task> regressed"`                |
| caf gate RED (target health failed - a RUN_COMMAND broke - or the audit raised a new High/Critical finding)                                                                                           | `ready_to_implement` (rework) | `task_routed_back` + `memory.caf_gate_result{outcome: RED}`, `reason: "caf-gate: <target-health-fail or finding>"` |
| Circuit breaker tripped during steps 25-26                                                                                                                                                            | `ready_to_implement` (rework) | `task_routed_back` memory row with `reason: "circuit_breaker_5_consecutive_test_failures"`                           |

The top row's `done` is not written by the agent: when the gates are green it halts at the acceptance gate, a human records the acceptance verdict, and that verdict writes `done` (HITL required, STATUS-REFERENCE §1.4).

The workflow commits the diff to the working tree (operator runs `git add . && git commit && git push` to publish).

## 11. Outer loop

The CUO supervisor invokes this workflow in a loop:

```
while ! stop_signal:
    next_task = read_backlog().next_eligible()   # deterministic: see 'Queue selection' in Resume semantics below
    if next_task is None: break        # backlog drained
    invoke_workflow("chief-technology-officer/ship-tasks", { repo_root, next_task })
```

The supervisor handles persistence (state survives across sessions because the truth is in BACKLOG.md + the memory chain), parallelism (multiple tasks may run in parallel when their dependency cones don't overlap), and observability (the per-phase `workflow_phase_complete` + the final `workflow_complete` rows are enough to reconstruct the run).

## 11a. Batch selection and parallel shipping (v2.5.0, TASK-IMP-074)

One-task-at-a-time is no longer the only sanctioned mode. The default is now BATCH shipping of parallel-safe tasks:

- **Batch selection.** The eligible set is every `ready_to_implement` task whose `depends_on` rows are all `done`. A batch is a maximal subset of that set whose members are pairwise independent: no `depends_on`/`blocks` edge between any two members, AND no overlap between their declared cones (frontmatter `new_files` + `modified_files` + `service`). tasks whose cones overlap stay serial relative to each other, in Queue-selection priority order.
- **Batched execution.** Phases MAY run batch-wide (map the repo once, implement all members, review all members, test all members) and commits MAY batch per phase across members. What stays strictly per-task: the artefact set (context map, matrix, plan, review packet, coverage gate), the ship-manifest, the BACKLOG/frontmatter status cells, and the recorded HITL verdicts.
- **HITL is unchanged by batching.** Both human-acceptance gates apply to every member individually. A single human reply MAY record verdicts for many members at once (one utterance, N recorded per-task verdicts — e.g. "approve all" / "accept all"); batching reduces round-trips, never guarantees.
- **Unlock rescan.** Whenever any task reaches `done`, re-scan the backlog for tasks whose `depends_on` just became fully satisfied; append the newly-eligible, cone-independent ones to the running batch queue and continue (EXECUTION-DISCIPLINE §1 — no pause to ask). Cone-overlapping unlocks queue serially behind the member they overlap with.
- **Status-page sync rule (group A).** Every backlog-state-update write rides with a regenerated `docs/status/` page in the same commit — enforced mechanically by the pre-commit hook (`.cyberos/lib/status-page.sh` + auto-stage on any `docs/tasks/**`, `CHANGELOG.md`, or `VERSION` change). A status cell that moves without the page moving is a bug.
- **Swarm execution (v2.6.0).** A batch SHOULD be shipped by a swarm — one sub-agent per member, dispatched in a single parallel round — not by looping the members serially in one agent. Cone-independence is exactly the property that makes this safe: members touch disjoint files, so their edits cannot race. Serialising a batch that was selected for parallelism throws away the only reason to have batched it.
  - Dispatch every member of the round in ONE message with N parallel calls. N sequential dispatches is a serial loop with extra steps.
  - Each sub-agent owns its member end-to-end and returns the artefact set. The parent owns what stays per-batch: the branch, the commits, the HITL round-trips, the BACKLOG writes.
  - Cap the round at the point where reviewing N diffs stops being possible. Sub-agents are cheap; a human reading 12 unrelated diffs at one gate is not. When the batch exceeds a reviewable round, ship it as consecutive rounds on the same branch.
  - A sub-agent that hits a §2 halt returns it; the parent collects halts and surfaces them together rather than stopping the whole round on the first one.
- **One branch per BATCH, not per task (v2.6.0).** The batch is the unit of review and the unit of merge, so it is the unit of branching. Name it `batch/<n>-<short-theme>` (e.g. `batch/3-auth-hardening`). Every member commits to that branch, per phase, with its own conventional commit — per-task commits stay, per-task branches go. Rationale: N branches for N tasks that were selected *because they are independent* produces N PRs that touch disjoint files and merge in any order — pure ceremony. Cone-overlapping tasks were never in the batch to begin with.
  - The next batch branches from the previous batch's merge, not from its tip, unless the operator says otherwise. Unlock rescan means batch N+1's eligibility usually depends on batch N being `done`.
  - A task routed back to `ready_to_implement` mid-batch leaves the batch; it re-enters selection later and MUST NOT hold its batch's branch open.
- **Mixed agent-human and human-only work: the guideline lives IN the task (v2.6.0, operator request).** When a task needs the operator to do something the agent must not (§2) or genuinely cannot, write the step-by-step guideline INTO that task's `spec.md` under `## Operator steps` — never into a new file. Never create `SETUP.md`, `RUNBOOK-<task>.md`, `INSTRUCTIONS.md`, or any sibling artefact for it.
  - Reason: a separate file is a second place the reader must find, and it rots the moment the task moves. The operator opens the task; the steps are there. Anything else asks them to hold two documents in their head and guess which is current.
  - Shape: numbered, copy-pasteable, one command or one click per step, with the expected output stated. If a step is GUI-only and the agent has OS/browser control, the agent does it (`EXECUTION-DISCIPLINE.md` §2b) and the guideline records what was driven — it does not ask the operator to repeat it.
  - The task halts at that gate only if the steps are genuinely operator-only under §2. "The agent wrote a guideline" is not itself a halt.

## 12. No partial-ship-and-pause within a task

The workflow MUST drive **all phases of a task to completion in one continuous session** (or route back to `ready_to_implement` cleanly). It runs continuously under the halt-only doctrine in [`../../EXECUTION-DISCIPLINE.md`](../../EXECUTION-DISCIPLINE.md): the agent stops ONLY for an operator-decision fork, a manual/operator-only action (push, deploy, destructive op, secret), a hard blocker past the circuit-breaker budget, or the operator stop signal. Everything else — compile/lint/clippy, a test or module gate the agent's own change broke, the order of slices or tasks — the agent self-resolves and continues.

**Rules:**

1. Read the full gap list + slice plan BEFORE running any step.
2. Don't ask between phases for self-resolvable work — continuation is implied by "drive this task". The two human-acceptance gates (review approval at `reviewing → ready_to_test`, and final acceptance at `testing → done`) are the exception: halt for the recorded human verdict there, since HITL is required.
3. Commit per phase for git-history hygiene; each phase = own conventional commit + verify gate.
4. Do NOT pause between tasks either. The outer loop (§11) advances to the next eligible task on its own; halt between tasks only on an `EXECUTION-DISCIPLINE.md` §2 condition, never just because one task finished.
5. If genuinely blocked mid-task (e.g. needs ADR-class operator decision), DOCUMENT the block in §10.7 of the task's audit.md, route back to `ready_to_implement` with `routed_back_count += 1` and `reason: "<blocker>"`. Do NOT silently ship a partial phase and walk away.

See `task-audit` skill §9.1 for the full clause + grandfathered exceptions.

## Resume semantics (ship-manifest@1) - added by TASK-CUO-206

Every run maintains a per-task run-state manifest at `docs/tasks/.workflow/<task-ID>.ship.json`,
shaped by `modules/skill/contracts/task/SHIP-MANIFEST.md` (ship-manifest@1). The manifest is a
CACHE of proven work, never an authority - task frontmatter and BACKLOG.md remain the only record of truth.

**Write points.** The manifest MUST be rewritten after EVERY completed, failed, or conditionally-skipped
step - no step's outcome goes unrecorded. Writes are two-phase atomic (`.tmp.<nonce>` then rename),
mirroring the memory-protocol discipline. Each step entry records `{index, skill, status, artefact_path,
artefact_sha256, verdict, completed_at}`; `task_sha256` (hash of the task spec at run start) and
`workflow_version` are pinned at manifest creation.

**Resume.** On invocation for a task whose manifest exists:

1. `workflow_version` mismatch -> needs_human. Never a silent mixed-version run.
2. `task_sha256` mismatch (task spec edited since run start) -> every step is stale; restart at step 1
   (history and `routed_back_count` are retained).
3. Otherwise re-hash EVERY recorded `artefact_sha256` against disk. The earliest mismatch marks that
   step and all later steps stale - redo from there. All intact -> continue at the first non-done step.
4. Echo the resume line before running: `resume <task-ID>: steps 1-N verified (K artefacts, hashes OK),
   continuing at step M/31 (<skill>). routed_back_count=R`.

**Human gates re-ask.** A manifest can never authorize a gate: resuming at a HITL gate step re-requests
the human approval. A recorded `hitl.requested_at` is when the question was asked - it is NOT the answer,
and MUST NOT be treated as approval.

**Terminal handling.** When the task reaches `done` (HITL gate 2 passed), delete the manifest. On route-back
to `ready_to_implement`, keep it with `routed_back_count += 1` - the next run restarts at step 1 by the
staleness rule but retains count and history.

**Queue selection (no task id given).** Among tasks at `ready_to_implement` whose `depends_on` are all `done`:
order by priority (MUST before SHOULD before COULD), then `created` ascending, then id ascending. Echo the
selection before step 1: `queue: picked <id> (priority=<p>, created=<d>) over <n> other eligible tasks`.
Reference implementation: `modules/cuo/cuo/ship_manifest.py` (doc-driven agents apply this section directly).

## Cross-references

- Execution discipline (continuous run, halt-only conditions): [`../../EXECUTION-DISCIPLINE.md`](../../EXECUTION-DISCIPLINE.md). Added 2026-06-20 (v2.2.0): the agent halts only for an operator-decision fork, a manual/operator-only action, a hard blocker past the budget, or the operator stop signal; it self-resolves everything else and runs continuously across phases and tasks.
- Task lifecycle: `modules/skill/contracts/task/STATUS-REFERENCE.md` (10-state enum, transitions, HITL semantics).
- Original prompt source: operator's "Zero-Touch Principal Engineer (Unattended Execution)" — absorbed 2026-05-18.
- BACKLOG state engine: `docs/tasks/BACKLOG.md`.
- Run-state manifest contract: `modules/skill/contracts/task/SHIP-MANIFEST.md` (ship-manifest@1, TASK-CUO-206). Manifests are gitignored session state under `docs/tasks/.workflow/`.
- Companion workflow: `chief-technology-officer/architect-new-system` — produces the tasks this workflow consumes.
- No-partial-ship rule: `task-audit` skill §9.1.
- Pre-flight spec audit (separate chain): `task-audit` skill — drives `draft → ready_to_implement`.
- Test coverage audit: `coverage-gate-audit` skill — drives `testing → done`.
- Out-of-band gate (step 28): `awh eval` against `modules/<module>/.awh/goldenset.yaml` seals `testing → done`. A task cannot reach `done` unless awh independently re-runs its §1 cited tests + module suite GREEN against the sealed, read-only baseline. See the awh absorption design.
- Code-audit gate (step 29): `bash scripts/caf_gate.sh <module>` (absorbed from CyberSkill/code-audit-framework, vendored at `tools/caf/`). Deterministic floor - target health (`tools/caf/core/evals/verify-target.sh` runs the module's own RUN_COMMANDS from `modules/<module>/audit-profile.yaml`) plus, when present, `code-audit-validate` against the sealed audit at `modules/<module>/.caf/`. CLEAN is required alongside the awh gate. See `docs/verification/caf-absorption-design.md`.

## Distribution sync — rules to channels (v2.5.0, TASK-IMP-074 group C)

The rules this document (and every `modules/skill/` contract) defines are DISTRIBUTED: they ship inside the cyberos-init payload to standalone/self-hosted `.cyberos/` installs, the Claude plugin, the MCP server, and the npx CLI. Rule changes MUST reach every channel through this auto-hook chain, never by hand-copying:

- **Build hook (local):** `.githooks/pre-commit` rebuilds `dist/cyberos` and runs `check-version-sync.sh` whenever `modules/cuo/**`, `modules/skill/**`, or `tools/cyberos-init/**` is staged — a rule edit cannot be committed without a fresh, stamp-verified payload build.
- **Push hook (CI):** `payload-gate.yml` re-proves the same invariant on every push/PR touching those paths.
- **Release hook:** `release.yml`'s payload job builds and uploads the stamped payload as release assets on every tag — the pull source for `cyberos update` / `version.sh` / plugin + MCP consumers.
- **Deploy hook:** `deploy.yml`'s docs job also triggers on `modules/cuo/**` and `modules/skill/**`, so the published site reflects rule changes without waiting for a release.
- **Drift signal:** the payload's `manifest.yaml` carries `rules_sha` — a deterministic content fingerprint over the distributed rule trees (`cuo/ plugin/ mcp/ cli/ memory/`). Channels compare it to detect rule drift even when VERSION is unchanged; `check-version-sync.sh` fails any payload missing it. Client-side comparison in `cyberos update`/plugin/MCP is the designated follow-up (TASK-IMP-074 §9).

### Pre-push / pre-install re-verification (v2.6.0, operator request)

Before `git push`, and before installing the payload onto ANY other repo, re-prove the chain
end-to-end. The hooks above fire on *staged paths*; they do not prove the built artefact is
the one a consumer will actually receive.

Run, in order, and read the output rather than the exit code alone:

1. `bash tools/cyberos-init/build.sh` — the payload a consumer pulls, rebuilt from source.
2. `bash tools/cyberos-init/check-version-sync.sh dist/cyberos` — VERSION identical across every stamped artefact.
3. `bash tools/cyberos-init/check-chain-coverage.sh dist/cyberos` — every vendored skill reachable from the chain.
4. `bash scripts/tests/run_all.sh` — every suite, including the payload suites under `tools/cyberos-init/tests/`.
5. **Install into a scratch repo and look at the result**, not at the script that produces it:
   ```
   rm -rf /tmp/verify && mkdir -p /tmp/verify && cd /tmp/verify && git init -q .
   bash <repo>/dist/cyberos/install.sh .
   find .cyberos -name 'feature.md'     # non-empty, or task-author HALTs on first task
   ```

**Why the last one is not optional.** On 2026-07-15 `build.sh` shipped no per-type templates
while `task-author` dispatched on them; reading `build.sh` did not reveal it, and `find` on a
real install did — in one command. Every channel (`.cyberos/` install, Claude plugin, MCP
server, npx CLI) receives the payload, not the repo. A rule that is correct in `modules/` and
absent from `dist/` is correct nowhere that matters.

The vendored copy under a target's `.cyberos/` is what actually executes. `build.sh` refreshes
`dist/`; only `install.sh` lays `dist/` into `.cyberos/`. Skipping the install step means the
target keeps running the OLD rules while `dist/` looks current — the exact failure that made a
fixed renderer produce stale output for a full session.

### Closing report (v2.6.0, operator request)

End every ship run — batch or single — with a short suggestion of next steps or possible
improvements. Not a recap of what was done; the operator watched that. What is now unblocked,
what the run exposed that is worth doing next, and what you would do if it were your call.
Say when the honest answer is "nothing — merge it".

*End of `chief-technology-officer/ship-tasks.md` workflow.*
