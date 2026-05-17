---
workflow_id: chief-financial-officer/quarterly-board-financials
workflow_version: 1.0.0
purpose: Author the financial chapter of the quarterly board deck — close summary + forecast vs actuals + cash position + KPIs.
persona: cuo/chief-financial-officer
cadence: quarterly
status: shipped

inputs:
  - { name: monthly_closes,     source: cuo/chief-financial-officer/monthly-close (last 3 months), format: monthly-close@1 x3 }
  - { name: prior_forecast,     source: cuo/chief-financial-officer/quarterly-forecast,            format: forecast@1 }
  - { name: budget,             source: cuo/chief-financial-officer/annual-budget,                 format: budget@1 }

outputs:
  - { name: board_financials,   format: board-deck@1 financial chapter, recipient: cuo/ceo (for inclusion in quarterly-board-update) + Board }

skill_chain:
  - { step: 1, skill: board-deck-author, inputs_from: { monthly_closes: monthly_closes, prior_forecast: prior_forecast, budget: budget }, outputs_to: financials_chapter_draft }
  - { step: 2, skill: board-deck-audit,  inputs_from: financials_chapter_draft, outputs_to: board_financials }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "actuals miss forecast by >10% without narrative; CEO owns the board-side story" }

consults:
  - { persona: cuo/chief-communications-officer, when: "miss requires PR-positioning prep" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with board_financials hash
  - HITL pause at step 2 on QA-VAR-001 (forecast miss without narrative) or QA-NUM-001 (unsourced figure)
---

# Quarterly board financials — `chief-financial-officer/quarterly-board-financials`

CFO's contribution to the quarterly board deck. Authors the financial chapter (closes summary + forecast vs actuals + cash position + KPIs) using the board-deck skill targeted at the financial section only. Handed off to CEO for inclusion in the full board deck via `chief-executive-officer/quarterly-board-update`.

## When to invoke

- "Write the board financial chapter for Q<n>"
- "Prep the financial section for next board meeting"
- "CFO's slides for Q<n> board"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/quarterly-board-financials \
  --input monthly_closes=./close/2026-Q1/ \
  --input prior_forecast=./forecast/2026-Q1/final.md \
  --input budget=./budget/2026/final/budget.md \
  --output-dir ./board/2026-Q1/financials/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + same-day operator review
- **Worst case:** material miss requires escalation; 1-3 day round-trip with CEO

## Skill chain

- **Step 1 `board-deck-author`** — drafts financial chapter only (board-deck skill supports chapter-mode); sections: close summary, forecast vs actuals, cash position, KPIs, asks.
- **Step 2 `board-deck-audit`** — validates per `board_deck_rubric@1.0` financial subsection rules.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-VAR-001 | Forecast miss > 10% with no narrative | Escalate to CEO |
| 2 | QA-NUM-001 | Unsourced figure | Operator supplies source |
| 2 | QA-PRIOR-001 | No comparison to prior quarter or prior year | Operator adds comparison |

## Cross-references
- `../README.md` §5 (Communication) — output type "board financials"
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — peer workflow that consumes this output
- `../../../skill/board-deck-{author,audit}/SKILL.md`
