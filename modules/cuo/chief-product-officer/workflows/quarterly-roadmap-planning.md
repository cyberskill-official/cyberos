---
workflow_id: chief-product-officer/quarterly-roadmap-planning
workflow_version: 1.0.0
purpose: Refresh the quarterly product roadmap — opportunity-solution tree, prioritization, capacity vs commitment, dependency mapping.
persona: cuo/chief-product-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_roadmap,         source: last quarter's product-roadmap@1, format: product-roadmap@1 }
  - { name: customer_feedback,     source: cuo/cco-customer (CAB outputs + NPS verbatims), format: markdown / customer-advisory-board@1 }
  - { name: metrics_review,        source: cuo/chief-product-officer/quarterly-product-metrics-review, format: product-metrics-review@1 }
  - { name: capacity_signal,       source: cuo/cto (engineering capacity per quarter), format: markdown }

outputs:
  - { name: product_roadmap,       format: product-roadmap@1, recipient: cuo/cpo-product + cuo/cto + cuo/cmo + cuo/cso-sales + Board (quarterly chapter) }

skill_chain:
  - { step: 1, skill: product-roadmap-author, inputs_from: { prior_roadmap: prior_roadmap, customer_feedback: customer_feedback, metrics_review: metrics_review, capacity_signal: capacity_signal }, outputs_to: roadmap_draft }
  - { step: 2, skill: product-roadmap-audit,  inputs_from: roadmap_draft, outputs_to: product_roadmap }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "roadmap proposes a strategic-direction shift (e.g. new pillar, deprecated pillar)" }
  - { persona: cuo/chief-technology-officer,            when: "capacity over-commitment > 110% of declared engineering capacity" }

consults:
  - { persona: cuo/chief-sales-officer,      when: "roadmap items have commit-dependent revenue implications" }
  - { persona: cuo/chief-marketing-officer,            when: "roadmap shifts demand-gen positioning" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with product_roadmap hash + committed-vs-stretch ratio
  - HITL pause at step 2 on QA-CAPACITY-001 (overcommit) or QA-OPPORTUNITY-001 (commitment without underlying opportunity)
---

# Quarterly roadmap planning — `chief-product-officer/quarterly-roadmap-planning`

CPO-Product's quarterly roadmap-refresh workflow. Per Marty Cagan / SVPG opportunity-solution-tree (Teresa Torres) + Reforge product-strategy. Drives the commit/stretch split for the next quarter with explicit dependency mapping and capacity-vs-commitment audit.

## When to invoke

- "Refresh the Q<n> product roadmap"
- "Quarterly product planning"
- "Plan the next quarter's product work"

## How to invoke

```bash
cyberos-cuo run cuo/chief-product-officer/quarterly-roadmap-planning \
  --input prior_roadmap=./product/2026-Q1/roadmap.md \
  --input customer_feedback=./customer/2026-Q1/cab-output.md \
  --input metrics_review=./product/2026-Q1/metrics-review.md \
  --input capacity_signal=./engineering/2026-Q2/capacity.md \
  --output-dir ./product/2026-Q2/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for engineering + design + CS + sales cross-input
- **Worst case:** capacity-vs-commitment mismatch requires re-cut + re-discovery — 1-2 quarter slip

## Skill chain

- **Step 1 `product-roadmap-author`** — drafts per Marty Cagan OST + Reforge product-strategy.
- **Step 2 `product-roadmap-audit`** — validates per `product_roadmap_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CAPACITY-001 | Overcommit | Operator phases / defers |
| 2 | QA-OPPORTUNITY-001 | Commitment without opportunity tree | Operator discovers or drops |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CPO-Product role profile
- `../../chief-technology-officer/workflows/architect-new-system.md` — downstream feeder (PRD → SRS → ADR chain)
- `../../../skill/product-roadmap-{author,audit}/SKILL.md`
