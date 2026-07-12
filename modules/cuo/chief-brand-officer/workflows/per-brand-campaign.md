---
workflow_id: chief-brand-officer/per-brand-campaign
workflow_version: 1.0.0
purpose: Author a brand-led campaign (vs product/promo campaign) — equity-building objective, identity-system application, narrative + creative brief.
persona: cuo/chief-brand-officer
cadence: per-event
status: shipped

inputs:
  - { name: brand_strategy,        source: cuo/chief-brand-officer/annual-brand-strategy, format: brand-strategy@1 }
  - { name: campaign_brief,        source: brand requestor (CBO / CMO / CEO), format: markdown }
  - { name: brand_assets,          source: brand-asset library, format: markdown index }

outputs:
  - { name: brand_campaign_plan,   format: campaign-plan@1, recipient: cuo/chief-brand-officer + cuo/cmo + creative team }

skill_chain:
  - { step: 1, skill: campaign-plan-author, inputs_from: { brand_strategy: brand_strategy, campaign_brief: campaign_brief, brand_assets: brand_assets }, outputs_to: plan_draft }
  - { step: 2, skill: campaign-plan-audit,  inputs_from: plan_draft, outputs_to: brand_campaign_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "campaign proposes identity-system extension or new brand-architecture entry" }

consults:
  - { persona: cuo/chief-marketing-officer,            when: "demand-gen + measurement layer needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with brand_campaign_plan hash + identity-system coverage + brand-pillar alignment
  - HITL pause at step 2 on QA-PILLAR-001 (campaign misaligned with declared brand pillar)
---

# Per brand campaign — `chief-brand-officer/per-brand-campaign`

Chief Brand Officer's per-campaign workflow for brand-led (equity-building) campaigns — distinct from product/promo-led campaigns CMO owns. Equity objective drives creative; measurement is awareness + consideration + brand-attribute lift (not demand-gen).

## When to invoke

- "Build the [brand campaign name]"
- "Brand-led campaign for [theme]"
- "Equity-building campaign"

## How to invoke

```bash
cyberos-cuo run cuo/chief-brand-officer/per-brand-campaign \
  --input brand_strategy=./brand/2026/strategy.md \
  --input campaign_brief=./brand/campaigns/2026-equity-push/brief.md \
  --input brand_assets=./brand/assets/index.md \
  --output-dir ./brand/campaigns/2026-equity-push/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for creative + measurement design
- **Worst case:** identity-system extension adds 1-2 month

## Skill chain

- **Step 1 `campaign-plan-author`** — drafts brand-equity-focused campaign.
- **Step 2 `campaign-plan-audit`** — validates per `campaign_plan_rubric@1.0` with brand-pillar alignment check.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PILLAR-001 | Misaligned with brand pillar | Operator realigns or escalates |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — Chief Brand Officer role profile
- `./annual-brand-strategy.md` — upstream parent
- `../../chief-marketing-officer/workflows/per-campaign-plan.md` — product/promo peer
- `../../../skill/campaign-plan-{author,audit}/SKILL.md`
