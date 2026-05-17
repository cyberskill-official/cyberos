---
workflow_id: chief-innovation-officer/quarterly-portfolio-review
workflow_version: 1.0.0
purpose: Review innovation portfolio — stage-gate decisions, kill recommendations, graduation candidates, budget rebalancing.
persona: cuo/chief-innovation-officer
cadence: quarterly
status: shipped

inputs:
  - { name: portfolio,             source: cuo/chief-innovation-officer/annual-innovation-portfolio, format: innovation-portfolio@1 }
  - { name: per_bet_status,        source: bet sponsors (stage-gate progress + learnings), format: markdown briefs }
  - { name: market_signals,        source: cuo/cmo competitive-brief, format: markdown }

outputs:
  - { name: portfolio_review,      format: innovation-portfolio@1 (quarterly chapter), recipient: cuo/chief-innovation-officer + cuo/ceo + Board (innovation chapter) }

skill_chain:
  - { step: 1, skill: innovation-portfolio-author, inputs_from: { portfolio: portfolio, per_bet_status: per_bet_status, market_signals: market_signals }, outputs_to: review_draft }
  - { step: 2, skill: innovation-portfolio-audit,  inputs_from: review_draft, outputs_to: portfolio_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "portfolio review recommends kill of any Horizon-1 bet OR triggers >$250K reallocation" }

consults:
  - { persona: cuo/chief-product-officer,    when: "graduation candidates ready for Horizon-1 roadmap" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with portfolio_review hash + stage-gate decisions + kill count + graduation count
  - HITL pause at step 2 on QA-GATE-001 (stage-gate decision lacks rationale)
---

# Quarterly portfolio review — `chief-innovation-officer/quarterly-portfolio-review`

Chief Innovation Officer's quarterly stage-gate cycle. Per Cooper Stage-Gate model + Lean Startup pivot-or-persevere framework. Triggers kill / continue / graduate / pivot decisions per bet.

## When to invoke

- "Run the Q<n> innovation review"
- "Stage-gate cycle"
- "Innovation portfolio quarterly check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-innovation-officer/quarterly-portfolio-review \
  --input portfolio=./innovation/2026/portfolio.md \
  --input per_bet_status=./innovation/2026-Q1/bet-status/ \
  --input market_signals=./market/2026-Q1/intel.md \
  --output-dir ./innovation/2026-Q1/review/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for sponsor round-trip
- **Worst case:** Horizon-1 kill triggers customer-comms + product transition

## Skill chain

- **Step 1 `innovation-portfolio-author`** — drafts quarterly chapter.
- **Step 2 `innovation-portfolio-audit`** — validates per `innovation_portfolio_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-GATE-001 | Stage-gate decision no rationale | Operator drafts |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7 — Chief Innovation Officer role profile
- `./annual-innovation-portfolio.md` — upstream parent
- `../../../skill/innovation-portfolio-{author,audit}/SKILL.md`
