---
workflow_id: chief-innovation-officer/annual-innovation-portfolio
workflow_version: 1.0.0
purpose: Author the annual innovation portfolio — horizons 1/2/3, investment thesis per bet, stage-gates, kill criteria.
persona: cuo/chief-innovation-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_portfolio,       source: last year's innovation-portfolio@1, format: innovation-portfolio@1 }
  - { name: bet_status,            source: per-bet stage-gate updates, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: market_signals,        source: competitive-brief + adjacent-market intel, format: markdown }

outputs:
  - { name: innovation_portfolio,  format: innovation-portfolio@1, recipient: cuo/chief-innovation-officer + cuo/ceo + cuo/cpo-product + Board (annual innovation chapter) }

skill_chain:
  - { step: 1, skill: innovation-portfolio-author, inputs_from: { prior_portfolio: prior_portfolio, bet_status: bet_status, ceo_priorities: ceo_priorities, market_signals: market_signals }, outputs_to: portfolio_draft }
  - { step: 2, skill: innovation-portfolio-audit,  inputs_from: portfolio_draft, outputs_to: innovation_portfolio }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "portfolio rebalance shifts Horizon-3 budget > 20% YoY OR proposes kill of legacy bet" }

consults:
  - { persona: cuo/chief-product-officer,    when: "Horizon-1/2 bets intersect roadmap" }
  - { persona: cuo/chief-financial-officer,            when: "envelope shift requires reallocation" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with innovation_portfolio hash + per-horizon bet count + kill-criteria coverage
  - HITL pause at step 2 on QA-KILL-001 (bet without kill criteria) or QA-HORIZON-001 (horizon imbalance >70/20/10 industry guideline)
---

# Annual innovation portfolio — `chief-innovation-officer/annual-innovation-portfolio`

Chief Innovation Officer's annual portfolio per McKinsey Three Horizons + Govindarajan Ten Types of Innovation + Christensen disruption framework. Tracks Horizon-1 (core), Horizon-2 (adjacencies), Horizon-3 (transformational) bets with explicit kill criteria.

## When to invoke

- "Build the 2026 innovation portfolio"
- "Annual innovation strategic refresh"
- "Refresh horizons portfolio"

## How to invoke

```bash
cyberos-cuo run cuo/chief-innovation-officer/annual-innovation-portfolio \
  --input prior_portfolio=./innovation/2025/portfolio.md \
  --input bet_status=./innovation/2025/bets/ \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input market_signals=./market/2026/intel.md \
  --output-dir ./innovation/2026/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-8 weeks for cross-function + Board review
- **Worst case:** Horizon-3 budget shift triggers strategic-plan revision

## Skill chain

- **Step 1 `innovation-portfolio-author`** — drafts per McKinsey Three Horizons + Govindarajan + Christensen.
- **Step 2 `innovation-portfolio-audit`** — validates per `innovation_portfolio_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KILL-001 | Bet no kill criteria | Operator drafts |
| 2 | QA-HORIZON-001 | Horizon imbalance | Operator rebalances or justifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Innovation Officer role profile
- `../../chief-product-officer/workflows/annual-product-strategy.md` — H1 peer
- `../../../skill/innovation-portfolio-{author,audit}/SKILL.md`
