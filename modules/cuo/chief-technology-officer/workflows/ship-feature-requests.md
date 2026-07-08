---
workflow_id: chief-technology-officer/ship-feature-requests
workflow_version: 2.3.0
purpose: Drive each eligible FR in `docs/feature-requests/BACKLOG.md` end-to-end through the full lifecycle — from `ready_to_implement` through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `docs/feature-requests/STATUS-REFERENCE.md` §1.1). Deep-maps the repo, generates the edge-case matrix, implements with 90 % coverage on touched files, injects observability, self-approves architectural deviations via ADRs, runs the multi-vector debugger with a 5-fail circuit breaker, runs the testing gate (`coverage-gate-author`/`-audit`), and physically updates BACKLOG.md status between every phase transition. Failure or blocker at any downstream phase routes the FR back to `ready_to_implement` (STATUS-REFERENCE §1.3) with `routed_back_count += 1`.
persona: chief-technology-officer
cadence: per-FR (loops continuously over BACKLOG.md)
status: shipped   # CUO-workflow lifecycle: planned | shipped | retired (distinct from FR lifecycle in STATUS-REFERENCE.md)
pattern: linear
hitl: required    # human-acceptance verdict mandatory at reviewing->ready_to_test and testing->done (STATUS-REFERENCE §1.4, EXECUTION-DISCIPLINE §2a); the agent never self-sets done
scope: all implementation work - net-new product FRs and improvement/hardening FRs alike; there is no separate improvement track (see section 1a)

inputs:
  - { name: backlog,                source: docs/feature-requests/BACKLOG.md,                                       format: markdown }
  - { name: repo_root,              source: workflow-caller,                                                        format: absolute path }
  - { name: stop_signal,            source: operator (Ctrl-C / workflow-stop event),                                format: bool }

outputs:
  - { name: updated_backlog,           format: markdown (BACKLOG.md with status mutations),         recipient: repo HEAD }
  - { name: implementation_diff,       format: git diff (files added/modified),                    recipient: human-reviewer (commit + push manual) }
  - { name: adr_records,               format: architecture-decision-record@1 (zero or more),      recipient: docs/adrs/ }
  - { name: edge_case_matrix,          format: edge-case-matrix@1 (one per FR),                    recipient: memory audit chain }
  - { name: coverage_report,           format: coverage-gate@1 (one per FR),                       recipient: memory audit chain }
  - { name: debug_trace,               format: debug-trace@1 (one per failed FR attempt),          recipient: memory audit chain }
  - { name: fr_audit_report,           format: feature-request-audit@2.0 (pre-flight, one per FR), recipient: memory audit chain + <FR>.audit.md §10 }
  - { name: coverage_gate_report,      format: coverage-gate-audit@1 (one per FR),                 recipient: memory audit chain + <FR>.audit.md §10.4 }
  - { name: awh_gate_report,           format: awh-eval@1 (one per FR, out-of-band rerun),         recipient: memory audit chain (memory.awh_gate_result) + <FR>.audit.md §10.5 }
  - { name: caf_gate_report,           format: caf-gate@1 (one per FR, code-audit floor),          recipient: memory audit chain (memory.caf_gate_result) + <FR>.audit.md §10.6 }

