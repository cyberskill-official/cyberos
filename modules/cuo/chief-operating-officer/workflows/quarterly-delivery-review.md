---
workflow_id: chief-operating-officer/quarterly-delivery-review
workflow_version: 1.0.0
purpose: Author the quarterly delivery-health review — per-engagement traffic-light, utilization, margin, risks, CSAT, capacity-vs-pipeline alignment.
persona: cuo/chief-operating-officer
cadence: quarterly
status: shipped

inputs:
  - { name: engagement_register, source: PMO / Linear / Jira / project tracker, format: csv export }
  - { name: utilization_data,    source: timesheet / capacity tool,             format: csv }
  - { name: financials,          source: cuo/cfo (engagement-level P&L),        format: monthly-close@1 segment data }
  - { name: csat_pulse,          source: customer-success-engagement@1 outputs (per engagement), format: customer-success-engagement@1 set }

outputs:
  - { name: delivery_review,     format: delivery-review@1, recipient: cuo/coo + cuo/ceo + Board (quarterly chapter) }

skill_chain:
  - { step: 1, skill: delivery-review-author, inputs_from: { engagement_register: engagement_register, utilization_data: utilization_data, financials: financials, csat_pulse: csat_pulse }, outputs_to: review_draft }
  - { step: 2, skill: delivery-review-audit,  inputs_from: review_draft, outputs_to: delivery_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,     when: "delivery-review-audit fires QA-MARGIN-001 — gross margin trending <30% on >2 engagements" }
  - { persona: cuo/chief-financial-officer,     when: "utilization < target by >10pts; revenue forecast at risk" }

consults:
  - { persona: cuo/chief-human-resources-officer,    when: "utilization >85% for >1 quarter — burnout/attrition risk" }
  - { persona: cuo/chief-customer-officer, when: "CSAT <8 on any major engagement — relationship intervention" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with delivery_review hash + green-count + utilization% + gross-margin%
  - HITL pause at step 2 on QA-RED-001 (red engagement without escalation plan)
---

# Quarterly delivery review — `chief-operating-officer/quarterly-delivery-review`

COO's quarterly delivery-health workflow for a delivery-led consultancy (CyberSkill's natural baseline). Combines engagement register + utilization + financials + CSAT into a traffic-light per-engagement view, rolls up to org-level utilization + margin, surfaces escalations. Output is the COO's chapter of the quarterly board update.

## When to invoke

- "Run the Q<n> delivery review"
- "Build the delivery health report for the board"
- "How are our engagements doing"

## How to invoke

```bash
cyberos-cuo run cuo/chief-operating-officer/quarterly-delivery-review \
  --input engagement_register=./pmo/2026-Q1/register.csv \
  --input utilization_data=./capacity/2026-Q1/util.csv \
  --input financials=./close/2026-Q1/segment.md \
  --input csat_pulse=./cs/2026-Q1/ \
  --output-dir ./delivery/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for engagement-lead round-trip
- **Worst case:** red-engagement intervention may add 2-4 weeks

## Skill chain

- **Step 1 `delivery-review-author`** — drafts per Bain agile-PMO + McKinsey delivery-excellence: per-engagement health / utilization / margin / risks / CSAT / capacity-vs-pipeline.
- **Step 2 `delivery-review-audit`** — validates per `delivery_review_rubric@1.0` (FM + SEC + QA-MARGIN-001 + QA-RED-001 + QA-UTIL-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RED-001 | Red engagement no escalation plan | Operator drafts plan |
| 2 | QA-MARGIN-001 | Gross margin <30% on multiple | Escalate to CEO |
| 2 | QA-UTIL-001 | Sustained over-utilization | Escalate to CHRO |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.1 — COO role profile
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — peer workflow that consumes this output
- `../../../skill/delivery-review-{author,audit}/SKILL.md`
