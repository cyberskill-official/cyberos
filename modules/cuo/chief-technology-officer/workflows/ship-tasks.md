---
workflow_id: chief-technology-officer/ship-tasks
workflow_version: 2.8.0
purpose: Drive each eligible task in `docs/tasks/BACKLOG.md` end-to-end through the full lifecycle — from `ready_to_implement` through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1.1). Deep-maps the repo, generates the edge-case matrix, implements with 90 % coverage on touched files, injects observability, records architectural deviations via ADRs, runs the multi-vector debugger with a 5-fail circuit breaker, runs the testing gate (`coverage-gate-author`/`-audit`), and physically updates BACKLOG.md status between every phase transition. Failure or blocker at any downstream phase routes the task back to `ready_to_implement` (STATUS-REFERENCE §1.3) with `routed_back_count += 1`.
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
  - { name: reconcile_report,          format: reconcile-report@1 (conditional, one per drifted entry), recipient: memory audit chain + HITL gate }
  - { name: caf_gate_report,           format: caf-gate@1 (one per task, code-audit floor),          recipient: memory audit chain (memory.caf_gate_result) + <task>/audit.md §10.6 }

skill_chain:
  # ── Conditional entry: the task claims work this workflow did not perform (see 'Reconcile entry' §) ──
  # `judgment:` is ADVISORY host-routing metadata (§11e). A host MAY route on it; nothing here reads it. Never a model name.
  - { step: 0,  skill: task-reconcile,                             inputs_from: { task: next_task, repo_root: repo_root },                    outputs_to: reconcile_report,                          judgment: mechanical, condition: 'entry state drifted — status past ready_to_implement AND (no ship-manifest OR manifest verify fails OR the claimed phase artefact set is missing)', phase: "entry (reconcile)" }
  # ── Phase: ready_to_implement → implementing (workflow start) ──
  - { step: 1,  skill: repo-context-map-author,                    inputs_from: { repo_root: repo_root, task_id: next_task_id },              outputs_to: context_map_draft,                         judgment: high, phase: "ready_to_implement → implementing" }
  - { step: 2,  skill: repo-context-map-audit,                     inputs_from: context_map_draft,                                        outputs_to: context_map,                               judgment: medium }
  - { step: 3,  skill: architecture-decision-record-author,        inputs_from: { context_map: context_map, task: next_task },                outputs_to: adr_draft,                                 judgment: high, condition: 'context_map.files_outside_immediate_domain > 3' }
  - { step: 4,  skill: architecture-decision-record-audit,         inputs_from: adr_draft,                                                outputs_to: adr,                                       judgment: medium, condition: "step 3 ran" }
  - { step: 5,  skill: edge-case-matrix-author,                    inputs_from: { task: next_task, context_map: context_map },                outputs_to: edge_case_matrix_draft,                    judgment: high }
  - { step: 6,  skill: edge-case-matrix-audit,                     inputs_from: edge_case_matrix_draft,                                   outputs_to: edge_case_matrix,                          judgment: medium }
  - { step: 7,  skill: mock-contract-test-author,                  inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix },      outputs_to: mock_contracts_draft,                      judgment: high, condition: "task.has_external_dependency" }
  - { step: 8,  skill: mock-contract-test-audit,                   inputs_from: mock_contracts_draft,                                     outputs_to: mock_contracts,                            judgment: medium, condition: "step 7 ran" }
  - { step: 9,  skill: implementation-plan-author,                 inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix, adr: adr },  outputs_to: impl_plan_draft,                     judgment: high }
  - { step: 10, skill: implementation-plan-audit,                  inputs_from: impl_plan_draft,                                          outputs_to: impl_plan,                                 judgment: medium }
  - { step: 11, skill: observability-injection-author,             inputs_from: { task: next_task, impl_plan: impl_plan },                    outputs_to: obs_injection_plan,                        judgment: medium }
  - { step: 12, skill: observability-injection-audit,              inputs_from: obs_injection_plan,                                       outputs_to: obs_injection,                             judgment: medium }
  # ── Phase transition: implementing → ready_to_review ──
  - { step: 13, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "implementing → ready_to_review", outcome: steps_1_to_12 }, outputs_to: backlog_mutation_phase_1, judgment: mechanical, phase: "implementing → ready_to_review" }
  - { step: 14, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_1,                                 outputs_to: backlog_after_phase_1,                     judgment: medium }
  # ── Phase: ready_to_review → reviewing → ready_to_test ──
  - { step: 15, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "ready_to_review → reviewing", outcome: reviewer_claim }, outputs_to: backlog_mutation_phase_2, judgment: mechanical }
  - { step: 16, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_2,                                 outputs_to: backlog_after_phase_2,                     judgment: medium }
  - { step: 17, skill: code-review-author,                         inputs_from: { task: next_task, impl_diff: implementation_diff, adr: adr, edge_case_matrix: edge_case_matrix }, outputs_to: code_review_draft, judgment: high }
  - { step: 18, skill: code-review-audit,                          inputs_from: code_review_draft,                                        outputs_to: code_review_report,                        judgment: medium }
  - { step: 19, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "reviewing → ready_to_test", outcome: code_review_report }, outputs_to: backlog_mutation_phase_3, judgment: mechanical }
  - { step: 20, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_3,                                 outputs_to: backlog_after_phase_3,                     judgment: medium }
  # ── Phase: ready_to_test → testing → done ──
  - { step: 21, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "ready_to_test → testing", outcome: tester_claim }, outputs_to: backlog_mutation_phase_4, judgment: mechanical }
  - { step: 22, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_4,                                 outputs_to: backlog_after_phase_4,                     judgment: medium }
  - { step: 23, skill: coverage-gate-author,                       inputs_from: { task: next_task, edge_case_matrix: edge_case_matrix },      outputs_to: coverage_gate_draft,                       judgment: medium }
  - { step: 24, skill: coverage-gate-audit,                        inputs_from: coverage_gate_draft,                                      outputs_to: coverage_gate_report,                      judgment: medium }
  - { step: 25, skill: debugging-cycle-author,                     inputs_from: { task: next_task, coverage_report: coverage_gate_report },   outputs_to: debug_cycle_draft,                         judgment: high, condition: "coverage_gate_report.tests_failed > 0" }
  - { step: 26, skill: debugging-cycle-audit,                      inputs_from: debug_cycle_draft,                                        outputs_to: debug_trace,                               judgment: medium, condition: "step 25 ran" }
  - { step: 27, skill: task-audit,                      inputs_from: { task: next_task, coverage_report: coverage_gate_report },   outputs_to: task_audit_report,                           judgment: high, description: "Post-implementation TRACE-004 closure — every §1 clause's cited test MUST be passed in coverage_gate_report. Pre-flight spec audit (`draft → ready_to_implement` transition) ran earlier, BEFORE this workflow; this is the closure check just before marking the task done." }
  - { step: 28, skill: awh-gate,                                   inputs_from: { task: next_task, module: next_task.module, goldenset: "modules/<module>/.awh/goldenset.yaml", baseline: "modules/<module>/.awh/eval-baseline.json" }, outputs_to: awh_gate_report, judgment: medium, description: "Out-of-band independent rerun (the check step 27 is NOT). `awh eval <goldenset> --base-dir . --seeds 1 --baseline <baseline> --max-regression 0.0` reruns the task's §1 cited tests plus the module suite against the sealed, read-only baseline. GREEN (no task regressed) is REQUIRED to reach the done-flip; RED routes the task back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Tests sealed via `awh lock modules/<module>/tests`. Emits memory.awh_gate_result." }
  - { step: 29, skill: caf-gate,                                 inputs_from: { task: next_task, module: next_task.module, audit_profile: "modules/<module>/audit-profile.yaml", audit_baseline: "modules/<module>/.caf/" }, outputs_to: caf_gate_report, judgment: medium, description: "Code-audit gate (absorbed from CyberSkill/code-audit-framework). Deterministic floor, no LLM: `bash scripts/caf_gate.sh <module>` runs the module's TARGET HEALTH via tools/caf/core/evals/verify-target.sh (the module's own RUN_COMMANDS - build/lint/typecheck/test - from modules/<module>/audit-profile.yaml, fail-closed) AND, when a sealed audit exists at modules/<module>/.caf/, `code-audit-validate --run modules/<module>/.caf --fail-on High` (no new High/Critical finding vs the sealed baseline). CLEAN is REQUIRED alongside the awh gate to reach the done-flip; RED routes the task back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Catches the class awh cannot: build/lint breaks, route 404s, changed data contracts (the CCAF/kymondongiap class). Emits memory.caf_gate_result. See docs/verification/caf-absorption-design.md." }
  - { step: 30, skill: backlog-state-update-author,                inputs_from: { task: next_task, transition: "testing → done", outcome: { task_audit_report: task_audit_report, awh_gate_report: awh_gate_report, caf_gate_report: caf_gate_report } }, outputs_to: backlog_mutation_phase_5, judgment: mechanical, condition: "awh_gate_report.outcome == GREEN AND caf_gate_report.outcome == CLEAN" }
  - { step: 31, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_5,                                 outputs_to: updated_backlog,                           judgment: medium }

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

