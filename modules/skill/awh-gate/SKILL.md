---
# ── Identity ─────────────────────────────────────────────────────────
name: awh-gate
description: >-
  Out-of-band verification gate for the testing to done transition. Independently reruns the
  Task's section-1 cited tests plus its module suite via `awh eval` against a sealed,
  read-only baseline, and blocks the transition on any regression. Emits an awh-eval@1 artefact:
  per-task pass@1, the weighted aggregate, the sealed-baseline hash, and a GREEN or RED verdict.
  Used by chief-technology-officer/ship-tasks at step 28, after the post-implementation
  Task-audit (step 27) and before the done flip (step 30). GREEN is required to reach
  done; RED routes the task back to ready_to_implement per STATUS-REFERENCE section 1.3.
  Use when the user asks to "run the awh gate", "verify this task out of band", or "gate testing to
  done". Do NOT use for spec correctness (that is task-audit, during draft to
  ready_to_implement) or for in-context coverage (that is coverage-gate-author); this skill is the
  independent rerun those two are not, because an agent grading its own work is not a check.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: awh-eval@1
  cyberos-rubric-target: awh_gate_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:task/{task_id}/awh-gate
audit:
  row_kind: awh_gate_result
  required_fields: [task_id, module, outcome, weighted_pass, harness_version, sealed_acceptance_hash]

inputs:
  - { name: task,        format: task@1, required: true }
  - { name: module,    format: string,            required: true }
  - { name: goldenset, format: path,              required: true }
outputs:
  - { name: report, format: awh-eval@1 }

triggers:
  - "run the awh gate"
  - "verify this task out of band"
  - "gate testing to done"
  - "independent rerun before done"
---

# awh-gate

The out-of-band verification gate. CyberOS already has author and audit skill pairs, a coverage gate, and a task audit, but each of those runs in the authoring context, so the model is grading its own homework. This skill is the independent layer absorbed from auto-work-harness: it reruns the real tests outside that context and blocks the testing to done transition on regression.

## When it runs

Step 28 of `chief-technology-officer/ship-tasks`, between the post-implementation `task-audit` (step 27) and the `backlog-state-update-author` done flip (step 30). The done flip is conditional on this skill returning GREEN.

## What it does

1. Resolve the task's module and its golden set at `modules/<module>/.awh/goldenset.yaml`.
2. Run `awh eval <goldenset> --base-dir . --seeds 1 --baseline modules/<module>/.awh/eval-baseline.json --max-regression 0.0`. The golden set reruns the module's real build and test plus the held-out acceptance test, which is sealed read-only via `awh lock` so the agent cannot edit the bar it is graded against.
3. Read the verdict. GREEN (no task regressed against the sealed baseline) is required to proceed to the done flip. RED routes the task back to `ready_to_implement` per STATUS-REFERENCE section 1.3 with `routed_back_count += 1`.
4. Emit one `awh_gate_result` row into the memory audit chain carrying `{task_id, module, outcome, weighted_pass, harness_version, sealed_acceptance_hash}`. The row kind is gated on protocol change P23 section 6; until that lands, the verdict is written to a side log (`.awh/gate-results.jsonl`).

## What it is not

This skill does not rewrite code or tests. It gates and measures. It does not replace the coverage gate or the spec audit; it is the independent rerun that confirms their result. See `website/docs/architecture/verification-gate.html` and `tools/awh/` for the vendored tool, and `tools/awh/RETIREMENT.md` for how the standalone auto-work-harness is retired once every module is green under this gate.

## Provenance

Vendored from auto-work-harness (source sha c1f2c77). Maturity ledger at `.awh/evolution-log.jsonl` (read with `awh maturity report --log .awh/evolution-log.jsonl`).
