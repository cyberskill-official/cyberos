---
workflow_id: chief-technology-officer/implement-backlog-frs
workflow_version: 1.0.0
purpose: Zero-touch end-to-end execution of the `docs/feature-requests/BACKLOG.md` queue. Reads the next eligible FR, deep-maps the repo, generates an edge-case matrix, implements, hits 90 % coverage on touched files, injects observability, self-approves architectural deviations via ADRs, runs the multi-vector debugger with a 5-fail circuit breaker, and physically updates BACKLOG.md state. Absorbs the "Zero-Touch Principal Engineer" prompt into the CTO catalogue so the project's own skills implement the project.
persona: cuo/chief-technology-officer
cadence: per-FR (loops continuously over BACKLOG.md)
status: shipped
pattern: per-instance

inputs:
  - { name: backlog,                source: docs/feature-requests/BACKLOG.md,                                       format: markdown }
  - { name: repo_root,              source: workflow-caller,                                                        format: absolute path }
  - { name: stop_signal,            source: operator (Ctrl-C / workflow-stop event),                                format: bool }

outputs:
  - { name: updated_backlog,        format: markdown (BACKLOG.md with status mutations),    recipient: repo HEAD }
  - { name: implementation_diff,    format: git diff (files added/modified),               recipient: human-reviewer (commit + push manual) }
  - { name: adr_records,            format: architecture-decision-record@1 (zero or more), recipient: docs/adrs/ }
  - { name: edge_case_matrix,       format: edge-case-matrix@1 (one per FR),               recipient: BRAIN audit chain }
  - { name: coverage_report,        format: coverage-gate@1 (one per FR),                  recipient: BRAIN audit chain }
  - { name: debug_trace,            format: debug-trace@1 (one per failed FR attempt),     recipient: BRAIN audit chain }

skill_chain:
  - { step: 1,  skill: repo-context-map-author,                    inputs_from: { repo_root: repo_root, fr_id: next_fr_id },              outputs_to: context_map_draft,        planned: true }
  - { step: 2,  skill: repo-context-map-audit,                     inputs_from: context_map_draft,                                        outputs_to: context_map,              planned: true }
  - { step: 3,  skill: architecture-decision-record-author,        inputs_from: { context_map: context_map, fr: next_fr },                outputs_to: adr_draft,                                 condition: "context_map.files_outside_immediate_domain > 3" }
  - { step: 4,  skill: architecture-decision-record-audit,         inputs_from: adr_draft,                                                outputs_to: adr,                                       condition: "step 3 ran" }
  - { step: 5,  skill: edge-case-matrix-author,                    inputs_from: { fr: next_fr, context_map: context_map },                outputs_to: edge_case_matrix_draft }
  - { step: 6,  skill: edge-case-matrix-audit,                     inputs_from: edge_case_matrix_draft,                                   outputs_to: edge_case_matrix }
  - { step: 7,  skill: mock-contract-test-author,                  inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix },      outputs_to: mock_contracts_draft,     planned: true,                  condition: "fr.has_external_dependency" }
  - { step: 8,  skill: mock-contract-test-audit,                   inputs_from: mock_contracts_draft,                                     outputs_to: mock_contracts,           planned: true,                  condition: "step 7 ran" }
  - { step: 9,  skill: implementation-plan-author,                 inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix, adr: adr },  outputs_to: impl_plan_draft }
  - { step: 10, skill: implementation-plan-audit,                  inputs_from: impl_plan_draft,                                          outputs_to: impl_plan }
  - { step: 11, skill: observability-injection-author,             inputs_from: { fr: next_fr, impl_plan: impl_plan },                    outputs_to: obs_injection_plan,       planned: true }
  - { step: 12, skill: observability-injection-audit,              inputs_from: obs_injection_plan,                                       outputs_to: obs_injection,            planned: true }
  - { step: 13, skill: coverage-gate-author,                       inputs_from: { fr: next_fr, edge_case_matrix: edge_case_matrix },      outputs_to: coverage_gate_draft }
  - { step: 14, skill: coverage-gate-audit,                        inputs_from: coverage_gate_draft,                                      outputs_to: coverage_report }
  - { step: 15, skill: debugging-cycle-author,                     inputs_from: { fr: next_fr, coverage_report: coverage_report },        outputs_to: debug_cycle_draft,        planned: true,                  condition: "coverage_report.tests_failed > 0" }
  - { step: 16, skill: debugging-cycle-audit,                      inputs_from: debug_cycle_draft,                                        outputs_to: debug_trace,              planned: true,                  condition: "step 15 ran" }
  - { step: 17, skill: backlog-state-update-author,                inputs_from: { fr: next_fr, outcome: derived_from_steps_1_to_16 },     outputs_to: backlog_mutation_draft,   planned: true }
  - { step: 18, skill: backlog-state-update-audit,                 inputs_from: backlog_mutation_draft,                                   outputs_to: updated_backlog,          planned: true }

