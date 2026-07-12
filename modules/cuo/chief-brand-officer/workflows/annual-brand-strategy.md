---
workflow_id: chief-brand-officer/annual-brand-strategy
workflow_version: 1.0.0
purpose: Author the annual brand strategy — brand architecture, equity model, identity system, brand pillars, audience tiering.
persona: cuo/chief-brand-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's brand-strategy@1, format: brand-strategy@1 }
  - { name: brand_research,        source: brand-tracking studies (BAV / Interbrand / Y&R), format: markdown }
  - { name: competitive_intel,     source: cuo/cmo competitive-brief, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: brand_strategy,        format: brand-strategy@1, recipient: cuo/chief-brand-officer + cuo/cmo + cuo/ceo + Board (annual brand chapter) }

skill_chain:
  - { step: 1, skill: brand-strategy-author, inputs_from: { prior_strategy: prior_strategy, brand_research: brand_research, competitive_intel: competitive_intel, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: brand-strategy-audit,  inputs_from: strategy_draft, outputs_to: brand_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes architecture change (house-of-brands ↔ branded-house)" }

consults:
  - { persona: cuo/chief-marketing-officer,            when: "campaign-implications need marketing depth" }
  - { persona: cuo/chief-customer-officer,   when: "audience tiering intersects customer-segmentation" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with brand_strategy hash + pillar count + audience-tier count
  - HITL pause at step 2 on QA-EQUITY-001 (brand-equity metrics not measurable)
---

# Annual brand strategy — `chief-brand-officer/annual-brand-strategy`

Chief Brand Officer's annual strategy per Aaker brand-equity + BAV BrandAsset Valuator + Interbrand framework. Sister workflow to CMO's quarterly brand-strategy refresh — CBO owns architecture + identity; CMO owns campaign expression.

## When to invoke

- "Build the 2026 brand strategy"
- "Annual brand-architecture review"
- "Refresh brand pillars + identity"

## How to invoke

```bash
cyberos-cuo run cuo/chief-brand-officer/annual-brand-strategy \
  --input prior_strategy=./brand/2025/strategy.md \
  --input brand_research=./brand/2026/research.md \
  --input competitive_intel=./market/2026/competitive.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./brand/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 8-12 weeks for cross-function + Board review
- **Worst case:** architecture shift triggers identity-system redesign (1-2 quarter)

## Skill chain

- **Step 1 `brand-strategy-author`** — drafts per Aaker + BAV + Interbrand.
- **Step 2 `brand-strategy-audit`** — validates per `brand_strategy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-EQUITY-001 | Equity metric not measurable | Operator quantifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — Chief Brand Officer role profile
- `../../chief-marketing-officer/workflows/quarterly-brand-strategy.md` — campaign-side peer
- `../../../skill/brand-strategy-{author,audit}/SKILL.md`
