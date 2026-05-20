---
workflow_id: chief-restructuring-officer/monthly-restructuring-update
workflow_version: 1.0.0
purpose: Author the monthly restructuring status update — plan progress, milestones, covenant status, stakeholder positioning.
persona: cuo/chief-restructuring-officer
cadence: monthly
status: shipped

inputs:
  - { name: turnaround_plan,       source: cuo/chief-restructuring-officer/per-turnaround-plan, format: turnaround-plan@1 }
  - { name: cash_forecast,         source: cuo/chief-restructuring-officer/weekly-cash-flow (latest), format: thirteen-week-cash-flow@1 }
  - { name: milestone_status,      source: per-milestone progress data, format: markdown }

outputs:
  - { name: restructuring_update,  format: turnaround-plan@1 (monthly chapter), recipient: cuo/cro-restructuring + Board + lenders + sponsor }

skill_chain:
  - { step: 1, skill: turnaround-plan-author, inputs_from: { turnaround_plan: turnaround_plan, cash_forecast: cash_forecast, milestone_status: milestone_status }, outputs_to: update_draft }
  - { step: 2, skill: turnaround-plan-audit,  inputs_from: update_draft, outputs_to: restructuring_update }

audit_hooks:
  - workflow_complete row on PASS with restructuring_update hash + milestone slip count
  - HITL pause at step 2 on QA-MILESTONE-001
---

# Monthly restructuring update — `chief-restructuring-officer/monthly-restructuring-update`

CRO-Restructuring's monthly status update per AlixPartners / FTI standard reporting cadence for distressed engagements.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `./per-turnaround-plan.md` — upstream parent
- `../../../skill/turnaround-plan-{author,audit}/SKILL.md`