escalates_to:
  - { persona: cuo/chief-information-security-officer,             when: "step 5 edge-case-matrix flags a SECURITY-class entry above warning + no corresponding ADR exists yet" }
  - { persona: cuo/chief-product-officer,                          when: "the FR's acceptance criteria are ambiguous — step 5 cannot enumerate the boundary cases without product input" }
  - { persona: cuo/chief-financial-officer,                        when: "step 9 implementation-plan-audit total_estimate_pts > 25 % of the target-quarter capacity, OR cumulative session cost > $500 in compute" }

consults:
  - { persona: cuo/chief-privacy-officer,                          when: "the FR touches personal data — verify GDPR / Vietnam Decree 13/2023 coverage in the edge-case matrix" }
  - { persona: cuo/chief-ai-officer,                               when: "the FR is AI-driven — verify EU AI Act risk-class + AI-specific test cases in the edge-case matrix" }

audit_hooks:
  - each skill emits one artefact_write row to the BRAIN audit chain per its frontmatter audit.row_kind
  - workflow emits a single workflow_complete row per FR with the 5-artefact summary (context_map / adr? / edge_case_matrix / coverage_report / debug_trace?) + per-artefact hash + the BACKLOG.md status transition
  - HITL pauses (typically at step 3 ADR-self-approval boundary, step 13 coverage < 90 %, step 15 5-fail circuit-breaker trip) halt the chain

circuit_breaker:
  consecutive_test_failures_per_fr: 5
  on_trip:
    - revert files to pre-execution state (`git restore` on touched paths)
    - mark FR `[FAILED: UNRESOLVABLE ERROR]` in BACKLOG.md (via step 17)
    - emit a `workflow_failed` BRAIN audit row with the last debug_trace
    - proceed to the next eligible FR (do NOT halt the outer loop)
---

# Implement BACKLOG FRs — `chief-technology-officer/implement-backlog-frs`

The canonical CTO workflow for **executing** the `BACKLOG.md` queue with zero human intervention. It absorbs the "Zero-Touch Principal Engineer" prompt directly into the CUO catalogue — every section of that prompt maps onto a CTO skill invocation, every decision gate becomes an audit-row emission, and the outer loop is the supervisor's responsibility.

## 1. The state engine

`docs/feature-requests/BACKLOG.md` is the **absolute** state manager. The supervisor reads it before each iteration:

- Skip every FR whose status is `shipped + strict-audited`, `[FAILED]`, or `[BLOCKED]`.
- Pick the first FR whose declared dependencies are all `shipped`.
- After the chain completes (success or failure), physically overwrite the BACKLOG.md status cell via the `backlog-state-update` skill (step 17/18). The mutation is atomic — same write that emits the BRAIN row.

## 2. Deep context mapping (steps 1-2)

Before any code is generated, the `repo-context-map` skill scans the repo for:

- Existing patterns for dependency injection, state management, error handling.
- Database schemas + type interfaces in the FR's declared module.
- Files outside the FR's immediate domain that the implementation would touch.

If more than three "outside-domain" files are flagged, the workflow auto-triggers an ADR (steps 3-4) using the existing `architecture-decision-record-author` + `-audit` pair. The Zero-Touch prompt's "self-approve architectural deviations" rule maps to: the ADR audit must pass at 10/10 against `adr-rubric@1.0` before the chain proceeds.

