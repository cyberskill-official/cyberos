---
workflow_id: chief-brand-officer/per-analyst-brand-briefing
workflow_version: 1.0.0
purpose: Brief brand-research firms (Interbrand / Y&R / BAV) on year-over-year brand position — feeds annual brand-ranking studies.
persona: cuo/chief-brand-officer
cadence: annual
status: shipped

inputs:
  - { name: brand_strategy,        source: cuo/chief-brand-officer/annual-brand-strategy, format: brand-strategy@1 }
  - { name: prior_briefing,        source: last year's analyst-briefing@1 (brand-research analyst angle), format: analyst-briefing@1 }
  - { name: brand_metrics,         source: brand-tracking studies + earned media + share of voice, format: markdown }

outputs:
  - { name: brand_analyst_briefing, format: analyst-briefing@1, recipient: cuo/chief-brand-officer + Interbrand / Y&R / BAV analysts + cuo/cmo }

skill_chain:
  - { step: 1, skill: analyst-briefing-author, inputs_from: { brand_strategy: brand_strategy, prior_briefing: prior_briefing, brand_metrics: brand_metrics }, outputs_to: briefing_draft }
  - { step: 2, skill: analyst-briefing-audit,  inputs_from: briefing_draft, outputs_to: brand_analyst_briefing }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "briefing surfaces brand-value decline >5% YoY" }

consults:
  - { persona: cuo/chief-communications-officer, when: "briefing tied to earned-media campaign" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with brand_analyst_briefing hash + analyst-firm-target + brand-attribute deltas
  - HITL pause at step 2 on QA-METRIC-001 (claim lacks tracking-study source)
---

# Per analyst brand briefing — `chief-brand-officer/per-analyst-brand-briefing`

Chief Brand Officer's annual briefing to brand-research firms (Interbrand Best Global Brands / Y&R BrandAsset Valuator / Kantar BrandZ). Distinct from CMO's product/AR analyst briefing.

## When to invoke

- "Brief Interbrand for [year] ranking"
- "Brand-analyst briefing"
- "Annual brand-tracking submission"

## How to invoke

```bash
cyberos-cuo run cuo/chief-brand-officer/per-analyst-brand-briefing \
  --input brand_strategy=./brand/2026/strategy.md \
  --input prior_briefing=./brand/analyst/2025/briefing.md \
  --input brand_metrics=./brand/2026/metrics.md \
  --output-dir ./brand/analyst/2026/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-3 weeks for analyst-firm questionnaire cycle
- **Worst case:** brand-value decline triggers strategic response cycle

## Skill chain

- **Step 1 `analyst-briefing-author`** — drafts brand-analyst variant.
- **Step 2 `analyst-briefing-audit`** — validates per `analyst_briefing_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-METRIC-001 | Claim no tracking source | Operator supplies |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — Chief Brand Officer role profile
- `../../chief-marketing-officer/workflows/quarterly-analyst-briefing.md` — product/AR peer
- `../../../skill/analyst-briefing-{author,audit}/SKILL.md`
