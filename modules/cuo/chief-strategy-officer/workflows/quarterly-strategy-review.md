---
workflow_id: chief-strategy-officer/quarterly-strategy-review
workflow_version: 1.0.0
purpose: Quarterly strategy review — progress on annual bets, environment monitoring, course corrections, board chapter.
persona: cuo/chief-strategy-officer
cadence: quarterly
status: shipped

inputs:
  - { name: corporate_strategy,    source: cuo/chief-strategy-officer/annual-corporate-strategy, format: strategy-document@1 }
  - { name: bet_progress,          source: per-bet sponsor updates, format: markdown briefs }
  - { name: market_signals,        source: cuo/cmo competitive-brief, format: markdown }

outputs:
  - { name: strategy_review,       format: strategy-document@1 (quarterly chapter), recipient: cuo/cso-strategy + cuo/ceo + Board (strategy chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { corporate_strategy: corporate_strategy, bet_progress: bet_progress, market_signals: market_signals }, outputs_to: review_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: review_draft, outputs_to: strategy_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy review recommends a strategic pivot OR kill of an active bet" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "rebalancing requires capital reallocation" }
  - { persona: cuo/chief-product-officer,    when: "product implications" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with strategy_review hash + progress summary + course-corrections count
  - HITL pause at step 2 on QA-PIVOT-001 (pivot recommended without environment-change evidence)
---

# Quarterly strategy review — `chief-strategy-officer/quarterly-strategy-review`

CSO-Strategy's quarterly strategy review per Rumelt strategy-as-a-living-discipline + Mintzberg emergent-strategy framework. Tracks annual bets, monitors environment, recommends course corrections.

## When to invoke

- "Run the Q<n> strategy review"
- "Quarterly strategy check"
- "Strategy progress review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-strategy-officer/quarterly-strategy-review \
  --input corporate_strategy=./strategy/2026/corporate.md \
  --input bet_progress=./strategy/2026-Q1/bet-status/ \
  --input market_signals=./market/2026-Q1/intel.md \
  --output-dir ./strategy/2026-Q1/review/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for sponsor round-trip
- **Worst case:** strategic pivot triggers full strategy revision (1 quarter)

## Skill chain

- **Step 1 `strategy-document-author`** — drafts quarterly chapter.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PIVOT-001 | Pivot no environment evidence | Operator strengthens or de-escalates |

## Cross-references
- `../../../../modules/cuo/README.md` §5.1 — CSO-Strategy role profile
- `./annual-corporate-strategy.md` — upstream parent
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
