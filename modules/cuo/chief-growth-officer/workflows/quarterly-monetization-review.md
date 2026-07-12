---
workflow_id: chief-growth-officer/quarterly-monetization-review
workflow_version: 1.0.0
purpose: Review monetization performance — pricing experiments, plan adoption, expansion ARPU, packaging health.
persona: cuo/chief-growth-officer
cadence: quarterly
status: shipped

inputs:
  - { name: revenue_review,        source: cuo/chief-revenue-officer/quarterly-revenue-review, format: board-deck@1 chapter }
  - { name: pricing_experiments,   source: pricing/packaging experiment log, format: markdown }
  - { name: customer_signals,      source: cuo/chief-customer-officer/quarterly-customer-health-review verbatims on pricing, format: customer-health-review@1 chapter }

outputs:
  - { name: monetization_review,   format: go-to-market-plan@1 (monetization chapter), recipient: cuo/cgo + cuo/cpo-product + cuo/cro-revenue + cuo/cfo }

skill_chain:
  - { step: 1, skill: go-to-market-plan-author, inputs_from: { revenue_review: revenue_review, pricing_experiments: pricing_experiments, customer_signals: customer_signals }, outputs_to: review_draft }
  - { step: 2, skill: go-to-market-plan-audit,  inputs_from: review_draft, outputs_to: monetization_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "review recommends pricing-model change" }

audit_hooks:
  - workflow_complete row on PASS with monetization_review hash
  - HITL pause at step 2 on QA-PRICING-001
---

# Quarterly monetization review — `chief-growth-officer/quarterly-monetization-review`

CGO's quarterly monetization review per Profitwell + OpenView pricing-research + Madhavan Ramanujam Monetizing Innovation framework.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.1
- `../../chief-revenue-officer/workflows/quarterly-revenue-review.md` — upstream peer
- `../../../skill/gtm-plan-{author,audit}/SKILL.md`
