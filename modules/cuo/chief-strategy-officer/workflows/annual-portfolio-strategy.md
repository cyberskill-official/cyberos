---
workflow_id: chief-strategy-officer/annual-portfolio-strategy
workflow_version: 1.0.0
purpose: Refresh the business-unit / product-line portfolio — invest / hold / harvest / divest decisions with strategic rationale.
persona: cuo/chief-strategy-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_portfolio,       source: last year's portfolio chapter of strategy-doc@1, format: strategy-document@1 }
  - { name: bu_performance,        source: BU / product-line P&L + growth metrics, format: markdown }
  - { name: market_attractiveness, source: industry analyses per BU, format: markdown }

outputs:
  - { name: portfolio_strategy,    format: strategy-document@1 (portfolio chapter), recipient: cuo/cso-strategy + cuo/ceo + cuo/cfo + Board (annual portfolio review) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_portfolio: prior_portfolio, bu_performance: bu_performance, market_attractiveness: market_attractiveness }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: portfolio_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "recommends divestiture or major reorg" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "divestiture economics" }
  - { persona: cuo/chief-product-officer,    when: "product-line implications" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with portfolio_strategy hash + per-BU classification + divestiture flag
  - HITL pause at step 2 on QA-CLASSIFICATION-001 (BU classification without rationale) or QA-DIVESTITURE-001 (divestiture without integration/exit plan)
---

# Annual portfolio strategy — `chief-strategy-officer/annual-portfolio-strategy`

CSO-Strategy's annual portfolio-strategy refresh per BCG Growth-Share Matrix + GE-McKinsey 9-box + Boston BCG Strategic Portfolio Analysis. Classifies each BU / product-line as invest / hold / harvest / divest.

## When to invoke

- "Refresh the 2026 portfolio strategy"
- "Annual portfolio review"
- "Invest/hold/harvest/divest decisions"

## How to invoke

```bash
cyberos-cuo run cuo/chief-strategy-officer/annual-portfolio-strategy \
  --input prior_portfolio=./strategy/2025/portfolio.md \
  --input bu_performance=./portfolio/2026/performance.md \
  --input market_attractiveness=./market/2026/per-bu.md \
  --output-dir ./strategy/2026/portfolio/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for BU input + Board review
- **Worst case:** divestiture triggers 1-2 quarter execution

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per BCG + GE-McKinsey 9-box.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CLASSIFICATION-001 | Classification no rationale | Operator drafts |
| 2 | QA-DIVESTITURE-001 | Divestiture no exit plan | Operator extends |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.1 — CSO-Strategy role profile
- `./annual-corporate-strategy.md` — parent strategy
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