skill_chain:
  # ── Phase: ready_to_implement → implementing (workflow start) ──
  - { step: 1,  skill: repo-context-map-author,                    inputs_from: { repo_root: repo_root, fr_id: next_fr_id },              outputs_to: context_map_draft,                         phase: "ready_to_implement → implementing" }
  - { step: 2,  skill: repo-context-map-audit,                     inputs_from: context_map_draft,                                        outputs_to: context_map }
  - { step: 3,  skill: architecture-decision-record-author,        inputs_from: { context_map: context_map, fr: next_fr },                outputs_to: adr_draft,                                 condition: 'context_map.files_outside_immediate_domain > 3' }
  - { step: 4,  skill: architecture-decision-record-audit,         inputs_from: adr_draft,                                                outputs_to: adr,                                       condition: "step 3 ran" }
  - { step: 5,  skill: edge-case-matrix-author,                    inputs_from: { fr: next_fr, context_map: context_map },                outputs_to: edge_case_matrix_draft }
  - { step: 6,  skill: edge-case-matrix-audit,                     inputs_from: edge_case_matrix_draft,                                   outputs_to: edge_case_matrix }
  - { step: 7,  skill: mock-contract-test-author,                  inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix },      outputs_to: mock_contracts_draft,                      condition: "fr.has_external_dependency" }
  - { step: 8,  skill: mock-contract-test-audit,                   inputs_from: mock_contracts_draft,                                     outputs_to: mock_contracts,                            condition: "step 7 ran" }
  - { step: 9,  skill: implementation-plan-author,                 inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix, adr: adr },  outputs_to: impl_plan_draft }
  - { step: 10, skill: implementation-plan-audit,                  inputs_from: impl_plan_draft,                                          outputs_to: impl_plan }
  - { step: 11, skill: observability-injection-author,             inputs_from: { fr: next_fr, impl_plan: impl_plan },                    outputs_to: obs_injection_plan }
  - { step: 12, skill: observability-injection-audit,              inputs_from: obs_injection_plan,                                       outputs_to: obs_injection }
  # ── Phase transition: implementing → ready_to_review ──
  - { step: 13, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, transition: "implementing → ready_to_review", outcome: steps_1_to_12 }, outputs_to: backlog_mutation_phase_1, phase: "implementing → ready_to_review" }
  - { step: 14, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_1,                                 outputs_to: backlog_after_phase_1 }
  # ── Phase: ready_to_review → reviewing → ready_to_test ──
  - { step: 15, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, transition: "ready_to_review → reviewing", outcome: reviewer_claim }, outputs_to: backlog_mutation_phase_2 }
  - { step: 16, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_2,                                 outputs_to: backlog_after_phase_2 }
  - { step: 17, skill: code-review-author,                         inputs_from: { fr: next_fr, impl_diff: implementation_diff, adr: adr, edge_case_matrix: edge_case_matrix }, outputs_to: code_review_draft }
  - { step: 18, skill: code-review-audit,                          inputs_from: code_review_draft,                                        outputs_to: code_review_report }
  - { step: 19, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, transition: "reviewing → ready_to_test", outcome: code_review_report }, outputs_to: backlog_mutation_phase_3 }
  - { step: 20, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_3,                                 outputs_to: backlog_after_phase_3 }
  # ── Phase: ready_to_test → testing → done ──
  - { step: 21, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, transition: "ready_to_test → testing", outcome: tester_claim }, outputs_to: backlog_mutation_phase_4 }
  - { step: 22, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_4,                                 outputs_to: backlog_after_phase_4 }
  - { step: 23, skill: coverage-gate-author,                       inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix },      outputs_to: coverage_gate_draft }
  - { step: 24, skill: coverage-gate-audit,                        inputs_from: coverage_gate_draft,                                      outputs_to: coverage_gate_report }
  - { step: 25, skill: debugging-cycle-author,                     inputs_from: { fr: next_fr, coverage_report: coverage_gate_report },   outputs_to: debug_cycle_draft,                         condition: "coverage_gate_report.tests_failed > 0" }
  - { step: 26, skill: debugging-cycle-audit,                      inputs_from: debug_cycle_draft,                                        outputs_to: debug_trace,                               condition: "step 25 ran" }
  - { step: 27, skill: feature-request-audit,                      inputs_from: { fr: next_fr, coverage_report: coverage_gate_report },   outputs_to: fr_audit_report,                           description: "Post-implementation TRACE-004 closure — every §1 clause's cited test MUST be passed in coverage_gate_report. Pre-flight spec audit (`draft → ready_to_implement` transition) ran earlier, BEFORE this workflow; this is the closure check just before marking the FR done." }
  - { step: 28, skill: awh-gate,                                   inputs_from: { fr: next_fr, module: next_fr.module, goldenset: "modules/<module>/.awh/goldenset.yaml", baseline: "modules/<module>/.awh/eval-baseline.json" }, outputs_to: awh_gate_report, description: "Out-of-band independent rerun (the check step 27 is NOT). `awh eval <goldenset> --base-dir . --seeds 1 --baseline <baseline> --max-regression 0.0` reruns the FR's §1 cited tests plus the module suite against the sealed, read-only baseline. GREEN (no task regressed) is REQUIRED to reach the done-flip; RED routes the FR back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Tests sealed via `awh lock modules/<module>/tests`. Emits memory.awh_gate_result." }
  - { step: 29, skill: caf-gate,                                 inputs_from: { fr: next_fr, module: next_fr.module, audit_profile: "modules/<module>/audit-profile.yaml", audit_baseline: "modules/<module>/.caf/" }, outputs_to: caf_gate_report, description: "Code-audit gate (absorbed from CyberSkill/code-audit-framework). Deterministic floor, no LLM: `bash scripts/caf_gate.sh <module>` runs the module's TARGET HEALTH via tools/caf/core/evals/verify-target.sh (the module's own RUN_COMMANDS - build/lint/typecheck/test - from modules/<module>/audit-profile.yaml, fail-closed) AND, when a sealed audit exists at modules/<module>/.caf/, `code-audit-validate --run modules/<module>/.caf --fail-on High` (no new High/Critical finding vs the sealed baseline). CLEAN is REQUIRED alongside the awh gate to reach the done-flip; RED routes the FR back to ready_to_implement per STATUS-REFERENCE §1.3 with routed_back_count += 1. Catches the class awh cannot: build/lint breaks, route 404s, changed data contracts (the CCAF/kymondongiap class). Emits memory.caf_gate_result. See docs/verification/caf-absorption-design.md." }
  - { step: 30, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, transition: "testing → done", outcome: { fr_audit_report: fr_audit_report, awh_gate_report: awh_gate_report, caf_gate_report: caf_gate_report } }, outputs_to: backlog_mutation_phase_5, condition: "awh_gate_report.outcome == GREEN AND caf_gate_report.outcome == CLEAN" }
  - { step: 31, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_phase_5,                                 outputs_to: updated_backlog }

