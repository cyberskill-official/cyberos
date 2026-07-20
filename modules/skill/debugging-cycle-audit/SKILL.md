---
# ── Identity ─────────────────────────────────────────────────────────
name: debugging-cycle-audit
description: >-
  Audit a debug-trace@1 against debugging_cycle_rubric@1.0: enforces budget compliance, non-vacuous hypotheses, resolvable file:line references, correct circuit-breaker arithmetic, and a defined resolution. Emits a `score / 10` verdict; refuses to pass on <10/10. Use when user asks to "audit this debugging cycle" or "check the debugging cycle". Do NOT use for "draft a new debugging cycle" (use debugging-cycle-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: debug-trace-audit@1
  cyberos-rubric-target: debugging_cycle_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:task/{task_id}/debug-trace.audit

audit:
  row_kind: debug_cycle_audited
  required_fields: [task_id, score, issues_open, issues_resolved]

inputs:
  - { name: debug_trace, format: debug-trace@1, required: true }
outputs:
  - { name: audit_report, format: debug-trace-audit@1 }
---

# debugging-cycle-audit

## 1. Rubric (debugging_cycle_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| DBG-001 | `attempts.length ≤ budget_max_attempts` (default 5) | 20% | error |
| DBG-002 | Every attempt has a non-vacuous `hypothesis` (≥ 1 full sentence, not "retry") | 20% | error |
| DBG-003 | Every attempt's `change.file:lines` resolves to a real diff in git | 15% | error |
| DBG-004 | `consecutive_failures` arithmetic matches the outcome chain | 15% | error |
| DBG-005 | `circuit_breaker_tripped == (consecutive_failures ≥ 5)` | 15% | error |
| DBG-006 | `resolution` is one of `passed | tripped-circuit-breaker` — never blank | 10% | error |
| DBG-007 | If tripped, `on_trip_actions` lists at least the three required ops (revert, mark FAILED, emit workflow_failed row) | 5% | error |

## 2. Pass criterion

10/10. The workflow proceeds to step 17 (backlog-state-update-author) once this audit passes — regardless of whether the resolution is `passed` or `tripped-circuit-breaker`, because the backlog update records both outcomes.

---

*End of debugging-cycle-audit SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