The canonical CTO workflow for **shipping** each `BACKLOG.md` task end-to-end through the full lifecycle. The workflow doesn't just implement — it drives the task through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `modules/skill/contracts/task/STATUS-REFERENCE.md` §1.1). The old name suggested the workflow stopped at code-write; the new name reflects that it covers the full ship.

### One workflow, improvement folds in here

This is the single implementation workflow. There is no separate improvement track any more. Enterprise-hardening and refactoring work (formerly driven by the retired `run-improvement-program` and the `docs/improvement/` backlogs) are tasks too: an improvement item is a task carrying `class: improvement`, and it runs this exact lifecycle with the same mandatory human-acceptance gates. Section 1a covers how improvement tasks are declared, where they live under `docs/tasks/`, and how their gate suite is derived. The retired `run-improvement-program.md` points here; the two `cyberos-improve-*` skills that drove the old separate loop have been removed.

## 1. The state engine

Each task's frontmatter `status` is the record of truth; `docs/tasks/BACKLOG.md` is the index the state engine reads and keeps in lockstep with it (on any mismatch, repair the backlog toward frontmatter). The state engine reads BACKLOG.md before each iteration:

- **Eligible task** = first row whose status is `ready_to_implement` AND whose declared `depends_on` rows are all in `done` status.
- **Skipped statuses**: `draft` (not yet audited — handled by the `draft → ready_to_implement` chain, not this workflow), `implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing` (in-flight under another invocation — possibly the previous session of this workflow; pick those up by re-entering at the matching phase), `done` (terminal success — no work to do), `on_hold` / `closed` (operator-decided off-ramps).
- Pick the first eligible task. Run all 31 steps end-to-end. Between every phase transition the workflow physically updates the BACKLOG.md status cell via `backlog-state-update-author/-audit`. The mutation is atomic — same write that emits the `workflow_phase_complete` (or `workflow_complete` for the final transition) memory row.

### Backlog layout — one file, both classes

There is exactly ONE backlog: `docs/tasks/BACKLOG.md` indexes every task, `class: product` and `class: improvement` alike. Never create a second backlog file for improvement work.

- Row format: `- [status] task-ID-slug - title`, with an `(improvement)` suffix tag on `class: improvement` rows; product rows are untagged. Example: `- [ready_to_implement] TASK-007-rate-limit - login rate limiting (improvement)`.
- Grouping: small repos group rows into lifecycle-status sections (`ready_to_implement` / in flight / done / on_hold-closed — the init template); large monorepos may group by module with the status tag on each row. Both are conforming: frontmatter is the record of truth and every row carries its status either way.
- Task files all live under `docs/tasks/`: flat (`TASK-001-slug.md`) for small repos, module subfolders (`<module>/task-<MOD>-NNN-slug.md`) for monorepos. `improvement/` is a normal subfolder there for cross-cutting hardening tasks — not a separate top-level home.