escalates_to:
  - { persona: chief-information-security-officer,                 when: "step 6 edge-case-matrix flags a SECURITY-class entry above warning + no corresponding ADR exists yet" }
  - { persona: chief-product-officer,                              when: "the FR's acceptance criteria are ambiguous — step 5 cannot enumerate the boundary cases without product input" }
  - { persona: chief-financial-officer,                            when: "step 10 implementation-plan-audit total_estimate_pts > 25 % of the target-quarter capacity, OR cumulative session cost > $500 in compute" }

consults:
  - { persona: chief-privacy-officer,                              when: "the FR touches personal data — verify GDPR / Vietnam Decree 13/2023 coverage in the edge-case matrix" }
  - { persona: chief-ai-officer,                                   when: "the FR is AI-driven — verify EU AI Act risk-class + AI-specific test cases in the edge-case matrix" }

audit_hooks:
  - each skill emits one artefact_write row to the memory audit chain per its frontmatter audit.row_kind
  - between every phase transition (steps 13-14, 15-16, 19-20, 21-22, 30-31) backlog-state-update emits a `workflow_phase_complete` memory row
  - on successful `testing → done` transition (step 30) backlog-state-update emits a `workflow_complete` memory row with the full artefact summary
  - on circuit-breaker trip or any in-cycle failure → status reverts to `ready_to_implement` and the writer emits `fr_routed_back` with the rework reason
  - HITL pauses (typically at step 4 ADR-self-approval boundary, step 24 coverage < 90 %, step 26 5-fail circuit-breaker trip) halt the chain

circuit_breaker:
  consecutive_test_failures_per_fr: 5
  on_trip:
    - revert files to pre-execution state (`git restore` on touched paths)
    - mark FR `ready_to_implement` in BACKLOG.md (with `routed_back_count += 1`) via step 30's rework branch
    - emit a `fr_routed_back` memory audit row with the last debug_trace + reason `"circuit_breaker_5_consecutive_test_failures"`
    - proceed to the next eligible FR (do NOT halt the outer loop)
---
# Ship Feature Requests — `chief-technology-officer/ship-feature-requests`

The canonical CTO workflow for **shipping** each `BACKLOG.md` FR end-to-end through the full lifecycle. Renamed from `implement-backlog-frs` (v1.x) in v2.0.0 because the workflow doesn't just implement — it drives the FR through `implementing → ready_to_review → reviewing → ready_to_test → testing → done` (per `docs/feature-requests/STATUS-REFERENCE.md` §1.1). The old name suggested the workflow stopped at code-write; the new name reflects that it covers the full ship.

### One workflow, improvement folds in here

This is the single implementation workflow. There is no separate improvement track any more. Enterprise-hardening and refactoring work (formerly driven by the retired `run-improvement-program` and the `docs/improvement/` backlogs) are feature-requests too: an improvement item is an FR carrying `class: improvement`, and it runs this exact lifecycle with the same mandatory human-acceptance gates. Section 1a covers how improvement FRs are declared, where they live under `docs/feature-requests/`, and how their gate suite is derived. The retired `run-improvement-program.md` points here; the two `cyberos-improve-*` skills that drove the old separate loop have been removed.

