---
workflow_id: chief-innovation-officer/annual-innovation-strategy
workflow_version: 1.0.0
purpose: Author the annual innovation strategy — moonshot vision, innovation operating model, partnership thesis, innovation OKRs.
persona: cuo/chief-innovation-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (innovation chapter), format: strategy-document@1 }
  - { name: portfolio,             source: cuo/chief-innovation-officer/annual-innovation-portfolio, format: innovation-portfolio@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: innovation_strategy,   format: strategy-doc@1, recipient: cuo/chief-innovation-officer + cuo/ceo + Board (annual innovation chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, portfolio: portfolio, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: innovation_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes new innovation operating model (CVC arm, accelerator, M&A engine)" }

consults:
  - { persona: cuo/chief-strategy-officer,   when: "M&A pathways for innovation acquisition" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with innovation_strategy hash + moonshot count + OKR target
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt incomplete)
---

# Annual innovation strategy — `chief-innovation-officer/annual-innovation-strategy`

Chief Innovation Officer's annual strategy per Rumelt good-strategy kernel + McKinsey Three Horizons + Christensen + Govindarajan. Distinct from portfolio (which is the bet roster) — strategy is the moonshot vision + operating model + partnership thesis.

## When to invoke

- "Build the 2026 innovation strategy"
- "Annual innovation strategic refresh"
- "Moonshot review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-innovation-officer/annual-innovation-strategy \
  --input prior_strategy=./innovation/2025/strategy.md \
  --input portfolio=./innovation/2026/portfolio.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./innovation/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** new operating model adds 1-2 quarter to set up (CVC fund, etc.)

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Rumelt + McKinsey + Christensen.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt incomplete | Operator extends |

## Cross-references
- `../../../../modules/cuo/README.md` §5.7 — Chief Innovation Officer role profile
- `./annual-innovation-portfolio.md` — downstream consumer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
