---
workflow_id: chief-product-officer/annual-product-strategy
workflow_version: 1.0.0
purpose: Author the annual product strategy — vision, pillars, kill criteria, 18-month bets, north-star evolution.
persona: cuo/chief-product-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (product chapter), format: strategy-document@1 }
  - { name: market_intel,          source: cuo/cmo + competitive-brief, format: markdown }
  - { name: metrics_history,       source: 4 quarters of product-metrics-review@1, format: product-metrics-review@1 (4Q) }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: product_strategy,      format: strategy-doc@1, recipient: cuo/cpo-product + cuo/ceo + cuo/cto + cuo/cmo + Board (annual review) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, market_intel: market_intel, metrics_history: metrics_history, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: product_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes a pillar deprecation OR new pillar requiring significant capital" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "pillar shifts imply architecture / platform investment" }
  - { persona: cuo/chief-marketing-officer,            when: "positioning narrative shifts demand-gen" }
  - { persona: cuo/chief-ai-officer,           when: "strategy includes AI-native features (use-case portfolio overlap)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with product_strategy hash + pillar count + kill-criteria count
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt diagnosis/policy/action incomplete) or QA-KILL-001 (no kill criteria for new bets)
---

# Annual product strategy — `chief-product-officer/annual-product-strategy`

CPO-Product's annual product strategy. Per Marty Cagan + Roger Martin Playing-to-Win + Rumelt good-strategy kernel. Combines prior strategy + market intel + 4-quarter metrics history + CEO priorities into refreshed vision / pillars / kill-criteria / 18-month bets / north-star evolution.

## When to invoke

- "Build the 2026 product strategy"
- "Annual product strategic refresh"
- "Refresh product vision + pillars"

## How to invoke

```bash
cyberos-cuo run cuo/chief-product-officer/annual-product-strategy \
  --input prior_strategy=./product/2025/strategy.md \
  --input market_intel=./market/2026/intel.md \
  --input metrics_history=./product/2025/metrics/ \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./product/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function alignment + Board review
- **Worst case:** pillar deprecation requires 1-2 quarter rollout + customer comms

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Cagan + Roger Martin + Rumelt.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt kernel incomplete | Operator extends |
| 2 | QA-KILL-001 | No kill criteria | Operator drafts per bet |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CPO-Product role profile
- `./quarterly-roadmap-planning.md` — downstream consumer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
