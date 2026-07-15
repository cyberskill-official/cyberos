---
template: task@1
id: TASK-CUO-203
title: "Harness Wave 4 — workflow-level evolution via outcome distribution"
type: feature
author: "@stephen"
department: engineering
status: done
priority: p2
created_at: 2026-05-19T20:45:00+07:00
ai_authorship: assisted
eu_ai_act_risk_class: limited
target_release: 2026-Q3
client_visible: false
module: cuo
new_files:
  - modules/cuo/cuo/core/workflow_evolution.py
  - modules/cuo/tests/test_workflow_evolution.py
depends_on: [TASK-CUO-200, TASK-CUO-201, TASK-CUO-202]
---

## Summary

Wave 4 of the continuous-improvement harness extends TASK-CUO-200..202 from skill-level to workflow-level evolution. The harness aggregates per-workflow outcome distributions (COMPLETED / ROUTED_BACK / HITL_HALT / FAILED) over rolling windows, and when a workflow's ROUTED_BACK rate exceeds a threshold OR the same phase keeps tripping repeat failures across multiple tasks, it emits a `workflow_refinement_proposal@1` proposing chain edits — e.g. "add a pre-review check before phase 8", "tighten the coverage-gate threshold from 90% to 95% per file", "swap the order of steps 11 and 12 so observability lands before impl-plan".

The stripe-dedup + auto-bump machinery from TASK-CUO-201/202 reuses cleanly — workflow stripes are just `<workflow_id>:<phase>:<pattern_hash>` and the applier extends to mutate workflow YAML frontmatter + skill_chain steps with the same major/minor/safety classification.

## Problem

Skill-level evolution (TASK-CUO-201/202) catches "skill X is failing too often". But sometimes the root cause is the WORKFLOW shape — e.g. ship-tasks' `coverage-gate-audit` keeps tripping on the same module because `implementation-plan-audit` doesn't check coverage planning early enough. Fixing the skill itself doesn't help; the chain needs a pre-coverage check inserted at step 10.

Without workflow-level evolution, structural issues that span multiple skills go unnoticed until Stephen manually correlates the per-skill reports and decides to redesign the chain. The harness should propose chain edits the same way it proposes skill edits.

## Proposed Solution

### §1 Normative requirements

1. **MUST** ship `cuo/core/workflow_evolution.py` with `compute_workflow_metrics(audit_dir, window) -> dict[str, WorkflowMetrics]` that aggregates per-workflow: total runs, COMPLETED count, ROUTED_BACK count (sum + per-task median), HITL_HALT count, average step count of failed runs, top-3 steps where failure occurred most often.
2. **MUST** define workflow-level threshold signals (mirroring SKILL-level `self_audit.anomaly_signals`):
   - `routed_back_rate_above` (default 0.3 → 30% of runs route back)
   - `hitl_halt_rate_above` (default 0.1 → 10% of runs halt for human)
   - `repeat_phase_failure_above` (default 3 → same phase fails on 3+ different tasks in window)
   - `chain_length_efficiency_below` (default 0.7 → average steps_run / chain_length below 70%)
