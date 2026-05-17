---
workflow_id: chief-human-resources-officer/quarterly-workforce-plan
workflow_version: 1.0.0
purpose: Refresh the rolling workforce plan — hire-by-quarter, attrition forecast, role-by-role pipeline, contractor envelope.
persona: cuo/chief-human-resources-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_workforce_plan, source: last quarter's workforce-plan@1,         format: workforce-plan@1 }
  - { name: capacity_signal,      source: cuo/coo (quarterly-capacity-plan deltas), format: capacity-plan@1 }
  - { name: attrition_data,       source: HRIS,                                    format: csv }
  - { name: budget_envelope,      source: cuo/cfo (annual budget headcount line),  format: budget@1 chapter }

outputs:
  - { name: workforce_plan,       format: workforce-plan@1, recipient: cuo/chro + cuo/coo + cuo/cfo + cuo/ceo }

skill_chain:
  - { step: 1, skill: workforce-plan-author, inputs_from: { prior_workforce_plan: prior_workforce_plan, capacity_signal: capacity_signal, attrition_data: attrition_data, budget_envelope: budget_envelope }, outputs_to: plan_draft }
  - { step: 2, skill: workforce-plan-audit,  inputs_from: plan_draft, outputs_to: workforce_plan }

escalates_to:
  - { persona: cuo/chief-financial-officer,         when: "headcount plan exceeds budget envelope by >5%" }
  - { persona: cuo/chief-executive-officer,         when: "attrition >15% trailing quarter at any function" }

consults:
  - { persona: cuo/chief-operating-officer,         when: "delivery utilization signals capacity stress" }
  - { persona: cuo/chief-customer-officer, when: "customer-facing role gaps risk CSAT" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with workforce_plan hash + hire count by quarter + attrition forecast
  - HITL pause at step 2 on QA-ATTRITION-001 (attrition forecast lacks driver narrative)
---

# Quarterly workforce plan — `chief-human-resources-officer/quarterly-workforce-plan`

CHRO's rolling 4-quarter workforce plan. Combines prior plan + COO capacity signal + attrition data + budget envelope into hire-by-quarter / role-by-role / contractor-envelope plan. Standard quarterly cadence; ad-hoc refresh when major engagement won/lost.

## When to invoke

- "Refresh the workforce plan"
- "Rolling hire plan for next 4 quarters"
- "Workforce-vs-capacity check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-human-resources-officer/quarterly-workforce-plan \
  --input prior_workforce_plan=./hr/2026-Q1/workforce-plan.md \
  --input capacity_signal=./capacity/2026-Q2/plan.md \
  --input attrition_data=./hr/2026-Q1/attrition.csv \
  --input budget_envelope=./budget/2026/hc-chapter.md \
  --output-dir ./hr/2026-Q2/
```

## Expected duration

- **Happy path:** 1-3 hours runtime + 1-2 weeks for function-head round-trip
- **Worst case:** budget overshoot triggers 1-quarter re-plan

## Skill chain

- **Step 1 `workforce-plan-author`** — drafts per SHRM workforce-planning model.
- **Step 2 `workforce-plan-audit`** — validates per `workforce_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ATTRITION-001 | Attrition forecast no driver | Operator adds narrative |
| 2 | QA-BUDGET-001 | Headcount > envelope | Escalate to CFO |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5 — CHRO role profile
- `../../chief-operating-officer/workflows/quarterly-capacity-plan.md` — peer feeding capacity signal
- `../../../skill/workforce-plan-{author,audit}/SKILL.md`
