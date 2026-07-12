---
workflow_id: chief-experience-officer/quarterly-customer-experience-review
workflow_version: 1.0.0
purpose: Review end-to-end customer experience — journey health, friction points, NPS by journey stage, experience-debt backlog.
persona: cuo/chief-experience-officer
cadence: quarterly
status: shipped

inputs:
  - { name: customer_health,       source: cuo/chief-customer-officer/quarterly-customer-health-review, format: customer-health-review@1 }
  - { name: nps_data,              source: cuo/chief-sales-officer/quarterly-nps-program, format: net-promoter-score-program@1 }
  - { name: product_metrics,       source: cuo/chief-product-officer/quarterly-product-metrics-review, format: product-metrics-review@1 }
  - { name: journey_telemetry,     source: customer-journey analytics, format: csv }

outputs:
  - { name: cx_review,             format: customer-health-review@1 (CX-augmented), recipient: cuo/cxo + cuo/cpo-product + cuo/cco-customer + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: customer-health-review-author, inputs_from: { customer_health: customer_health, nps_data: nps_data, product_metrics: product_metrics, journey_telemetry: journey_telemetry }, outputs_to: review_draft }
  - { step: 2, skill: customer-health-review-audit,  inputs_from: review_draft, outputs_to: cx_review }

escalates_to:
  - { persona: cuo/chief-product-officer,    when: "friction-point root cause is product-driven" }
  - { persona: cuo/chief-customer-officer,   when: "friction-point root cause is service-driven" }

audit_hooks:
  - workflow_complete row on PASS with cx_review hash
  - HITL pause at step 2 on QA-JOURNEY-001 (journey-stage attribution unclear)
---

# Quarterly customer experience review — `chief-experience-officer/quarterly-customer-experience-review`

CXO's end-to-end CX review per Forrester CX Index + Temkin (Qualtrics XM) framework + KKM Experience Pyramid.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CXO role profile
- `../../chief-customer-officer/workflows/quarterly-customer-health-review.md` — peer (CXO is journey-wide; CCO-Customer is account-tier)
- `../../../skill/customer-health-review-{author,audit}/SKILL.md`
