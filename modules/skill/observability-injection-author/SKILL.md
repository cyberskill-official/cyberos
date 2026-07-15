---
# ── Identity ─────────────────────────────────────────────────────────
name: observability-injection-author
description: >-
  Walk the critical paths of the task's implementation plan and emit an `observability-injection@1`: (a) structured-log lines at every state transition (always carrying tenant_id + subject_id when in-scope), (b) trace spans wrapping every external IO call, (c) counter increments for every error branch, (d) a coverage estimate (% of branches with a log/metric/trace point). Used by chief-technology-officer/ship-tasks as step 11. Use when user asks to "draft a observability injection" or "create the observability injection". Do NOT use for "audit existing observability injection" (use observability-injection-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: observability-injection@1
  cyberos-rubric-target: observability_injection_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/observability-injection
audit:
  row_kind: observability_injection_authored
  required_fields: [task_id, log_points, trace_spans, error_counters, branch_coverage_pct]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,        format: task@1,       required: true }
  - { name: impl_plan, format: implementation-plan@1,   required: true }
outputs:
  - { name: obs_injection, format: observability-injection@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - workflow `chief-technology-officer/ship-tasks` step 11
blockers:
  - "no observability sink configured in repo (tracing/log subscriber missing) — must be resolved first"
  - "implementation-plan has zero critical paths — author misclassified the task; escalate"
---

# observability-injection-author

## 1. Purpose

Make the new code legible at runtime — every state transition logged,
every external IO traced, every error branch counted — without bolting
observability on as an afterthought. Plan the instrumentation **before**
implementation; the implementation-plan-author treats this as a
required input for any path it generates.

## 2. Output schema

```yaml
# observability-injection@1
task_id: task-<MODULE>-<NNN>
generated_at: <ISO-8601>
language: rust | python | typescript | mixed
subscriber: "tracing / log / structlog / pino / ..."

log_points:
  - { id: LOG-001, file: "<absolute>", line_hint: "<int|null>", level: info | warn | error, message_shape: "...", carries: [tenant_id, subject_id, task_id], when: "<state transition>" }

trace_spans:
  - { id: SPAN-001, file: "<absolute>", wraps: "<function or external IO call>", attributes: [tenant_id, subject_id, downstream], propagates: true }

error_counters:
  - { id: ERR-001, metric_name: "cyberos_<module>_<error>_total", labels: [error_class, tenant_id], increments_at: "<file:branch>" }

branch_coverage:
  total_branches: <int>
  branches_with_obs_point: <int>
  coverage_pct: <float>

redaction_policy:
  - { field_pattern: "password|secret|token", action: "drop-before-emit" }
  - { field_pattern: "email|phone|name",      action: "hash-before-emit (sha256_truncated)" }
```

## 3. Quality gates

- Every state transition in `impl_plan.state_machine` has ≥ 1 log_point.
- Every external IO in `impl_plan.external_calls` has ≥ 1 trace_span.
- Every error branch in `impl_plan.error_branches` has ≥ 1 error_counter.
- `branch_coverage.coverage_pct ≥ 80`.
- `redaction_policy` is non-empty if the task touches any PII (verified
  against the task's `data:` frontmatter classification).

## 4. Chains to

`observability-injection-audit` then `coverage-gate-author`.

---

*End of observability-injection-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
