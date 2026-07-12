---
workflow_id: chief-data-officer/annual-data-strategy
workflow_version: 1.0.0
purpose: Author the annual data strategy — domains, data products, governance, infrastructure, team operating model.
persona: cuo/chief-data-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's data-strategy@1,                format: data-strategy@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief),                     format: markdown }
  - { name: data_product_inventory, source: cuo/cdo-data data-product register,        format: csv export }
  - { name: governance_state,      source: data-governance@1 (annual review),          format: data-governance@1 }

outputs:
  - { name: data_strategy,         format: data-strategy@1, recipient: cuo/cdo-data + cuo/ceo + cuo/cto + cuo/caio + Board (annual data chapter) }

skill_chain:
  - { step: 1, skill: data-strategy-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, data_product_inventory: data_product_inventory, governance_state: governance_state }, outputs_to: strategy_draft }
  - { step: 2, skill: data-strategy-audit,  inputs_from: strategy_draft, outputs_to: data_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes platform consolidation/replacement OR new data-mesh adoption" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "infrastructure investments need platform alignment" }
  - { persona: cuo/chief-ai-officer,           when: "data foundations for AI/ML use cases" }
  - { persona: cuo/chief-privacy-officer,    when: "domain decomposition impacts privacy controls" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with data_strategy hash + domain count + data-product count + governance-maturity level
  - HITL pause at step 2 on QA-DOMAIN-001 (domain ownership ambiguous) or QA-MESH-001 (mesh principles not consistently applied)
---

# Annual data strategy — `chief-data-officer/annual-data-strategy`

CDO-Data's annual data strategy. Per DAMA-DMBOK + Data Mesh (Zhamak Dehghani) + Modern Data Stack patterns. Combines prior strategy + CEO priorities + data-product inventory + governance state into domains / data products / governance / infrastructure / team operating model.

## When to invoke

- "Build the 2026 data strategy"
- "Annual data strategic refresh"
- "Data strategy review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-data-officer/annual-data-strategy \
  --input prior_strategy=./data/2025/strategy.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input data_product_inventory=./data/2026/products.csv \
  --input governance_state=./data/2026/governance.md \
  --output-dir ./data/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function alignment + Board review
- **Worst case:** platform consolidation may require multi-year roadmap

## Skill chain

- **Step 1 `data-strategy-author`** — drafts per DAMA-DMBOK + Data Mesh + Modern Data Stack.
- **Step 2 `data-strategy-audit`** — validates per `data_strategy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-DOMAIN-001 | Ownership ambiguous | Operator assigns |
| 2 | QA-MESH-001 | Mesh principles inconsistent | Operator reconciles |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CDO-Data role profile
- `../../chief-ai-officer/workflows/annual-ai-strategy.md` — peer (data foundations feed AI)
- `../../../skill/data-strategy-{author,audit}/SKILL.md`