## 1. The state engine

`docs/feature-requests/BACKLOG.md` is the **absolute** source of truth for FR state. The state engine reads BACKLOG.md before each iteration:

- **Eligible FR** = first row whose status is `ready_to_implement` AND whose declared `depends_on` rows are all in `done` status.
- **Skipped statuses**: `draft` (not yet audited — handled by the `draft → ready_to_implement` chain, not this workflow), `implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing` (in-flight under another invocation — possibly the previous session of this workflow; pick those up by re-entering at the matching phase), `done` (terminal success — no work to do), `on_hold` / `closed` (operator-decided off-ramps).
- Pick the first eligible FR. Run all 30 steps end-to-end. Between every phase transition the workflow physically updates the BACKLOG.md status cell via `backlog-state-update-author/-audit`. The mutation is atomic — same write that emits the `workflow_phase_complete` (or `workflow_complete` for the final transition) memory row.

### HITL — human-in-the-loop is REQUIRED

Human acceptance is mandatory (STATUS-REFERENCE.md §1.4, EXECUTION-DISCIPLINE.md §2a). The workflow drives the machine-verifiable transitions automatically, but two transitions are human-acceptance gates the agent MUST NOT cross by itself:

- **Review acceptance** (`reviewing → ready_to_test`, steps 19-20): the agent produces the code-review packet (steps 17-18) with every §1 clause mapped to a named test, then HALTS. A human records the approval verdict, which advances the cell.
- **Final acceptance** (`testing → done`, steps 30-31): the agent brings every machine gate green (coverage, TRACE-004, awh, caf), then HALTS. A human records the acceptance verdict. The agent NEVER self-sets `done`.

Between the gates the agent runs continuously and self-resolves everything it can verify (compile, lint, a test it broke, a red module gate on its own change); it does not pause for self-resolvable work. The only mandatory stops inside an FR are these two verdicts.

An operator keeps the superset power to override any cell to any other cell at any time. Common operations:

- **Re-audit a shipped FR** (replaces the v1.2.0 `mode: re_audit`): flip `done → ready_to_review`; on next invocation this workflow picks up at the `reviewing` phase and re-runs steps 15-30.
- **Skip review** for a trivial FR: flip `ready_to_review → ready_to_test` directly (an explicit, recorded override).
- **Park an in-flight FR**: flip `implementing → on_hold`; this workflow skips it on the next iteration.

Every human verdict or override emits one `memory.status_overridden` aux row capturing `{actor, fr_id, prior_status, new_status, reason}`. This workflow detects the persisted state on resume by comparing it against the previous step's expected outcome.

### Failure / blocker semantics — route back to `ready_to_implement`

Any failure in `implementing` (steps 1-12), `reviewing` (steps 17-18), or `testing` (steps 23-28) routes the FR back to `ready_to_implement` with `routed_back_count += 1`. The reason is recorded in:

1. A `memory.fr_routed_back` aux audit row with the failure context (debug_trace, failing-test-name, or blocker reason).
2. A comment cell on the BACKLOG row (`<!-- routed back: <reason> -->`).
3. A future **Issue Request** artefact (TBD — see STATUS-REFERENCE §1.3) that will auto-spawn from the rework signal.

There are NO terminal failure statuses any more. The previous `[FAILED: UNRESOLVABLE ERROR]` and `[BLOCKED: ...]` enums are gone — failures are routing decisions, not states. Operator can still send a doomed FR to `closed` manually via HITL.

## 1a. Improvement FRs (the folded-in hardening track)

Enterprise-hardening, refactoring, and audit-remediation work is not a separate track. Each such item is an FR that runs this same lifecycle, with the same mandatory human-acceptance gates. It carries `class: improvement` in its frontmatter (a net-new feature carries `class: product`, the default). The class does not change the lifecycle; it records intent and selects the gate profile.

Where improvement FRs live:

- Module-scoped hardening (touches one module, e.g. memory) is an `FR-<MODULE>-*` entry under `docs/feature-requests/<module>/`, exactly like a product FR for that module.
- Cross-cutting hardening (spans modules, e.g. a repo-wide audit remediation) lives under `docs/feature-requests/improvement/` with its own README index. That README lists the current programs and tracks the migration of the retired `docs/improvement/` backlogs (`MEM-*`, `T-*`, `IMP-*`) into FR ids.

