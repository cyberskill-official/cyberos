---
workflow_id: chief-financial-officer/monthly-cash-management
workflow_version: 1.0.0
purpose: Refresh the 13-week cash forecast — operating receipts, disbursements, financing activity — with weekly reconciliation against actuals.
persona: cuo/chief-financial-officer
cadence: monthly
status: shipped

inputs:
  - { name: prior_twcf,         source: last month's 13-week-cash-flow@1, format: thirteen-week-cash-flow@1 }
  - { name: ar_aging,           source: NetSuite / SAP AR module,         format: csv aging report }
  - { name: ap_aging,           source: NetSuite / SAP AP module,         format: csv aging report }
  - { name: payroll_schedule,   source: Gusto / ADP / Rippling,           format: csv calendar }
  - { name: debt_schedule,      source: treasury,                         format: markdown schedule }

outputs:
  - { name: cash_forecast,      format: 13-week-cash-flow@1, recipient: cuo/cfo + cuo/ceo + lenders (covenant compliance) }

skill_chain:
  - { step: 1, skill: thirteen-week-cash-flow-author, inputs_from: { prior_twcf: prior_twcf, ar_aging: ar_aging, ap_aging: ap_aging, payroll_schedule: payroll_schedule, debt_schedule: debt_schedule }, outputs_to: twcf_draft }
  - { step: 2, skill: thirteen-week-cash-flow-audit,  inputs_from: twcf_draft, outputs_to: cash_forecast }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "thirteen-week-cash-flow-audit fires QA-RUNWAY-001 — runway <6 months at projected burn" }
  - { persona: cuo/chief-legal-officer,   when: "QA-COVENANT-001 fires — covenant breach risk within forecast window" }

consults:
  - { persona: cuo/chief-accounting-officer, when: "AR collections lag >30 days vs DSO target" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with cash_forecast hash + min-cash-week + projected runway
  - HITL pause at step 2 on QA-CASH-RECON-001 (week N+1 starting cash ≠ week N ending cash)
---

# Monthly cash management — `chief-financial-officer/monthly-cash-management`

CFO's monthly cash-management workflow using the 13-week-cash-flow (TWCF) industry-standard model. Refreshes weekly receipts/disbursements/financing/min-cash by reconciling prior TWCF against actuals and rolling forward. Standard tool during normal operations AND mandatory during distress (where it transitions to CRO-Restructuring per chief-restructuring-officer/weekly-cash-flow).

## When to invoke

- "Refresh the 13-week cash forecast"
- "Run TWCF for May"
- "Update the cash runway forecast"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/monthly-cash-management \
  --input prior_twcf=./cash/2026-04/twcf.md \
  --input ar_aging=./close/2026-05/ar-aging.csv \
  --input ap_aging=./close/2026-05/ap-aging.csv \
  --input payroll_schedule=./hr/2026-payroll-calendar.csv \
  --input debt_schedule=./treasury/debt-schedule.md \
  --output-dir ./cash/2026-05/
```

## Expected duration

- **Happy path:** 30-60 min runtime + 1 business day operator review
- **Worst case:** covenant-breach risk escalation may trigger same-day CLO-Legal + CEO discussion

## Skill chain

- **Step 1 `thirteen-week-cash-flow-author`** — drafts per TWCF standard structure: 13 weekly columns × (operating receipts / operating disbursements / financing / net change / cumulative cash).
- **Step 2 `thirteen-week-cash-flow-audit`** — validates per `13_week_cash_flow_rubric@1.0` (FM + SEC + QA-CASH-RECON-001 (period-over-period reconciliation) + QA-RUNWAY-001 + QA-COVENANT-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CASH-RECON-001 | Week N+1 starting cash ≠ week N ending cash | Operator fixes formulae |
| 2 | QA-RUNWAY-001 | Runway <6 months | Escalate to CEO |
| 2 | QA-COVENANT-001 | Covenant breach within window | Escalate to CLO-Legal |

## Cross-references
- `../README.md` §5 (Operational) — output type "cash mgmt"
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../chief-restructuring-officer/workflows/weekly-cash-flow.md` — distress-mode peer workflow that takes over when restructuring is declared
- `../../../skill/13-week-cash-flow-{author,audit}/SKILL.md`
