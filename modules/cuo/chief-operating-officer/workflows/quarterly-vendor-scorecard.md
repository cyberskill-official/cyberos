---
workflow_id: chief-operating-officer/quarterly-vendor-scorecard
workflow_version: 1.0.0
purpose: Score top vendors against SLA + cost + risk + strategic-fit; identify consolidation / renewal / off-board candidates.
persona: cuo/chief-operating-officer
cadence: quarterly
status: shipped

inputs:
  - { name: vendor_register,     source: AP / procurement system, format: csv export }
  - { name: sla_attainment,      source: per-vendor SLA reports,  format: csv / markdown }
  - { name: spend_data,          source: AP,                      format: csv }
  - { name: renewal_calendar,    source: procurement calendar,    format: markdown }

outputs:
  - { name: vendor_scorecard,    format: vendor-scorecard@1, recipient: cuo/coo + cuo/cfo + cuo/cpo-procurement }

skill_chain:
  - { step: 1, skill: vendor-scorecard-author, inputs_from: { vendor_register: vendor_register, sla_attainment: sla_attainment, spend_data: spend_data, renewal_calendar: renewal_calendar }, outputs_to: scorecard_draft }
  - { step: 2, skill: vendor-scorecard-audit,  inputs_from: scorecard_draft, outputs_to: vendor_scorecard }

escalates_to:
  - { persona: cuo/chief-financial-officer,             when: "scorecard recommends off-boarding >$100K/yr vendor — financial impact" }
  - { persona: cuo/chief-legal-officer,       when: "off-board candidate has cancellation-penalty exposure" }

consults:
  - { persona: cuo/chief-procurement-officer, when: "renewal in next quarter; consolidation opportunity vs another vendor in same category" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with vendor_scorecard hash + off-board count + renewal-window count
  - HITL pause at step 2 on QA-SLA-001 (SLA breach not flagged) or QA-RISK-001 (risk score without rationale)
---

# Quarterly vendor scorecard — `chief-operating-officer/quarterly-vendor-scorecard`

COO's quarterly vendor scoring. Combines SLA attainment + spend + risk + strategic-fit into a structured scorecard with renew / consolidate / off-board recommendations. Per Kraljic matrix + CIPS vendor-management standards (referenced from `procurement-strategy` skill).

## When to invoke

- "Run the Q<n> vendor scorecard"
- "Vendor performance review"
- "Renewal triage for next quarter"

## How to invoke

```bash
cyberos-cuo run cuo/chief-operating-officer/quarterly-vendor-scorecard \
  --input vendor_register=./vendors/2026-Q1/register.csv \
  --input sla_attainment=./vendors/2026-Q1/sla/ \
  --input spend_data=./ap/2026-Q1/vendor-spend.csv \
  --input renewal_calendar=./procurement/renewal-calendar.md \
  --output-dir ./vendors/2026-Q1/
```

## Expected duration

- **Happy path:** 1-3 hours runtime + 1 week for vendor-owner sign-off
- **Worst case:** off-board recommendation may trigger 1-quarter notice-of-cancellation cycle

## Skill chain

- **Step 1 `vendor-scorecard-author`** — drafts per Kraljic matrix structure: per-vendor score / category / recommendation.
- **Step 2 `vendor-scorecard-audit`** — validates per `vendor_scorecard_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SLA-001 | SLA breach unmissed | Operator surfaces |
| 2 | QA-RISK-001 | Risk score lacks rationale | Operator adds |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.1 — COO role profile
- `../../cpo-procurement/README.md` — peer persona owning the procurement strategy
- `../../../skill/vendor-scorecard-{author,audit}/SKILL.md`