Gate profile by class:

- The gate suite for any FR is derived from the touched module's `audit-profile.yaml` (the RUN_COMMANDS caf runs as target health) plus the coverage, TRACE, and edge-case gates that apply to every FR.
- The awh out-of-band rerun (step 28) applies when the touched module has a sealed goldenset at `modules/<module>/.awh/`. An improvement FR that touches a module without a goldenset declares awh N/A in its §1 and relies on coverage + caf + the review gate; it does not fabricate an awh pass. Standing up the goldenset can itself be an improvement FR.
- No FR, product or improvement, may weaken a protected invariant (auth model, tenant RLS, hash-chained audit, consent-gated capture, gateway-only model calls) to make a gate green. That is an operator-decision fork: park it and record why (EXECUTION-DISCIPLINE §2).

Everything else (selection from BACKLOG.md, one FR with a commit per phase, the two human gates, route-back on failure) is identical to a product FR.

## 2. Deep context mapping (steps 1-2)

Before any code is generated, the `repo-context-map` skill scans the repo for existing patterns for dependency injection, state management, error handling; database schemas + type interfaces in the FR's declared module; files outside the FR's immediate domain that the implementation would touch.

If more than three "outside-domain" files are flagged, the workflow auto-triggers an ADR (steps 3-4) using the existing `architecture-decision-record-author` + `-audit` pair. The ADR audit must pass at 10/10 against `adr-rubric@1.0` before the chain proceeds.

> **Spec audit was already done.** v2.0.0 drops the pre-flight `feature-request-audit` at step 3 (which was v1.1.0's safety net). The reason: spec correctness is the responsibility of the `draft → ready_to_implement` chain. By the time this workflow picks up an FR in `ready_to_implement`, the spec has already passed `audit_rubric@2.0` at 10/10. If the spec drifted afterwards (e.g. an AGENTS.md amendment broke a TRACE-001 citation), the operator either re-audits the spec via HITL (flip status back to `draft` so the spec chain re-runs) or runs `feature-request-audit` standalone. The post-impl audit at step 27 still enforces TRACE-004 (every clause's cited test is `passed`).

## 3. Edge-case matrix (steps 5-6)

The `edge-case-matrix` skill generates a structured matrix covering: null/empty inputs; extreme bounds (off-by-one, integer overflow, time-zone DST, leap second); malformed payloads (truncated, oversized, non-UTF8); concurrent race conditions (double-submit, double-acknowledge, cross-tenant); security-class entries (auth bypass, RLS escape, injection).

The audit enforces the matrix is not vacuous — every category has ≥1 entry — and that SECURITY-class entries are paired to either an existing test or an ADR.

## 4. Mocks + contract tests (steps 7-8)

If `fr.has_external_dependency = true` (CAPTCHA / 2FA / paywall / missing API keys / future service), `mock-contract-test-author` defines the **exact** expected Request/Response shape of the missing service plus a Mock Service that **passes** the contract test. The FR's frontmatter gets `implementation_kind: mocked` (per STATUS-REFERENCE §3) so the mocked-against-real distinction is preserved without polluting the lifecycle status.

The contract test stays in the suite forever — when the real dependency lands, swapping the mock out is a single import change and the contract guarantees behavioural parity.

## 5. Implementation (steps 9-10)

The `implementation-plan-author` + `-audit` pair drives the actual code. Inputs are the FR, the edge-case matrix, and the (optional) ADR. The audit enforces: (a) every edge-case row is addressed in the plan, (b) the plan respects the existing patterns identified in step 1, (c) capacity estimate is reasonable.

## 6. Observability injection (steps 11-12)

`observability-injection-author` walks the critical paths of the new code and emits: structured-log lines at every state transition (incl. `tenant_id`, `subject_id` when present); trace spans wrapping every external IO; counter increments for every error branch.

The audit checks coverage: ≥80 % of branches have a log/metric/trace point.

## 7. Phase transition: `implementing → ready_to_review` (steps 13-14)

`backlog-state-update-author/-audit` flips the BACKLOG status cell from `implementing` to `ready_to_review` and emits a `workflow_phase_complete` memory row carrying the artefact bundle (context_map, adr?, edge_case_matrix, mock_contracts?, impl_plan, obs_injection).

## 8. Review (steps 15-20)

