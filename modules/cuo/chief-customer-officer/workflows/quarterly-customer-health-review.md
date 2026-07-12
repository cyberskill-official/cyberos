---
workflow_id: chief-customer-officer/quarterly-customer-health-review
workflow_version: 1.0.0
purpose: Author the quarterly customer-health review across active customer base — health distribution, at-risk list, expansion + advocacy pipelines.
persona: cuo/chief-customer-officer
cadence: quarterly
status: shipped
pattern: persona_pair
peer_persona: chief-data-officer
peer_workflow: annual-customer-360-architecture
shared_artefact: customer-profile
handoff_step: 2

inputs:
  - { name: prior_review,          source: last quarter's customer-health-review@1, format: customer-health-review@1 }
  - { name: cs_platform_export,    source: Gainsight / Catalyst / ChurnZero, format: csv }
  - { name: product_usage_data,    source: Amplitude / Mixpanel per-account telemetry, format: csv }
  - { name: csm_book_data,         source: CS team capacity tool, format: csv (CSM × account assignments) }

outputs:
  - { name: customer_health_review, format: customer-health-review@1, recipient: cuo/cco-customer + cuo/cro-revenue + cuo/ceo + Board (chapter) }

skill_chain:
  - { step: 1, skill: customer-health-review-author, inputs_from: { prior_review: prior_review, cs_platform_export: cs_platform_export, product_usage_data: product_usage_data, csm_book_data: csm_book_data }, outputs_to: review_draft }
  - { step: 2, skill: customer-health-review-audit,  inputs_from: review_draft, outputs_to: customer_health_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "% green-rated < 60% OR top-10 account at-risk" }
  - { persona: cuo/chief-revenue-officer,    when: "expansion-pipeline conversion < 15% (Bain benchmark)" }

consults:
  - { persona: cuo/chief-product-officer,    when: "at-risk accounts cite product issues" }
  - { persona: cuo/chief-human-resources-officer,           when: "CSM utilization > 110% — burnout risk" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with customer_health_review hash + %-green + at-risk count + expansion-ARR
  - HITL pause at step 2 on QA-INTERVENTION-001 (at-risk no intervention plan) or QA-COHORT-001 (cohort transitions unexplained)
---

# Quarterly customer health review — `chief-customer-officer/quarterly-customer-health-review`

CCO-Customer's quarterly customer-health workflow. Per Gainsight Customer Success Operating Model + Catalyst CS Ops + TSIA CS benchmarks + Bessemer Cloud Index. Distinct from `chief-revenue-officer/quarterly-churn-analysis` (backward on churners) — this is forward-looking on ACTIVE customers.

## When to invoke

- "Run the Q<n> customer health review"
- "Customer-health pulse"
- "Who's at risk + who's expanding"

## How to invoke

```bash
cyberos-cuo run cuo/chief-customer-officer/quarterly-customer-health-review \
  --input prior_review=./customer/2026-Q1/health.md \
  --input cs_platform_export=./customer/2026-Q1/gainsight.csv \
  --input product_usage_data=./customer/2026-Q1/usage.csv \
  --input csm_book_data=./cs/2026-Q1/books.csv \
  --output-dir ./customer/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for CSM input
- **Worst case:** top-10 at-risk triggers CEO-level intervention

## Skill chain

- **Step 1 `customer-health-review-author`** — drafts per Gainsight + Catalyst + TSIA.
- **Step 2 `customer-health-review-audit`** — validates per `customer_health_review_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-INTERVENTION-001 | At-risk no plan | Operator drafts |
| 2 | QA-COHORT-001 | Cohort transition unexplained | Operator adds narrative |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Customer role profile
- `../../chief-revenue-officer/workflows/quarterly-churn-analysis.md` — peer (churn = backward, health = forward)
- `../../../skill/customer-health-review-{author,audit}/SKILL.md`
