---
workflow_id: chief-revenue-officer/weekly-revenue-cadence
workflow_version: 1.0.0
purpose: Weekly revenue operations cadence — pipeline + CS health + expansion signals + renewal-at-risk in one consolidated view.
persona: cuo/chief-revenue-officer
cadence: weekly
status: shipped

inputs:
  - { name: pipeline,           source: cuo/chief-sales-officer/weekly-pipeline-review, format: pipeline-report@1 }
  - { name: cs_engagement_set,  source: customer-success-engagement@1 for top accounts,     format: customer-success-engagement@1 (multiple) }
  - { name: prior_cadence,      source: last week's rhythm-of-business@1,     format: rhythm-of-business@1 }

outputs:
  - { name: revenue_cadence,    format: rhythm-of-business@1 (revenue chapter), recipient: cuo/cro-revenue + cuo/ceo + cuo/cso-sales + cuo/cco-customer }

skill_chain:
  - { step: 1, skill: rhythm-of-business-author, inputs_from: { pipeline: pipeline, cs_engagement_set: cs_engagement_set, prior_cadence: prior_cadence }, outputs_to: cadence_draft }
  - { step: 2, skill: rhythm-of-business-audit,  inputs_from: cadence_draft, outputs_to: revenue_cadence }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "pipeline coverage < 3x OR top-5 account at-risk" }
  - { persona: cuo/chief-financial-officer,         when: "revenue forecast at risk vs commit" }

consults:
  - { persona: cuo/chief-product-officer, when: "renewal-at-risk reason is product-driven" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with revenue_cadence hash + pipeline-coverage + at-risk-accounts count
  - HITL pause at step 2 on QA-RENEWAL-001 (at-risk renewal without intervention plan)
---

# Weekly revenue cadence — `chief-revenue-officer/weekly-revenue-cadence`

CRO-Revenue's consolidated weekly revenue operating cadence. Wraps CSO-Sales pipeline review with CS health + expansion + renewal-at-risk into one revenue-team view. Per Winning by Design RevOps + Bain customer-economics framework.

## When to invoke

- "Run the weekly revenue cadence"
- "Revenue operations review"
- "Where's the revenue this week"

## How to invoke

```bash
cyberos-cuo run cuo/chief-revenue-officer/weekly-revenue-cadence \
  --input pipeline=./sales/2026-W20/pipeline.md \
  --input cs_engagement_set=./customer/2026-W20/cs/ \
  --input prior_cadence=./revenue/2026-W19/cadence.md \
  --output-dir ./revenue/2026-W20/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + Friday afternoon roll-up
- **Worst case:** at-risk renewal triggers same-day intervention

## Skill chain

- **Step 1 `rhythm-of-business-author`** — drafts the revenue chapter of the operating rhythm.
- **Step 2 `rhythm-of-business-audit`** — validates per chapter-mode rules.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RENEWAL-001 | At-risk renewal no intervention | Operator drafts |
| 2 | QA-COVERAGE-001 | Pipeline coverage < 3x | Escalate to CEO |

## Cross-references
- `../../../../modules/cuo/README.md` §5.2 — CRO-Revenue role profile
- `../../chief-sales-officer/workflows/weekly-pipeline-review.md` — upstream chain
- `../../cco-customer/README.md` — customer-success peer
- `../../../skill/rhythm-of-business-{author,audit}/SKILL.md`
