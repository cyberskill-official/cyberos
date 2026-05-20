---
workflow_id: chief-strategy-officer/annual-corporate-strategy
workflow_version: 1.0.0
purpose: Author the annual corporate strategy — diagnosis, guiding policy, coherent actions, where-to-play, how-to-win.
persona: cuo/chief-strategy-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1, format: strategy-document@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: market_intel,          source: industry + competitive intel, format: markdown }
  - { name: portfolio_state,       source: business-unit performance + product/innovation portfolios, format: markdown }

outputs:
  - { name: corporate_strategy,    format: strategy-doc@1, recipient: cuo/cso-strategy + cuo/ceo + entire C-suite + Board (annual strategy) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, market_intel: market_intel, portfolio_state: portfolio_state }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: corporate_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes exit-a-market, enter-a-market, M&A initiative" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "capital-allocation implications" }
  - { persona: cuo/chief-product-officer,    when: "product-portfolio implications" }
  - { persona: cuo/chief-marketing-officer,            when: "positioning implications" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with corporate_strategy hash + bets count + kernel completeness
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt diagnosis/policy/action incomplete) or QA-WTP-001 (where-to-play boundaries vague)
---

# Annual corporate strategy — `chief-strategy-officer/annual-corporate-strategy`

CSO-Strategy's annual corporate strategy per Rumelt good-strategy kernel + Roger Martin Playing-to-Win + Porter Five Forces + Christensen disruption. The master document for "where do we play and how do we win" cascading into all other strategies.

## When to invoke

- "Build the 2026 corporate strategy"
- "Annual strategic refresh"
- "Refresh corporate strategy"

## How to invoke

```bash
cyberos-cuo run cuo/chief-strategy-officer/annual-corporate-strategy \
  --input prior_strategy=./strategy/2025/corporate.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input market_intel=./market/2026/intel.md \
  --input portfolio_state=./portfolio/2026/state.md \
  --output-dir ./strategy/2026/corporate/
```

## Expected duration

- **Happy path:** 16-32 hours runtime + 3-6 months for cross-function + Board review
- **Worst case:** market-exit / entry decision triggers multi-year execution

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Rumelt + Roger Martin + Porter + Christensen.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt kernel incomplete | Operator extends |
| 2 | QA-WTP-001 | Where-to-play vague | Operator tightens |

## Cross-references
- `../../../../modules/cuo/README.md` §5.1 — CSO-Strategy role profile
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — board-deck strategy chapter
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
