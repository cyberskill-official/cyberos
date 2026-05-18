---
workflow_id: chief-accounting-officer/annual-budget-controllership
workflow_version: 1.0.0
purpose: Support annual budget cycle from controllership perspective — actual baselines, accounting-driven cost classification, capex vs opex guidance.
persona: cuo/chief-accounting-officer
cadence: annual
status: shipped

inputs:
  - { name: cfo_budget,            source: cuo/chief-financial-officer/annual-budget, format: budget@1 }
  - { name: prior_actuals,         source: 12 months of monthly-close@1, format: monthly-close@1 (12) }
  - { name: accounting_policy,     source: cuo/chief-accounting-officer/annual-accounting-policy, format: strategy-document@1 (policy chapter) }

outputs:
  - { name: budget_controllership, format: budget@1 (controllership chapter), recipient: cuo/cao-accounting + cuo/cfo + function heads }

skill_chain:
  - { step: 1, skill: budget-author, inputs_from: { cfo_budget: cfo_budget, prior_actuals: prior_actuals, accounting_policy: accounting_policy }, outputs_to: controllership_draft }
  - { step: 2, skill: budget-audit,  inputs_from: controllership_draft, outputs_to: budget_controllership }

audit_hooks:
  - workflow_complete row on PASS with budget_controllership hash
  - HITL pause at step 2 on QA-CLASSIFICATION-001 (capex vs opex)
---

# Annual budget controllership — `chief-accounting-officer/annual-budget-controllership`

CAO-Accounting's controllership support for annual budget per Adaptive / Anaplan / Pigment + US GAAP capex-vs-opex guidance.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../chief-financial-officer/workflows/annual-budget.md` — upstream parent
- `../../../skill/budget-{author,audit}/SKILL.md`
