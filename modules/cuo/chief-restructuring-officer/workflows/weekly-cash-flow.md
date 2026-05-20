---
workflow_id: chief-restructuring-officer/weekly-cash-flow
workflow_version: 1.0.0
purpose: Refresh the 13-week cash forecast under distress — weekly receipts, disbursements, debt-service, minimum-cash covenant tracking.
persona: cuo/chief-restructuring-officer
cadence: weekly
status: shipped

inputs:
  - { name: prior_twcf,            source: last week's 13-week-cash-flow@1, format: thirteen-week-cash-flow@1 }
  - { name: ar_aging,              source: ERP AR (real-time), format: csv }
  - { name: ap_aging,              source: ERP AP (real-time), format: csv }
  - { name: debt_schedule,         source: treasury + covenant tracking, format: markdown }

outputs:
  - { name: distress_cash_forecast, format: 13-week-cash-flow@1, recipient: cuo/cro-restructuring + cuo/cfo + lenders + Board (weekly during distress) }

skill_chain:
  - { step: 1, skill: thirteen-week-cash-flow-author, inputs_from: { prior_twcf: prior_twcf, ar_aging: ar_aging, ap_aging: ap_aging, debt_schedule: debt_schedule }, outputs_to: twcf_draft }
  - { step: 2, skill: thirteen-week-cash-flow-audit,  inputs_from: twcf_draft, outputs_to: distress_cash_forecast }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "runway < 4 weeks OR covenant breach within window" }
  - { persona: cuo/chief-legal-officer,      when: "covenant trip imminent — pre-negotiation needed" }

audit_hooks:
  - workflow_complete row on PASS with distress_cash_forecast hash + min-cash-week
  - HITL pause at step 2 on QA-CASH-RECON-001 (week-over-week reconciliation) or QA-RUNWAY-001
---

# Weekly cash flow (distress mode) — `chief-restructuring-officer/weekly-cash-flow`

CRO-Restructuring's distress-mode weekly TWCF — high-frequency variant of CFO's `monthly-cash-management`. Standard TWCF runs monthly; distress TWCF runs weekly with same-day actuals reconciliation.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-financial-officer/workflows/monthly-cash-management.md` — normal-ops peer
- `../../../skill/13-week-cash-flow-{author,audit}/SKILL.md`