Backlog writes are executed by `tools/install/docs-tools/backlog-mutate.mjs` (`.cyberos/docs-tools/backlog-mutate.mjs` in installed repos) - the byte-discipline executor for `backlog-state-update` mutations: it flips one status cell only after verifying the old line byte-for-byte (refusing on drift with a non-zero exit), inserts one row under the uniqueness gate with the section's own grammar and stem-ascending placement, and keeps section-header counts true - never hand-sed (TASK-IMP-085).

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
3. A `type: bug` task auto-drafted from the rework signal (LANDED 2026-07-14 — this replaced the previously planned "Issue Request" artefact; see STATUS-REFERENCE §1.3 for the field mapping and the second intake path via CUO triage).

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

Step 27's terminal `pass`/`fail` verdict is ALSO logged, one append-only row, to the skill-trust ledger (`docs/tasks/.workflow/skill-trust.tsv`) via `node tools/install/docs-tools/skill-log.mjs append --skill <skill> --verdict <pass|fail> --task <task-id>` (TASK-IMP-113). This is a **measurement side-effect only**: the tier label `skill-log.mjs --render` prints is INFORMATIONAL (spec §1.4) — no step, gate, or queue in this workflow reads a tier to decide anything, and the transition table below is untouched by it. The ledger is append-only run-state (gitignored by the install seed), a report for the operator asking "which skills actually work?" — never a gate. A `needs_human` pause or a run cut mid-flight is not a terminal verdict and is not logged.

Acceptance evidence for content deliverables — a backlog row, a doc passage, anything whose deliverable IS bytes in a file — MUST be measured on the committed object (`git show <commit>:<path>`), never a working view (v2.6.3, TASK-IMP-092). A working view can read back exactly what was just written into it while no commit carries the change; the committed object is the only thing a reviewer, a consumer repo, or the next session receives. (Learned 2026-07-16, TASK-IMP-086 incident: every working-view read looked consistent while no commit ever carried the rows.)

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
    batch = read_backlog().next_batch()          # MANDATORY (v2.8.0): a greedy cone-independent
                                                 # set, not one task. Emits batch-selection@1.
    if batch is empty: break           # backlog drained
    if len(batch) > 1: dispatch_swarm(batch)     # one sub-agent per member, ONE parallel round
    else:              ship(batch[0])            # a 1-member batch is serial by arithmetic, not by choice
    # NO re-invoke here. The `while` above IS the loop. The pre-v2.8.0 shape ended each pass by
    # re-invoking the workflow with `next_task`; the batch rewrite replaced that variable and left
    # the call behind, so the line named something nothing assigns AND re-dispatched work that the
    # two lines above had already shipped. (External review, 2026-07-17.)
```

The supervisor handles persistence (state survives across sessions because the truth is in BACKLOG.md + the memory chain), parallelism (multiple tasks may run in parallel when their dependency cones don't overlap), and observability (the per-phase `workflow_phase_complete` + the final `workflow_complete` rows are enough to reconstruct the run).

## 11a. Batch selection and parallel shipping (v2.5.0, TASK-IMP-074)

One-task-at-a-time is no longer the only sanctioned mode. The default is now BATCH shipping of parallel-safe tasks:

- **Batch selection is EXECUTED, not preferred (v2.8.0).** Before step 1 the workflow MUST run `node .cyberos/docs-tools/batch-select.mjs --json` and record its `batch-selection@1` output in the ship-manifest. It is a deterministic helper rather than a vendored skill - batch membership is arithmetic over frontmatter (machine floor, TASK-IMP-084), and only the HITL verdicts are judgment. Shipping serially while `batch.length > 1` is a violation the artefact makes visible; an absent artefact is itself the violation.
- **Batch selection is a STEP, not a preference (v2.8.0).** This section described batching as the default from v2.5.0 and the outer loop still asked for `next_eligible()` - one task. Nothing computed a batch, so nothing could notice when a batch was skipped, and on 2026-07-17 the workflow shipped TASK-IMP-104 alone while TASK-IMP-106 sat eligible and cone-independent beside it. A default that no step computes is not a default; it is a comment. Step -1 now computes it and records the reasoning.
- **Batch selection.** The eligible set is every `ready_to_implement` task whose `depends_on` rows are all `done`. A batch is a greedy subset of that set whose members are pairwise independent: no `depends_on`/`blocks` edge between any two members, AND no overlap between their declared cones (frontmatter `new_files` + `modified_files` + `service`). tasks whose cones overlap stay serial relative to each other, in Queue-selection priority order.

An EMPTY cone ships ALONE - and "alone" means it SHIPS. Two ways to get one, and the rule covers both: the fields are ABSENT (the author said nothing, so nothing is known), or they are PRESENT and declare `(none)` (the author claims the task touches nothing - which is either not a task or not true, since a task that changes no file does nothing). Neither can be proven independent of a batch member, so neither may JOIN a batch. The refusal names WHICH case it is: telling an author their fields are "absent" when they wrote `(none)` sends them looking for a line they already have. (External review, PR #53.) But when nothing declared is eligible, it BECOMES the batch, by itself, at the head of the priority order. Shipping it alone is exactly what its unknown cone permits: there is no sibling to race.

The first cut of this rule excluded it and stopped there, so a queue whose only eligible task was undeclared produced an EMPTY batch - and the outer loop above reads an empty batch as `break # backlog drained`. The task never shipped, the loop reported success, and the backlog stalled forever while looking finished. The refusal message said "ships alone" one line above the code that made it never ship. (External review, 2026-07-17, on the fix for the previous review.) Undeclared is UNKNOWN, not empty - and unknown cannot be proven independent of anything. Before 2026-07-17 an empty cone conflicted with nothing by construction, so the silent spec joined EVERY batch: TASK-IMP-117 rewrites 501 specs, TASK-TEMPLATE.md and build.sh, declared none of it, and was admitted alongside a task excluded for touching build.sh. Two sub-agents, one file, one parallel round. TASK-IMP-104 taught the near-miss version (declared install.sh, edited two more files inside its service); the fix folded `service` into the cone and assumed a cone was declared at all. Nothing enforced that. The remedy for an exclusion is to declare the cone, not to widen it.

