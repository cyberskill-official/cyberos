---
workflow_id: chief-financial-officer/annual-budget
workflow_version: 1.0.0
purpose: Build the annual operating budget — revenue + opex + capex + headcount by function with board envelope and approval cycle.
persona: cuo/chief-financial-officer
cadence: annual
status: shipped

inputs:
  - { name: board_envelope,     source: CEO (post-board approval of strategic plan), format: markdown brief with top-line + EBITDA targets }
  - { name: function_drafts,    source: each function head's draft budget,            format: markdown / xlsx (one per function) }
  - { name: prior_actuals,      source: prior year monthly-close x12,                 format: monthly-close@1 set }
  - { name: workforce_plan,     source: cuo/chro,                                     format: workforce-plan@1 }

outputs:
  - { name: budget,             format: budget@1, recipient: Board of Directors + all function heads }

skill_chain:
  - { step: 1, skill: budget-author, inputs_from: { board_envelope: board_envelope, function_drafts: function_drafts, prior_actuals: prior_actuals, workforce_plan: workforce_plan }, outputs_to: budget_draft }
  - { step: 2, skill: budget-audit,  inputs_from: budget_draft, outputs_to: budget }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "function totals overshoot board envelope; CEO arbitrates re-allocation" }

consults:
  - { persona: cuo/chief-human-resources-officer,        when: "workforce-plan deltas vs draft headcount need reconciliation" }
  - { persona: cuo/chief-operating-officer,         when: "capex line items need ops review (esp. facilities + IT infra)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with budget hash + envelope variance %
  - HITL pause at step 2 on QA-ENVELOPE-001 (function sum > board envelope) or QA-VAR-001 (function ask >25% over prior actual)
---

# Annual budget — `chief-financial-officer/annual-budget`

CFO's annual operating-budget workflow. Cascades the board-approved envelope (top-line + EBITDA target) down to function-level budgets with reconciliation against the workforce plan. Audited for envelope-fit, prior-year variance, and zero-base discipline.

## When to invoke

- "Build the 2026 budget"
- "Run the annual budget cycle"
- "Roll up function budgets for board approval"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/annual-budget \
  --input board_envelope=./budget/2026/board-envelope.md \
  --input function_drafts=./budget/2026/function-drafts/ \
  --input prior_actuals=./close/2025/ \
  --input workforce_plan=./hr/2026-workforce-plan.md \
  --output-dir ./budget/2026/final/
```

## Expected duration

- **Happy path:** 3-5 hours runtime + 4-6 weeks for full bottom-up + top-down reconciliation
- **Worst case:** if board envelope is undersubscribed by 20%+, full re-cut adds 2 weeks

## Skill chain

- **Step 1 `budget-author`** — drafts per zero-base + functional-rollup template: drivers / opex by function / capex / headcount by month / contingency.
- **Step 2 `budget-audit`** — validates per `budget_rubric@1.0` (FM + SEC + QA-ENVELOPE-001 + QA-VAR-001 + QA-ZEROBASE-001 (every line justified)).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ENVELOPE-001 | Function sum > board envelope | Escalate to CEO for re-allocation |
| 2 | QA-VAR-001 | Function ask >25% over prior actual | Operator adds justification |
| 2 | QA-ZEROBASE-001 | Line item is copy-forward, not zero-based | Operator re-justifies |

## Cross-references
- `../README.md` §5 (Strategic) — output type "budget"
- `../../../../modules/cuo/README.md` §5.2
- `../../../skill/budget-{author,audit}/SKILL.md`
