---
workflow_id: chief-product-officer/quarterly-product-metrics-review
workflow_version: 1.0.0
purpose: Author the quarterly product OKR + metrics review — DAU/WAU/MAU, retention cohorts, feature adoption, north-star movement.
persona: cuo/chief-product-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_review,          source: last quarter's product-metrics-review@1, format: product-metrics-review@1 }
  - { name: analytics_export,      source: Amplitude / Mixpanel / Pendo / Heap,    format: csv / json export }
  - { name: okr_tracker,           source: Lattice / Ally / Workboard product-OKR view, format: markdown / csv }
  - { name: cohort_definitions,    source: cuo/cpo-product (current cohort definitions), format: markdown }

outputs:
  - { name: product_metrics_review, format: product-metrics-review@1, recipient: cuo/cpo-product + cuo/ceo + Board (product chapter) }

skill_chain:
  - { step: 1, skill: product-metrics-review-author, inputs_from: { prior_review: prior_review, analytics_export: analytics_export, okr_tracker: okr_tracker, cohort_definitions: cohort_definitions }, outputs_to: review_draft }
  - { step: 2, skill: product-metrics-review-audit,  inputs_from: review_draft, outputs_to: product_metrics_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "north-star metric drops >10% QoQ OR OKR attainment <50%" }

consults:
  - { persona: cuo/chief-data-officer,       when: "cohort definitions need data-governance review" }
  - { persona: cuo/chief-revenue-officer,    when: "metric trends correlate with revenue-side health" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with product_metrics_review hash + north-star delta + OKR attainment %
  - HITL pause at step 2 on QA-COHORT-001 (cohort math inconsistent with definition)
---

# Quarterly product metrics review — `chief-product-officer/quarterly-product-metrics-review`

CPO-Product's quarterly product OKR + metrics review. Per Amplitude / Mixpanel / Pendo product-analytics best practices + Reforge + Sequoia PLG. Critical for product-led companies; central to board product-chapter. Feeds the quarterly-roadmap-planning workflow.

## When to invoke

- "Run the Q<n> product metrics review"
- "Product OKR review"
- "DAU/MAU/retention check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-product-officer/quarterly-product-metrics-review \
  --input prior_review=./product/2026-Q1/metrics-review.md \
  --input analytics_export=./analytics/2026-Q1/amplitude.csv \
  --input okr_tracker=./product/2026-Q1/okrs.md \
  --input cohort_definitions=./product/cohorts.md \
  --output-dir ./product/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for cross-function review
- **Worst case:** north-star drop triggers cross-functional intervention

## Skill chain

- **Step 1 `product-metrics-review-author`** — drafts per Amplitude + Mixpanel + Reforge frameworks.
- **Step 2 `product-metrics-review-audit`** — validates per `product_metrics_review_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-COHORT-001 | Cohort math inconsistent | Operator reconciles |
| 2 | QA-NORTHSTAR-001 | North-star drop unexplained | Escalate to CEO |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CPO-Product role profile
- `./quarterly-roadmap-planning.md` — downstream consumer
- `../../../skill/product-metrics-review-{author,audit}/SKILL.md`