After the implementing artefacts are settled, status flips to `reviewing` (steps 15-16) and `code-review-author` reads the diff against the §1 clauses and the edge-case matrix, flagging gaps and naming the test cases that would prove each clause. The audit confirms every §1 clause has a named test reference and every edge-case-matrix row has either a test or an ADR justification.

Review acceptance is a mandatory human gate (HITL, see the state engine). The agent presents the review packet with every §1 clause mapped to a named test, then halts; on a recorded human approval verdict, status flips to `ready_to_test` (steps 19-20). On rejection (review uncovers a missing clause or an unaddressed edge case) the FR routes back to `ready_to_implement` (see §1 failure semantics).

> v1.x note — v2.0.0 introduces `code-review-author` and `code-review-audit` as new skills covering the explicit `reviewing` phase. Before v2.0.0 the review work was implicit in the post-impl `feature-request-audit` call; v2.0.0 separates them so the reviewer phase has its own audit row + handoff point. If those skill files don't exist yet in `modules/skill/`, they need to be authored before this workflow can run end-to-end — see the BACKLOG for `FR-SKILL-code-review-author` and `-audit` placeholders.

## 9. Testing phase: coverage gate + post-impl FR audit + awh + caf gates (steps 21-29)

Status flips to `testing` (steps 21-22). `coverage-gate-author` runs the test suite, computes coverage on touched files, and fails the gate if per-file coverage on files touched in this FR is < 90 %. The audit emits the raw terminal output of the coverage tool as the artefact.

If any test fails, `debugging-cycle-author` runs the multi-vector pass (classify failure vector — state/network/memory/logic/flake; output hypothesis + targeted change; re-run; after 5 consecutive failures revert + trip circuit breaker). The audit emits the full hypothesis-and-attempt log.

After coverage + debugging settle, `feature-request-audit` runs the post-impl pass at step 27 to enforce **TRACE-004** — every §1 clause's cited test MUST appear as `passed` in `coverage_gate_report`. A §1 clause may have an AC and a named test from the pre-flight pass, but if the actual test is failing or absent from the coverage report, the FR cannot ship `done`.

## 10. Phase transition: `testing → done` (steps 30-31)

The final phase transition. Outcomes derived by steps 27-29 (post-impl audit + the awh out-of-band test-rerun gate + the caf code-audit gate). Both gates must pass: awh proves the tests still pass; caf proves the module's own build/lint/typecheck/test still run and the audit finds no new High/Critical issue. They are complementary - awh catches test regressions, caf catches the class awh cannot see (a build/lint break, a route that 404s, a changed data contract). Green gates are necessary but not sufficient: this transition is a human-acceptance gate, so the agent halts once the gates are green and a human records the acceptance verdict that sets `done`. The agent never sets `done` itself (HITL required, STATUS-REFERENCE §1.4).

