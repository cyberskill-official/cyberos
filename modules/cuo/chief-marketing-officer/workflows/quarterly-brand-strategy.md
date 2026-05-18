---
workflow_id: chief-marketing-officer/quarterly-brand-strategy
workflow_version: 1.0.0
purpose: Refresh the quarterly brand strategy — positioning, narrative, messaging architecture, audience segmentation.
persona: cuo/chief-marketing-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_brand_strategy,  source: last quarter's brand-strategy@1, format: brand-strategy@1 }
  - { name: gtm_plan,              source: cuo/chief-sales-officer/annual-gtm-plan, format: go-to-market-plan@1 }
  - { name: competitive_intel,     source: competitive-brief outputs, format: markdown }
  - { name: customer_research,     source: cuo/cco-customer + CAB synthesis, format: customer-advisory-board@1 + research notes }

outputs:
  - { name: brand_strategy,        format: brand-strategy@1, recipient: cuo/cmo + cuo/chief-brand-officer + cuo/ceo + cuo/cso-sales }

skill_chain:
  - { step: 1, skill: brand-strategy-author, inputs_from: { prior_brand_strategy: prior_brand_strategy, gtm_plan: gtm_plan, competitive_intel: competitive_intel, customer_research: customer_research }, outputs_to: strategy_draft }
  - { step: 2, skill: brand-strategy-audit,  inputs_from: strategy_draft, outputs_to: brand_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes positioning shift OR new audience tier" }

consults:
  - { persona: cuo/chief-product-officer,    when: "positioning needs product-feature anchor" }
  - { persona: cuo/chief-communications-officer, when: "narrative requires PR rollout" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with brand_strategy hash + audience-tier count + messaging-pillar count
  - HITL pause at step 2 on QA-POSITION-001 (positioning conflicts with competitive intel)
---

# Quarterly brand strategy — `chief-marketing-officer/quarterly-brand-strategy`

CMO's quarterly brand-strategy refresh per Aaker brand-equity model + BAV BrandAsset Valuator + Marty Neumeier brand-gap framework. Refreshes positioning / narrative / messaging architecture / audience segmentation against GTM + competitive + customer signals.

## When to invoke

- "Refresh the Q<n> brand strategy"
- "Quarterly brand review"
- "Update positioning"

## How to invoke

```bash
cyberos-cuo run cuo/chief-marketing-officer/quarterly-brand-strategy \
  --input prior_brand_strategy=./brand/2026-Q1/strategy.md \
  --input gtm_plan=./gtm/2026/plan.md \
  --input competitive_intel=./market/2026-Q2/competitive.md \
  --input customer_research=./customer/2026-Q1/cab.md \
  --output-dir ./brand/2026-Q2/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for cross-function alignment
- **Worst case:** positioning shift triggers asset-refresh cycle (1-2 quarter)

## Skill chain

- **Step 1 `brand-strategy-author`** — drafts per Aaker + BAV + Neumeier.
- **Step 2 `brand-strategy-audit`** — validates per `brand_strategy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-POSITION-001 | Positioning conflicts intel | Operator reconciles |
| 2 | QA-AUDIENCE-001 | Audience tier overlaps | Operator clarifies |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — CMO role profile
- `../../chief-brand-officer/README.md` — partner persona where exists
- `../../../skill/brand-strategy-{author,audit}/SKILL.md`