3. **MUST** declare these signals in each workflow file's `self_audit:` frontmatter (similar to SKILL.md's). Workflow YAML extension: add the same `self_audit` + `human_fine_tune` blocks the SKILL.md frontmatter already has. (Schema change in `cuo/contracts/workflow/CONTRACT.md`.) *(traces_to: §1 #3 → AC #2)*
4. **MUST** compute workflow stripes via `compute_workflow_stripe(workflow_id, signal_id, failure_pattern) -> str` with format `<persona>/<workflow_slug>:<signal_id>:<pattern_hash>`.
5. **MUST** route workflow-refinement proposals to the SAME `docs/proposals/open/` directory as skill proposals (TASK-CUO-201). They are distinguishable by `frontmatter.kind: workflow_refinement` vs `skill_refinement`.
6. **MUST** extend TASK-CUO-202's proposal classifier to handle workflow diffs:
   - `step_addition` (insert a new step in `skill_chain:`) — minor bump (workflow_version), queue.
   - `step_removal` — major bump, queue.
   - `step_reorder` — major bump, queue.
   - `escalates_to_addition` — minor bump, queue.
   - `pattern_change` (e.g. `linear` → `per_instance`) — major bump, queue.
   - `condition_tune` (numeric threshold in a `condition:` clause) — minor bump, queue.
   - All workflow diffs default to QUEUE — workflows are higher-stakes than individual skills. Auto-apply requires explicit `--auto-workflow-diffs` flag.
7. **MUST** emit `cuo.workflow_refinement_emitted` memory aux row per new workflow proposal; `cuo.workflow_refinement_applied` per apply.
8. **MUST** add CLI:
   - `cyberos-cuo workflow metrics --since 30d` — table of per-workflow outcome distribution.
   - `cyberos-cuo workflow propose --workflow <id>` — manually trigger proposal authoring against one workflow.
9. **SHOULD** include per-task routed-back history in the workflow report — if task-X has routed back 3+ times in the window, the proposal body cites it as the canonical case study.
10. **MUST** include in proposal body: `## Before` (current chain snippet), `## After` (proposed chain snippet), `## Rationale` (which signal tripped + evidence rows), `## Backward-compat notes` (whether the change breaks in-flight tasks at older chain positions).

### §2 Stripe collision with skill stripes

Skill stripes from TASK-CUO-201 have format `<skill_slug>:<signal_id>:<hash>`; workflow stripes have format `<persona>/<workflow_slug>:<signal_id>:<hash>`. The presence of `/` in workflow stripes makes them disjoint from skill stripes — no collision possible. `cyberos-cuo proposal list` SHOULD group by kind (skill vs workflow) for readability.

## Alternatives Considered

1. **Workflow proposals as a separate folder** (`docs/workflow-proposals/`) — splits the operator surface area unnecessarily. Unified `docs/proposals/` with `kind:` discrimination is simpler.
2. **Only humans propose workflow edits** — but workflow drift is exactly the kind of thing the harness is best at noticing across many tasks; humans see one task at a time.
3. **Tie workflow evolution to skill evolution (apply skill diff cascades into workflow diff)** — too coupled; the propagation rules become fragile. Keep them independent.

## Success Metrics

| metric | baseline | target | deadline |
|---|---|---|---|
| Workflow refinement proposals produced per quarter | n/a | 2–5 per workflow that ships > 50 tasks | 30 days post-ship |
| % of workflow proposals approved by Stephen | n/a | ≥ 50% | continuous |
| ROUTED_BACK rate improvement for workflows with applied proposals | n/a | -25% in the following window | per-proposal |

## Scope

In scope: per-workflow metrics aggregator, threshold-signal evaluator (reusing TASK-CUO-200's signal-function pattern), workflow stripe computer, proposal authoring (LLM with workflow self-reflection prompt), classifier extension, CLI subcommands, audit row emission.

### Out of scope

- Multi-workflow refactors (single proposal touches > 1 workflow)
- Persona-level evolution (e.g. proposing to spawn a new persona)
- Auto-runs of proposed chain edits on the CURRENT in-flight tasks (those use the chain as it was at start of their run; new chains apply to future invocations)

## Dependencies

- **TASK-CUO-200** — read-only harness foundation
- **TASK-CUO-201** — proposal emitter + stripe dedup (reused for workflow stripes)
- **TASK-CUO-202** — proposal applier (extended in §1 #6 for workflow diff buckets)

## AI Risk Assessment

### Data Sources

The workflow-evolution harness reads three trusted sources: the memory audit chain (operator-controlled append-only log), workflow YAML frontmatter (operator-authored), and skill SKILL.md files (operator-authored, audit-gated). No external network calls. Proposal authoring runs an LLM with the failing workflow's own definition + the failing audit rows as context — no untrusted user input.

### Human Oversight

ALL workflow-level diffs default to queue (§1 #6 says "All workflow diffs default to QUEUE — workflows are higher-stakes than individual skills"). Auto-apply requires an explicit `--auto-workflow-diffs` flag on `cyberos-cuo proposal apply`, AND the proposal bucket must still classify as `condition_tune` or below. Pattern changes (e.g. `linear → time_critical`) are MAJOR + always queue regardless of flags. Stripe-repeat-halt (inherited from TASK-CUO-201) means the harness halts immediately rather than proposing a second workflow edit for the same root cause.

### Failure Modes

(a) **Mid-flight task uses old chain** — when a chain edit applies during an in-flight task run, the task continues using the chain it started with (snapshot at run-start). Risk: the operator expects the new chain to fix the in-flight task. Mitigation: §2 explicitly out-of-scope; only future invocations use the new chain. (b) **Workflow stripe collision** — workflow stripes use `<persona>/<workflow_slug>` format; the `/` makes them disjoint from skill stripes — verifiable via `compute_workflow_stripe` test (AC #5). (c) **Proposed step references a non-existent skill** — the validator (existing `validate_chain`) catches this BEFORE the applier writes; the proposal is rejected as ill-formed. (d) **Backward-compat regression** — the proposal body's `## Backward-compat notes` section (§1 #10) is mandatory; if the LLM omits it, the proposal fails its own structural validation.

## AI Authorship Disclosure

- **Tools used:** Anthropic Claude.
- **Scope:** §1 normative clauses (10 items), §2 stripe collision discussion, §4 ACs (10 items), §5 named tests, alternatives, AI Risk Assessment.
- **Human review:** Stephen Cheng reviewed. The "all workflow diffs default to queue" rule (§1 #6) is operator-mandated (workflows are higher-stakes than skills).

## §4 Acceptance Criteria

1. `cyberos-cuo workflow metrics --since 30d` outputs per-workflow rows: total_runs, completed, routed_back, hitl_halt, failed, average chain length used. *(traces_to: §1 #1, #8)*
2. A workflow with 5 runs all COMPLETED has zero tripped signals. *(traces_to: §1 #2)*
3. A workflow with 4 ROUTED_BACK out of 10 runs trips `routed_back_rate_above: 0.3` and emits one `workflow_refinement_proposal@1` to `docs/proposals/open/`. *(traces_to: §1 #2, #5, #7)*
4. The emitted proposal's body has all 4 mandatory sections: Before / After / Rationale / Backward-compat. *(traces_to: §1 #10)*
5. Workflow stripe format MUST be `<persona>/<workflow_slug>:<signal_id>:<8 hex>` — verifiable via regex. *(traces_to: §1 #4)*
6. The classifier (extension of TASK-CUO-202) returns `step_addition` → `minor` + queue for a proposal that inserts a new step in the `skill_chain:` block. *(traces_to: §1 #6)*
7. Repeat occurrence of the same workflow stripe (per TASK-CUO-201's halt-on-repeat rule) halts the workflow with `HITL_HALT`. *(traces_to: §1 #5 via TASK-CUO-201 §1 #7)*
8. Per applied workflow proposal: exactly one `cuo.workflow_refinement_applied` aux row + `workflow_version` in YAML frontmatter is bumped per §2 rules. *(traces_to: §1 #6, #7)*
9. A workflow proposal whose pattern change converts `linear` → `time_critical` is classified `major` + queued (not auto-applied even with `--auto-workflow-diffs`). *(traces_to: §1 #6)*
10. The workflow report cites at least one specific task id per tripped signal — the operator can drill from the report into the task's debug trace. *(traces_to: §1 #9)*

## §5 Verification

- `modules/cuo/tests/test_workflow_evolution.py::test_metrics_aggregation` (AC #1, #2)
- `modules/cuo/tests/test_workflow_evolution.py::test_routed_back_rate_trips` (AC #3)
- `modules/cuo/tests/test_workflow_evolution.py::test_proposal_body_sections` (AC #4)
- `modules/cuo/tests/test_workflow_evolution.py::test_workflow_stripe_format` (AC #5)
- `modules/cuo/tests/test_workflow_evolution.py::test_step_addition_classifier` (AC #6)
- `modules/cuo/tests/test_workflow_evolution.py::test_repeat_stripe_halts` (AC #7)
- `modules/cuo/tests/test_workflow_evolution.py::test_apply_emits_audit_row` (AC #8)
- `modules/cuo/tests/test_workflow_evolution.py::test_pattern_change_is_major` (AC #9)
- `modules/cuo/tests/test_workflow_evolution.py::test_report_cites_task_ids` (AC #10)