| Step 27 audit + step 28 awh gate + step 29 caf gate + circuit breaker status                                                                                                                          | New status                      | Mutation                                                                                                               |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| All TRACE-001..005 passing + 0 failed tests + awh gate GREEN (independent rerun, no task regressed vs the sealed baseline) + caf gate CLEAN (target health PASS + no new High/Critical audit finding) + recorded human acceptance verdict | `done`                        | `workflow_complete` memory row (written when the human records acceptance), BACKLOG cell `testing → done`         |
| TRACE-004 fails (test exists per spec but isn't passing)                                                                                                                                              | `ready_to_implement` (rework) | `fr_routed_back` memory row with `reason: "trace-004: <test_name> not in coverage_gate_report"`                    |
| awh gate RED (a task regressed vs the sealed baseline, or the FR's cited test is not passing on independent rerun)                                                                                    | `ready_to_implement` (rework) | `fr_routed_back` + `memory.awh_gate_result{outcome: RED}`, `reason: "awh-gate: <task> regressed"`                |
| caf gate RED (target health failed - a RUN_COMMAND broke - or the audit raised a new High/Critical finding)                                                                                           | `ready_to_implement` (rework) | `fr_routed_back` + `memory.caf_gate_result{outcome: RED}`, `reason: "caf-gate: <target-health-fail or finding>"` |
| Circuit breaker tripped during steps 25-26                                                                                                                                                            | `ready_to_implement` (rework) | `fr_routed_back` memory row with `reason: "circuit_breaker_5_consecutive_test_failures"`                           |

The top row's `done` is not written by the agent: when the gates are green it halts at the acceptance gate, a human records the acceptance verdict, and that verdict writes `done` (HITL required, STATUS-REFERENCE §1.4).

The workflow commits the diff to the working tree (operator runs `git add . && git commit && git push` to publish).

## 11. Outer loop

The CUO supervisor invokes this workflow in a loop:

```
while ! stop_signal:
    next_fr = read_backlog().next_eligible()
    if next_fr is None: break        # backlog drained
    invoke_workflow("chief-technology-officer/ship-feature-requests", { repo_root, next_fr })
```

The supervisor handles persistence (state survives across sessions because the truth is in BACKLOG.md + the memory chain), parallelism (multiple FRs may run in parallel when their dependency cones don't overlap), and observability (the per-phase `workflow_phase_complete` + the final `workflow_complete` rows are enough to reconstruct the run).

## 12. No partial-ship-and-pause within an FR

The workflow MUST drive **all phases of an FR to completion in one continuous session** (or route back to `ready_to_implement` cleanly). It runs continuously under the halt-only doctrine in [`../../EXECUTION-DISCIPLINE.md`](../../EXECUTION-DISCIPLINE.md): the agent stops ONLY for an operator-decision fork, a manual/operator-only action (push, deploy, destructive op, secret), a hard blocker past the circuit-breaker budget, or the operator stop signal. Everything else — compile/lint/clippy, a test or module gate the agent's own change broke, the order of slices or FRs — the agent self-resolves and continues.

**Rules:**

1. Read the full gap list + slice plan BEFORE running any step.
2. Don't ask between phases for self-resolvable work — continuation is implied by "drive this FR". The two human-acceptance gates (review approval at `reviewing → ready_to_test`, and final acceptance at `testing → done`) are the exception: halt for the recorded human verdict there, since HITL is required.
3. Commit per phase for git-history hygiene; each phase = own conventional commit + verify gate.
4. Do NOT pause between FRs either. The outer loop (§11) advances to the next eligible FR on its own; halt between FRs only on an `EXECUTION-DISCIPLINE.md` §2 condition, never just because one FR finished.
5. If genuinely blocked mid-FR (e.g. needs ADR-class operator decision), DOCUMENT the block in §10.7 of the .audit.md, route back to `ready_to_implement` with `routed_back_count += 1` and `reason: "<blocker>"`. Do NOT silently ship a partial phase and walk away.

See `feature-request-audit` skill §9.1 for the full clause + grandfathered exceptions.

## Cross-references

- Execution discipline (continuous run, halt-only conditions): [`../../EXECUTION-DISCIPLINE.md`](../../EXECUTION-DISCIPLINE.md). Added 2026-06-20 (v2.2.0): the agent halts only for an operator-decision fork, a manual/operator-only action, a hard blocker past the budget, or the operator stop signal; it self-resolves everything else and runs continuously across phases and FRs.
- FR lifecycle: `docs/feature-requests/STATUS-REFERENCE.md` (10-state enum, transitions, HITL semantics).
- Original prompt source: operator's "Zero-Touch Principal Engineer (Unattended Execution)" — absorbed 2026-05-18.
- BACKLOG state engine: `docs/feature-requests/BACKLOG.md`.
- Companion workflow: `chief-technology-officer/architect-new-system` — produces the FRs this workflow consumes.
- No-partial-ship rule: `feature-request-audit` skill §9.1.
- Pre-flight spec audit (separate chain): `feature-request-audit` skill — drives `draft → ready_to_implement`.
- Test coverage audit: `coverage-gate-audit` skill — drives `testing → done`.
- Out-of-band gate (step 28): `awh eval` against `modules/<module>/.awh/goldenset.yaml` seals `testing → done`. An FR cannot reach `done` unless awh independently re-runs its §1 cited tests + module suite GREEN against the sealed, read-only baseline. See the awh absorption design.
- Code-audit gate (step 29): `bash scripts/caf_gate.sh <module>` (absorbed from CyberSkill/code-audit-framework, vendored at `tools/caf/`). Deterministic floor - target health (`tools/caf/core/evals/verify-target.sh` runs the module's own RUN_COMMANDS from `modules/<module>/audit-profile.yaml`) plus, when present, `code-audit-validate` against the sealed audit at `modules/<module>/.caf/`. CLEAN is required alongside the awh gate. See `docs/verification/caf-absorption-design.md`.

*End of `chief-technology-officer/ship-feature-requests.md` workflow.*