`batch-selection@1` shape: each `excluded[]` entry carries `blocked_by` - the id of the member that blocked it, or NULL when the task excluded itself (an empty cone; nothing blocked it, its own silence did). Every entry carried a real id before 2026-07-17, so a consumer written against the older artefact may assume non-null. Nothing reads it that way today, but the shape is part of a named artefact and therefore a contract. (External review, PR #53.)

Greedy, not maximum: members are admitted in priority order and a task excluded by an earlier admission is never reconsidered, so a strictly larger independent set may exist. That is deliberate - the maximum independent set is NP-hard, and a bigger batch that ignores priority is the wrong batch. This doctrine said "maximal" until 2026-07-17; the word promised a guarantee the loop does not compute, and nothing should be built on it. What batch-select gives you is that the batch is COMPUTED rather than chosen by mood, which is the whole point of §11a.
- **Batched execution.** Phases MAY run batch-wide (map the repo once, implement all members, review all members, test all members) and commits MAY batch per phase across members. What stays strictly per-task: the artefact set (context map, matrix, plan, review packet, coverage gate), the ship-manifest, the BACKLOG/frontmatter status cells, and the recorded HITL verdicts.
- **HITL is unchanged by batching.** Both human-acceptance gates apply to every member individually. A single human reply MAY record verdicts for many members at once (one utterance, N recorded per-task verdicts — e.g. "approve all" / "accept all"); batching reduces round-trips, never guarantees.
- **Unlock rescan.** Whenever any task reaches `done`, re-scan the backlog for tasks whose `depends_on` just became fully satisfied; append the newly-eligible, cone-independent ones to the running batch queue and continue (EXECUTION-DISCIPLINE §1 — no pause to ask). Cone-overlapping unlocks queue serially behind the member they overlap with.
- **Status-page sync rule (group A).** Every backlog-state-update write rides with a regenerated `docs/status/` page in the same commit — enforced mechanically by the pre-commit hook (`.cyberos/lib/status-page.sh` + auto-stage on any `docs/tasks/**`, `CHANGELOG.md`, or `VERSION` change). A status cell that moves without the page moving is a bug.
- **Cones are optimistic; `service` is part of the cone and is not decorative (v2.8.0).** A task's declared `modified_files` is what the author EXPECTED to touch, not what the implementer touched. Live evidence: TASK-IMP-104 declared `install.sh` and ended up editing `version.sh` and `lib/update-check.sh` - both inside its `service` (`tools/install`), neither in its `modified_files`. Had the cone been read as files-only, a sibling `tools/install` task could have been batched alongside it and raced on files nobody declared. Two tasks sharing a `service` therefore OVERLAP and MUST stay serial, even when their declared file lists are disjoint.
- **Swarm execution (v2.6.0; MUST as of v2.8.0).** A batch with more than one member MUST be shipped by a swarm — one sub-agent per member, dispatched in a single parallel round — not by looping the members serially in one agent. Cone-independence is exactly the property that makes this safe: members touch disjoint files, so their edits cannot race. Serialising a batch that was selected for parallelism throws away the only reason to have batched it.
- Dispatch every member of the round in ONE message with N parallel calls. N sequential dispatches is a serial loop with extra steps.
- Each sub-agent owns its member end-to-end and returns the artefact set. The parent owns what stays per-batch: the branch, the commits, the HITL round-trips, the BACKLOG writes.
- Shared-tree gates belong to the PARENT. A sub-agent self-verifies only inside its own cone (its member's test file, lint scoped to its member's files) and MUST NOT run whole-workspace checks — a repo-wide typecheck/build/coverage run while a sibling member is mid-write fails spuriously on the sibling's half-written files. The parent runs the full gate suite exactly once per phase, after every member of the round has landed. (Learned on the 2026-07-16 sachviet consumer-repo run: two cone-disjoint members, per-cone vitest+eslint in the sub-agents, one parent gate run — clean.)
- Cap the round at the point where reviewing N diffs stops being possible. Sub-agents are cheap; a human reading 12 unrelated diffs at one gate is not. When the batch exceeds a reviewable round, ship it as consecutive rounds on the same branch.
- A sub-agent that hits a §2 halt returns it; the parent collects halts and surfaces them together rather than stopping the whole round on the first one.
- Shared files are owned by ONE writer through ONE filesystem view per run — cone-independence includes view-independence (v2.6.3, TASK-IMP-092). Two agents writing one file through different filesystem views produce lost updates even when their writes are time-serialized: each view's reads stay self-consistent while the other view's writes vanish, so every read looks right and the committed truth is wrong. Route every write to a shared file (BACKLOG.md above all) through its one owner on its one view. (Learned 2026-07-16, TASK-IMP-086 incident: the member agent wrote the backlog through one view while the parent's phase flips ran through another — no commit ever carried both.)
- Constrained environments: when the run itself is sandboxed (per-command time caps, background processes dying with the call, synced-mount filesystems), follow the environment runbook — “Running CyberOS under sandboxed agents” in the payload GUIDE (source: `tools/install/docs/index.md`); the rules above stay normative there.
- **One branch per BATCH, not per task (v2.6.0).** The batch is the unit of review and the unit of merge, so it is the unit of branching. Name it `batch/<n>-<short-theme>` (e.g. `batch/3-auth-hardening`). Every member commits to that branch, per phase, with its own conventional commit — per-task commits stay, per-task branches go. Rationale: N branches for N tasks that were selected *because they are independent* produces N PRs that touch disjoint files and merge in any order — pure ceremony. Cone-overlapping tasks were never in the batch to begin with.
- The next batch branches from the previous batch's merge, not from its tip, unless the operator says otherwise. Unlock rescan means batch N+1's eligibility usually depends on batch N being `done`.
- A task routed back to `ready_to_implement` mid-batch leaves the batch; it re-enters selection later and MUST NOT hold its batch's branch open.
- **Mixed agent-human and human-only work: the guideline lives IN the task (v2.6.0, operator request).** When a task needs the operator to do something the agent must not (§2) or genuinely cannot, write the step-by-step guideline INTO that task's `spec.md` under `## Operator steps` — never into a new file. Never create `SETUP.md`, `RUNBOOK-<task>.md`, `INSTRUCTIONS.md`, or any sibling artefact for it.
- Reason: a separate file is a second place the reader must find, and it rots the moment the task moves. The operator opens the task; the steps are there. Anything else asks them to hold two documents in their head and guess which is current.
- Shape: numbered, copy-pasteable, one command or one click per step, with the expected output stated. If a step is GUI-only and the agent has OS/browser control, the agent does it (`EXECUTION-DISCIPLINE.md` §2b) and the guideline records what was driven — it does not ask the operator to repeat it.
- The task halts at that gate only if the steps are genuinely operator-only under §2. "The agent wrote a guideline" is not itself a halt.
## 11b. Route-back ceiling (v2.8.0, TASK-IMP-108)

`routed_back_count` has been written on every route-back since it was defined and read as a limit exactly nowhere - 18 references in this file, all increments. The 5-fail circuit breaker bounds the DEBUGGING cycle inside one testing phase; nothing bounds how many times a task circles the whole loop. A task that keeps failing can cycle implement -> review -> test -> route back forever, burning budget with no escalation. This session hit the adjacent failure twice (API spend limits) and the loop had nothing to say about it.

- **At `routed_back_count >= 3`, ship-tasks MUST HALT** at an operator gate instead of re-entering. Present every route-back reason on record side by side, and ask for one verdict: re-enter / split the task / `on_hold` / `closed`. Re-entering without a recorded verdict is a violation.
- **Three is a judgment, not a derivation**, and the workflow says so rather than implying false precision. The reasoning: "the same task failed three DIFFERENT ways" is evidence about the spec, not the implementation - and a spec problem is not fixed by another implementation pass.
- **The ceiling counts cycles, not causes.** Three route-backs from one flaky test still halt. A human reading three identical reasons decides in seconds; the alternative is a loop that never asks.
- **The halt belongs to the parent** (§11a): a swarm sub-agent MUST NOT resolve it. The verdict is the operator's.
- **Under the ceiling, nothing changes.** A task at `routed_back_count: 2` re-enters normally.
- **`entered_via: spec_rejected` routes to `draft`, not `ready_to_implement`** (STATUS-REFERENCE §1.3). Pairs with this ceiling: three route-backs usually IS a spec problem wearing an implementation problem's clothes.

## 11c. Standing goals: what `done` claimed, re-verified (v2.8.0, TASK-IMP-109)

`done` is terminal and nothing re-checks it. TRACE-004 proves every clause had a passing test ON THE DAY IT SHIPPED; nothing looks again. A task shipped in batch 1 could be broken today and the corpus would still show it green. A goal you verify once is an assumption with a timestamp.

`task-reconcile` does NOT close this. It measures drift when a task RE-ENTERS the workflow - a turnstile, not a sentinel. A `done` task that never comes back is never examined again by anything.

- **At the `done` flip, ship-tasks MUST write `docs/goals/<task-id>.md`** carrying: `predicates` (the task's §1 cited tests - already collected, because TRACE-004 just verified them, so the predicate is free), `born`, `source`, `status: satisfied`, `last_pass`, `on_violation: report`.
- **The predicate set is the CITED TESTS, nothing invented.** A goal MUST NOT claim a check the task never made.
- **ACs whose evidence is a justified `verify:` are NOT enrolled** and the goal names them as not mechanically re-verifiable. A predicate that cannot be re-run is not a predicate.
- **A task reaching `done` with zero runnable predicates STILL gets a goal**, marked `predicate: none` with the reason. The absence is the finding; it must never read as a pass.
- **Re-verification is `node .cyberos/docs-tools/verify-goals.mjs`.** DETECTION ONLY: a violated goal changes no status, writes no code, and re-opens no task. The remedy is a new `type: bug` task through create-tasks -> ship-tasks. The sentinel detects; the pipeline fixes. An auto-fix on a violated acceptance is the machine grading its own homework at the moment nobody is watching.
- **Retirement is a human decision, logged.** A flaky predicate is quarantined (`status: retired`, reason recorded), never deleted - a goal deleted without a reason is the evidence loss this whole mechanism exists to prevent.
- **When it runs is the operator's business.** Scheduling is a host decision; CyberOS is invoked.

## 11d. Batch economics: what the loop cost (v2.8.0, TASK-IMP-114)

`routed_back_count` is the only cycle metric this loop has ever written, and nothing has ever added two of them together. There is no wall-time and no token accounting anywhere, so "was this batch worth running" has never had a number — while five batches whose relative cost nobody can compare went past. This run supplied the missing data the expensive way: two API spend-limit cutoffs, one of them losing three of four swarm agents mid-flight.

- **At the batch close, ship-tasks MUST write `docs/batches/<batch-id>.md`** — an append-only LEDGER of one batch. It carries `batch`, `members`, `started`, `ended`, `route_backs`, `gate_reasks`, and `tokens` when the harness reports them. The status page renders one row per ledger.
- **The ledger records ONLY what nothing else knows.** Tasks shipped is NOT written into it: the page DERIVES it from each member's own frontmatter, because `status` already IS "did this task ship". A number copied into a second place is a number that will eventually disagree with the first, and frontmatter is the record of truth either way.
- **`route_backs` is the batch's own count, NOT a sum of `routed_back_count`.** That counter is a task's LIFETIME total across every batch it ever sat in. The row wants the route-backs that happened in THIS batch — which is where the cost fell — so summing the lifetime counter over the members charges batch 1 for a route-back that happened in batch 2, and charges it retroactively: a closed batch's row would drift upward months later. Two different facts, one word. Record the batch's own; a task that routed back and left (§11a) keeps its route-back on the batch it left, because that batch is what paid for it.
- **Anything the ledger does not record reads `unknown`, never `0`.** A zero asserts a fact nobody measured. This applies to `route_backs` and `gate_reasks` exactly as it applies to `tokens`.
- **This is not a new writer on the phase path.** Steps 1-31 are untouched, nothing new is measured inside a task, and no gate consumes any of this. The batch close transcribes instants the run already had — the same shape as §11c's goal write at the `done` flip.
- **Timestamps belong here and nowhere else.** `started` and `ended` are the two instants that bound the batch; the page computes wall time from them and never from the clock. A ledger is the one artefact where a wall-clock reading is honest, because it records when something happened instead of deriving a fact from now.
- **A batch cut mid-flight has no `ended`.** Leave the key absent and the page renders `incomplete`. It MUST NOT be back-filled with the time somebody noticed, and the page MUST NOT compute a duration to now. An unfinished batch has no wall time; inventing one is the fabrication the whole corpus exists to refuse.
- **Tokens are OPTIONAL and MUST NOT be zeroed.** Omit the key when the harness reports nothing — a `0` asserts a fact nobody measured. Only a COMMITTED figure reaches the rendered row: a harness-reported number that varies per read would break the byte-stability TASK-IMP-082's `fp-` stamp depends on, and a metric that requires one specific host is a metric that expires.
- **A batch of one still gets a ledger.** Small batches are the comparison baseline.
- **Route-backs count in the batch where they happened**, which is where the cost fell — a task routed back leaves its batch (§11a) and its next attempt is the next batch's cost, not this one's.
- **This MEASURES; it MUST NOT gate.** No threshold, no budget, no warning, nothing that turns a row red, and no status changes because of a number here. Measuring is not enforcing. A metric that starts blocking is a gate that arrived without anyone deciding to add one; the operator reads the row and makes the call. Spend limits are a real problem, and the answer to them is not a machine that stops itself for a reason nobody agreed to.

## 11e. Judgment tiering: which steps need judgment (v2.8.0, TASK-IMP-115)

Every step above runs at whatever reasoning the host happens to give it. Nothing marked which steps deserve expensive judgment (step 27's task-audit, step 3's ADR) and which are near-mechanical (the backlog flips — already a script, correctly). So a host had exactly one strategy available: spend the same on all 32. This session supplied the argument the expensive way, with two API spend-limit cutoffs, one of them losing three of four swarm agents mid-flight.

Each `skill_chain` step therefore carries `judgment: high | medium | mechanical`.

- **It is ADVISORY, and that is the whole design.** A host MAY route on it. **Nothing in the payload reads it to decide anything** — no step, no gate, no helper, no condition. It is information, not instruction. A host that ignores the field entirely is correct, and is the default: the chain runs identically with or without it. The field can only be wrong about the work; it can never break the run.
- **NO MODEL STRINGS. EVER.** No model name, no price, no host effort level appears here or anywhere in the payload. Those are the host's facts — accurate the day they are written and wrong soon after — and a `<vendor>-<model>-5` literal in a rule is a rule with an expiry date nobody will notice passing. The payload describes the WORK; the host picks the worker. This is the constraint the field exists to respect, not a caveat on it.
- **`mechanical` = a docs-tools helper produces the step's result.** The agent runs the tool and transcribes; no model is deciding anything. The label is never applied on a feels-deterministic basis — it is anchored to a helper the payload names for that skill:

  | Steps | Skill | Helper that does the work | Where the payload says so |
  |---|---|---|---|
  | 0 | `task-reconcile` | `docs-tools/task-reconcile.mjs` | `modules/skill/task-reconcile/SKILL.md` frontmatter `tool:` |
  | 13, 15, 19, 21, 30 | `backlog-state-update-author` | `docs-tools/backlog-mutate.mjs` | §1 above — "the byte-discipline executor for `backlog-state-update` mutations … never hand-sed" |

- **`high` = the step's output is a judgment the chain then depends on.** Every one carries its reason here, because a level asserted without a reason is a level that was guessed:

  | Step | Skill | Why a model is deciding |
  |---|---|---|
  | 1 | `repo-context-map-author` | the "outside-domain" call is a judgment, and step 3's ADR trigger is derived from it |
  | 3 | `architecture-decision-record-author` | an ADR IS the architectural decision |
  | 5 | `edge-case-matrix-author` | enumerates the boundary and SECURITY cases nobody wrote down |
  | 7 | `mock-contract-test-author` | designs the contract shape of a service that does not exist yet |
  | 9 | `implementation-plan-author` | the implementation itself |
  | 17 | `code-review-author` | produces the packet the human acceptance gate reads |
  | 25 | `debugging-cycle-author` | classifies the failure vector and forms the hypothesis |
  | 27 | `task-audit` | TRACE-004 closure — the judgment half; its `task-lint.mjs` floor only seeds the mechanical findings |

- **Ambiguous is `medium`, never `high`.** Overstating a step's need is how the expensive default returns wearing a label. `medium` is also the honest reading of a step whose work a helper only half-owns: step 23's `coverage-scope.mjs` computes the per-file table but its own header reserves the judgment fields (`tests_failed`, `ecm_rows_uncovered`) for the author skill, so step 23 is `medium`, not `mechanical`.
- **Steps 28-29 are the known rough edge.** Both gates are deterministic — caf-gate's skill says "no LLM" — but their executors (`tools/awh`, `scripts/caf_gate.sh`) are not docs-tools helpers, and TASK-IMP-115's AC 2 scopes `mechanical` to docs-tools backing. They read `medium`: under-informative rather than wrong. Widening the helper family is a follow-up, not a silent reinterpretation here.
- **Where it does NOT extend yet.** `create-tasks` and the `plan` workflow (TASK-IMP-111) carry no `judgment` field. This is one workflow's experiment; extend it once a host has actually routed on it and said the information was worth having.
- **A wrong level is a drift bug, not an outage.** A step marked `mechanical` whose helper is later replaced by a model is simply wrong until someone fixes it — the suite arm (`test_mechanical_steps_are_helper_backed`) is what notices, by re-proving the helper exists and is still the thing the payload names for that skill.

## 12. No partial-ship-and-pause within a task

The workflow MUST drive **all phases of a task to completion in one continuous session** (or route back to `ready_to_implement` cleanly). It runs continuously under the halt-only doctrine in `EXECUTION-DISCIPLINE.md` (`modules/cuo/EXECUTION-DISCIPLINE.md` in the platform repo; vendored beside this file at `.cyberos/cuo/` in installed repos): the agent stops ONLY for an operator-decision fork, a manual/operator-only action (push, deploy, destructive op, secret), a hard blocker past the circuit-breaker budget, or the operator stop signal. Everything else — compile/lint/clippy, a test or module gate the agent's own change broke, the order of slices or tasks — the agent self-resolves and continues.

**Rules:**

1. Read the full gap list + slice plan BEFORE running any step.
2. Don't ask between phases for self-resolvable work — continuation is implied by "drive this task". The two human-acceptance gates (review approval at `reviewing → ready_to_test`, and final acceptance at `testing → done`) are the exception: halt for the recorded human verdict there, since HITL is required.
3. Commit per phase for git-history hygiene; each phase = own conventional commit + verify gate.
4. Do NOT pause between tasks either. The outer loop (§11) advances to the next eligible task on its own; halt between tasks only on an `EXECUTION-DISCIPLINE.md` §2 condition, never just because one task finished.
5. If genuinely blocked mid-task (e.g. needs ADR-class operator decision), DOCUMENT the block in §10.7 of the task's audit.md, route back to `ready_to_implement` with `routed_back_count += 1` and `reason: "<blocker>"`. Do NOT silently ship a partial phase and walk away.

See `task-audit` skill §9.1 for the full clause + grandfathered exceptions.

## Reconcile entry — when a task claims work this workflow did not perform (v2.7.0, TASK-IMP-101)

This workflow trusts two things: its own run manifests (hash-verified — see Resume semantics below) and its own gates (route-back on failure). A status cell is neither. A task can arrive already implemented — mid-shipping from another session, or long "done" — with no manifest, a manifest that no longer verifies, or a phase artefact set that does not exist. Trusting that cell is how a claim outruns its evidence (learned 2026-07-16, the TASK-IMP-086 incident: a task marked done whose deliverable no commit carried).

**Trigger.** Before entering the chain for a task, if its status is past `ready_to_implement` AND (no ship-manifest exists OR `ship-manifest.mjs verify` fails OR the claimed phase's artefact set is missing), run step 0:

```
node .cyberos/docs-tools/task-reconcile.mjs <task-ID> --run-tests
```

A VALID manifest means resume semantics own the task — reconcile does not fire, and the two mechanisms never double-handle the same state.

**The gate.** The report carries exactly one recommendation. Present it — claimed status, the recommendation, the two or three facts driving it, what each branch costs — and take the operator's verdict:

| Verdict | The workflow then |
|---|---|
| `resume_at_phase(N)` | re-enters the chain at step N and continues normally |
| `route_back` | flips to `ready_to_implement`, `routed_back_count += 1` (STATUS-REFERENCE §1.3), records the report's reasons and emits `task_routed_back` |
| `adopt_candidate` | backfills the phase artefact set from the evidence, then re-enters at the verified phase |

**The rule.** The agent NEVER executes a branch — resume, route back, or adopt — without the recorded human verdict. This is a third, CONDITIONAL human gate; the two acceptance gates (reviewing → ready_to_test, testing → done) are untouched and still apply afterwards. An operator verdict that departs from the recommendation is legitimate and emits `memory.status_overridden` with its reason. Skill contract: `modules/skill/task-reconcile/SKILL.md`.

## depends_on evidence gate (v2.7.0, TASK-IMP-101)

Before starting any task, every `depends_on` id whose status is `done` MUST carry evidence: a `coverage-gate` artefact in either artefact home (the task folder or `docs/tasks/.workflow/<task-ID>/`), or a `reconcile-report@1` whose verdict the operator accepted. A dependency that carries neither is a done-by-claim, not a done-by-evidence, and the dependent task is BLOCKED — surfaced at a gate where the operator may override.

Rationale: building on unverified foundations is how one bad claim becomes a subtree of them. The check is cheap and the override is always available — what it removes is the SILENT case, where nobody knew the foundation was unwitnessed.

- Both artefact homes count, so the historical corpus (bundles under `docs/tasks/.workflow/`) does not false-block.
- `depends_on` naming an off-ramped task (`closed`, `duplicate`, `cannot_reproduce`) is an unmet dependency under the existing eligibility rules and is surfaced the same way.
- Every override emits `memory.status_overridden` `{actor, task_id, prior_status, new_status, reason}` — the operator's call, on the record.

## Resume semantics (ship-manifest@1) - added by TASK-CUO-206

Every run maintains a per-task run-state manifest at `docs/tasks/.workflow/<task-ID>.ship.json`, shaped by `modules/skill/contracts/task/SHIP-MANIFEST.md` (ship-manifest@1). The manifest is a CACHE of proven work, never an authority - task frontmatter and BACKLOG.md remain the only record of truth.

**Write points.** The manifest MUST be rewritten after EVERY completed, failed, or conditionally-skipped step - no step's outcome goes unrecorded. Writes are two-phase atomic (`.tmp.<nonce>` then rename), mirroring the memory-protocol discipline. Each step entry records `{index, skill, status, artefact_path, artefact_sha256, verdict, completed_at}`; `task_sha256` (hash of the task spec at run start) and `workflow_version` are pinned at manifest creation.

**Resume.** On invocation for a task whose manifest exists:

1. `workflow_version` mismatch -> needs_human. Never a silent mixed-version run.
2. `task_sha256` mismatch (task spec edited since run start) -> every step is stale; restart at step 1 (history and `routed_back_count` are retained).
3. Otherwise re-hash EVERY recorded `artefact_sha256` against disk. The earliest mismatch marks that step and all later steps stale - redo from there. All intact -> continue at the first non-done step.
4. Echo the resume line before running: `resume <task-ID>: steps 1-N verified (K artefacts, hashes OK), continuing at step M/31 (<skill>). routed_back_count=R`.

**Human gates re-ask.** A manifest can never authorize a gate: resuming at a HITL gate step re-requests the human approval. A recorded `hitl.requested_at` is when the question was asked - it is NOT the answer, and MUST NOT be treated as approval.

**Terminal handling.** When the task reaches `done` (HITL gate 2 passed), delete the manifest. On route-back to `ready_to_implement`, keep it with `routed_back_count += 1` - the next run restarts at step 1 by the staleness rule but retains count and history.

**Queue selection (no task id given).** Among tasks at `ready_to_implement` whose `depends_on` are all `done`: order by priority: `p0` before `p1` before `p2` before `p3` (legacy MoSCoW values map per FM-105), then `created` ascending, then id ascending. Echo the selection before step 1: `queue: picked <id> (priority=<p>, created=<d>) over <n> other eligible tasks`. Reference implementation: `modules/cuo/cuo/ship_manifest.py` (doc-driven agents apply this section directly). The vendored executable of this section is `tools/install/docs-tools/ship-manifest.mjs` (`.cyberos/docs-tools/ship-manifest.mjs` in installed repos) - the doc-driven reference implementation alongside `ship_manifest.py`: `init` pins, `record` hashes artefacts at write time, `verify`/`resume-line` walk the staleness order above with distinct exit codes and echo the mandated resume line; reach for it instead of re-deriving the algorithm (TASK-IMP-085).

## Cross-references

- Execution discipline (continuous run, halt-only conditions): `EXECUTION-DISCIPLINE.md` (`modules/cuo/EXECUTION-DISCIPLINE.md` in the platform repo; vendored beside this file at `.cyberos/cuo/` in installed repos). Added 2026-06-20 (v2.2.0): the agent halts only for an operator-decision fork, a manual/operator-only action, a hard blocker past the budget, or the operator stop signal; it self-resolves everything else and runs continuously across phases and tasks.
- Task lifecycle: `modules/skill/contracts/task/STATUS-REFERENCE.md` (12-state enum, transitions, HITL semantics).
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

The rules this document (and every `modules/skill/` contract) defines are DISTRIBUTED: they ship inside the cyberos payload to standalone/self-hosted `.cyberos/` installs, the Claude plugin, the MCP server, and the npx CLI. Rule changes MUST reach every channel through this auto-hook chain, never by hand-copying:

- **Build hook (local):** `.githooks/pre-commit` rebuilds `dist/cyberos` and runs `check-version-sync.sh` whenever `modules/cuo/**`, `modules/skill/**`, or `tools/install/**` is staged — a rule edit cannot be committed without a fresh, stamp-verified payload build.
- **Push hook (CI):** `payload-gate.yml` re-proves the same invariant on every push/PR touching those paths.
- **Release hook:** `release.yml`'s payload job builds and uploads the stamped payload as release assets on every tag — the pull source for `cyberos update` / `version.sh` / plugin + MCP consumers.
- **Deploy hook:** `deploy.yml`'s docs job also triggers on `modules/cuo/**` and `modules/skill/**`, so the published site reflects rule changes without waiting for a release.
- **Drift signal:** the payload's `manifest.yaml` carries `rules_sha` — a deterministic content fingerprint over the distributed rule trees (`cuo/ plugin/ mcp/ cli/ memory/`). Channels compare it to detect rule drift even when VERSION is unchanged; `check-version-sync.sh` fails any payload missing it. Client-side comparison in `cyberos update`/plugin/MCP is the designated follow-up (TASK-IMP-074 §9).

### Pre-push / pre-install re-verification (v2.6.0, operator request)

Before `git push`, and before installing the payload onto ANY other repo, re-prove the chain end-to-end. The hooks above fire on *staged paths*; they do not prove the built artefact is the one a consumer will actually receive.

Run, in order, and read the output rather than the exit code alone:

1. `bash tools/install/build.sh` — the payload a consumer pulls, rebuilt from source.
2. `bash tools/install/check-version-sync.sh dist/cyberos` — VERSION identical across every stamped artefact.
3. `bash tools/install/check-chain-coverage.sh dist/cyberos` — every vendored skill reachable from the chain.
4. `bash scripts/tests/run_all.sh` — every suite, including the payload suites under `tools/install/tests/`.
5. **Install into a scratch repo and look at the result**, not at the script that produces it:
   ```
   rm -rf /tmp/verify && mkdir -p /tmp/verify && cd /tmp/verify && git init -q .
   bash <repo>/dist/cyberos/install.sh .
   find .cyberos -name 'feature.md'     # non-empty, or task-author HALTs on first task
   ```

**Why the last one is not optional.** On 2026-07-15 `build.sh` shipped no per-type templates while `task-author` dispatched on them; reading `build.sh` did not reveal it, and `find` on a real install did — in one command. Every channel (`.cyberos/` install, Claude plugin, MCP server, npx CLI) receives the payload, not the repo. A rule that is correct in `modules/` and absent from `dist/` is correct nowhere that matters.

The vendored copy under a target's `.cyberos/` is what actually executes. `build.sh` refreshes `dist/`; only `install.sh` lays `dist/` into `.cyberos/`. Skipping the install step means the target keeps running the OLD rules while `dist/` looks current — the exact failure that made a fixed renderer produce stale output for a full session.

### Closing report (v2.6.0, operator request)

End every ship run — batch or single — with a short suggestion of next steps or possible improvements. Not a recap of what was done; the operator watched that. What is now unblocked, what the run exposed that is worth doing next, and what you would do if it were your call. Say when the honest answer is "nothing — merge it".

*End of `chief-technology-officer/ship-tasks.md` workflow.*