## 3. Edge-case matrix (steps 5-6)

Before implementation, the `edge-case-matrix` skill generates a structured matrix covering:

- Null / empty inputs
- Extreme bounds (off-by-one, integer overflow, time-zone DST, leap second)
- Malformed payloads (truncated, oversized, non-UTF8)
- Concurrent race conditions (double-submit, double-acknowledge, cross-tenant)
- Security-class entries (auth bypass, RLS escape, injection)

The audit (step 6) enforces the matrix is not vacuous — every category has ≥1 entry — and that SECURITY-class entries are paired to either an existing test or an ADR.

## 4. Mocks + contract tests (steps 7-8)

If `fr.has_external_dependency = true` (CAPTCHA / 2FA / paywall / missing API keys / future service), `mock-contract-test-author` defines:

- The **exact** expected Request/Response shape of the missing service.
- A Mock Service that **passes** the contract test.
- A `shipped + mocked-dependency` BACKLOG status tag.

The contract test stays in the suite forever — when the real dependency lands, swapping the mock out is a single import change and the contract guarantees behavioural parity.

## 5. Implementation (step 9-10)

The existing `implementation-plan-author` + `-audit` pair drives the actual code. Inputs are the FR, the edge-case matrix, and the (optional) ADR. The audit enforces: (a) every edge-case row is addressed in the plan, (b) the plan respects the existing patterns identified in step 1, (c) capacity estimate is reasonable.

## 6. Observability injection (steps 11-12)

`observability-injection-author` walks the critical paths of the new code and emits:

- Structured-log lines at every state transition (incl. `tenant_id`, `subject_id` when present).
- Trace spans wrapping every external IO.
- Counter increments for every error branch.

The audit checks coverage: ≥80 % of branches have a log/metric/trace point.

## 7. Coverage gate (steps 13-14)

`coverage-gate-author` runs the test suite, computes coverage on touched files, and fails the gate if the **per-file coverage on files touched in this FR** is < 90 %. The audit emits the raw terminal output of the coverage tool as the artefact.

## 8. Multi-vector debugging + circuit breaker (steps 15-16, conditional)

If the coverage gate fails OR any test fails, `debugging-cycle-author` runs the multi-vector pass:

1. Classify the failure vector (state / network / memory / logic / flake).
2. Output the exact hypothesis + the targeted file/line change.
3. Re-run the test suite.
4. After 5 consecutive failures, revert all touched files and trip the circuit breaker.

The audit emits the full hypothesis-and-attempt log so the human reviewer can see what was tried.

## 9. Backlog state update (steps 17-18)

The final skill writes the new BACKLOG.md status row (one of `shipped + strict-audited`, `shipped + mocked-dependency`, `[FAILED: UNRESOLVABLE ERROR]`, `[BLOCKED: <reason>]`), commits the diff to the working tree (the human runs `git add . && git commit && git push`), and emits the `workflow_complete` BRAIN audit row.

## 10. Outer loop

The CUO v3.0.0-a4 supervisor invokes this workflow in a loop:

```
while ! stop_signal:
    next_fr = read_backlog().next_eligible()
    if next_fr is None: break        # backlog drained
    invoke_workflow("chief-technology-officer/implement-backlog-frs", { repo_root, next_fr })
```

The supervisor handles persistence (state survives across sessions because the truth is in BACKLOG.md + the BRAIN chain), parallelism (multiple FRs may run in parallel when their dependency cones don't overlap), and observability (one workflow_complete row per FR is enough to reconstruct the run).

## Cross-references

- Original prompt source: operator's "Zero-Touch Principal Engineer (Unattended Execution)" — absorbed 2026-05-18.
- BACKLOG state engine: `docs/feature-requests/BACKLOG.md` + the `AUTHORING_DISCIPLINE.md` companion at `modules/skill/feature-request-audit/`.
- Companion workflow: `chief-technology-officer/architect-new-system` — produces the FRs this workflow consumes.

---

*End of `chief-technology-officer/implement-backlog-frs.md` workflow.*
