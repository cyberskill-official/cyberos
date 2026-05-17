---
workflow_id: chief-growth-officer/annual-growth-strategy
workflow_version: 1.0.0
purpose: Author the annual growth strategy — north-star metric, growth loops, channel mix (PLG/SLG/CLG), monetization model.
persona: cuo/chief-growth-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's gtm-plan@1 (growth chapter), format: gtm-plan@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: product_strategy,      source: cuo/chief-product-officer/annual-product-strategy, format: strategy-doc@1 }

outputs:
  - { name: growth_strategy,       format: gtm-plan@1, recipient: cuo/cgo + cuo/cmo + cuo/cpo-product + cuo/cso-sales + cuo/ceo + Board (growth chapter) }

skill_chain:
  - { step: 1, skill: gtm-plan-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, product_strategy: product_strategy }, outputs_to: strategy_draft }
  - { step: 2, skill: gtm-plan-audit,  inputs_from: strategy_draft, outputs_to: growth_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes monetization-model shift" }

consults:
  - { persona: cuo/chief-product-officer,    when: "PLG strategy requires product surface changes" }

audit_hooks:
  - workflow_complete row on PASS with growth_strategy hash + growth-loops count
  - HITL pause at step 2 on QA-LOOP-001 or QA-NORTHSTAR-001
---

# Annual growth strategy — `chief-growth-officer/annual-growth-strategy`

CGO's annual strategy per Reforge growth-strategy + OpenView PLG + Brian Balfour 4-Fits framework (product-market / product-channel / channel-model / model-market).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.1
- `../../chief-sales-officer/workflows/annual-gtm-plan.md` — sales-led peer
- `../../../skill/gtm-plan-{author,audit}/SKILL.md`
