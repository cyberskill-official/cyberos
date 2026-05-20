---
workflow_id: chief-operating-officer/quarterly-capacity-plan
workflow_version: 1.0.0
purpose: Build the quarterly delivery capacity plan — role-by-role demand vs supply, gaps, hire triggers, contractor envelope.
persona: cuo/chief-operating-officer
cadence: quarterly
status: shipped

inputs:
  - { name: pipeline,            source: cuo/cso-sales (qualified pipeline + close dates), format: pipeline-report@1 }
  - { name: workforce_plan,      source: cuo/chro,                                          format: workforce-plan@1 }
  - { name: current_engagements, source: PMO,                                               format: csv export }

outputs:
  - { name: capacity_plan,       format: capacity-plan@1, recipient: cuo/coo + cuo/chro + cuo/cfo }

skill_chain:
  - { step: 1, skill: capacity-plan-author, inputs_from: { pipeline: pipeline, workforce_plan: workforce_plan, current_engagements: current_engagements }, outputs_to: plan_draft }
  - { step: 2, skill: capacity-plan-audit,  inputs_from: plan_draft, outputs_to: capacity_plan }

escalates_to:
  - { persona: cuo/chief-financial-officer,         when: "contractor-envelope blow-out > 15% of opex band" }
  - { persona: cuo/chief-human-resources-officer,        when: "hire-trigger fires for >3 roles simultaneously" }

consults:
  - { persona: cuo/chief-sales-officer,   when: "pipeline assumption needs realism check" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with capacity_plan hash + per-role gap count + contractor envelope
  - HITL pause at step 2 on QA-GAP-001 (role-gap without mitigation) or QA-OVERBOOK-001 (utilization plan >90%)
---

# Quarterly capacity plan — `chief-operating-officer/quarterly-capacity-plan`

COO's quarterly capacity-planning workflow. Maps qualified pipeline + workforce plan against current engagements per role; surfaces gaps; recommends hire triggers + contractor envelope. Critical for a services consultancy where capacity = revenue.

## When to invoke

- "Run the Q<n> capacity plan"
- "Do we have the people for next quarter"
- "Capacity vs demand check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-operating-officer/quarterly-capacity-plan \
  --input pipeline=./sales/2026-Q1/pipeline.md \
  --input workforce_plan=./hr/2026-workforce-plan.md \
  --input current_engagements=./pmo/2026-Q1/register.csv \
  --output-dir ./capacity/2026-Q2/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 3-5 business days for engagement-lead validation
- **Worst case:** gap-driven re-cut of forecasting may take 2 weeks

## Skill chain

- **Step 1 `capacity-plan-author`** — drafts per role / per timeframe demand-vs-supply.
- **Step 2 `capacity-plan-audit`** — validates per `capacity_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-GAP-001 | Role gap no mitigation | Operator drafts hire/contractor plan |
| 2 | QA-OVERBOOK-001 | Plan implies >90% util | Escalate to CHRO |

## Cross-references
- `../../../../modules/cuo/README.md` §5.1 — COO role profile
- `../../chief-human-resources-officer/workflows/quarterly-workforce-plan.md` — peer feeding hire-trigger data
- `../../../skill/capacity-plan-{author,audit}/SKILL.md`
