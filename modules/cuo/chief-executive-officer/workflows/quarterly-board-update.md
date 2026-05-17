---
workflow_id: chief-executive-officer/quarterly-board-update
workflow_version: 1.0.0
purpose: Author the quarterly board deck — financial summary, strategy update, OKR progress, risks, asks.
persona: cuo/chief-executive-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_okrs,         source: cuo/chief-of-staff/quarterly-okr-cascade or last quarter's deck,  format: okr-set@1 }
  - { name: financials,         source: cuo/chief-financial-officer/quarterly-board-financials,                               format: monthly-close@1 + forecast@1 }
  - { name: ask_envelope,       source: workflow-caller,                                                  format: markdown brief (3-5 board asks) }

outputs:
  - { name: board_deck,         format: board-deck@1, recipient: cuo/ceo + Board of Directors }

skill_chain:
  - { step: 1, skill: board-deck-author, inputs_from: { prior_okrs: prior_okrs, financials: financials, ask_envelope: ask_envelope }, outputs_to: board_deck_draft }
  - { step: 2, skill: board-deck-audit,  inputs_from: board_deck_draft, outputs_to: board_deck }

escalates_to:
  - { persona: cuo/chief-financial-officer,             when: "financials inputs are missing the latest monthly-close" }
  - { persona: cuo/chief-of-staff,  when: "prior_okrs cannot be located — call OKR-cascade workflow first" }

consults:
  - { persona: cuo/chief-communications-officer, when: "board update has external-narrative implications (e.g. funding announcement, layoff)" }
  - { persona: cuo/chief-legal-officer,          when: "board update includes legal items (litigation, M&A, governance change)" }

audit_hooks:
  - each skill emits an artefact_write row to the BRAIN audit chain
  - workflow_complete row written on PASS with the board_deck artefact hash
  - HITL pause at step 1 PLAN (operator approves deck outline) and step 2 if QA-NUM-001 fires on unsourced numbers
---

# Quarterly board update — `chief-executive-officer/quarterly-board-update`

CEO's quarterly cadence for assembling the board deck. Combines OKR roll, financial summary, strategic update, and 3-5 explicit asks of the board.

## When to invoke

- "Build the Q<n> board deck"
- "Time for board prep"
- "Draft the next board update"

## How to invoke

```bash
cyberos-cuo run cuo/chief-executive-officer/quarterly-board-update \
  --input prior_okrs=./engagements/2026-Q1/okrs/final.md \
  --input financials=./engagements/2026-Q1/financials/monthly-close-mar.md \
  --input ask_envelope=./engagements/2026-Q1/board-asks.md \
  --output-dir ./engagements/2026-Q1/board/
```

## Expected duration

- **Happy path:** 20-40 min runtime + 1 business day operator round-trip
- **Worst case:** if board-deck-audit hits EXHAUSTED, escalate to CEO for manual revision

## Skill chain

- **Step 1 `board-deck-author`** — drafts the deck per Carta + a16z board-deck conventions. Pause for PLAN approval (outline).
- **Step 2 `board-deck-audit`** — validates against `board_deck_rubric@1.0` (FM + SEC: required board sections + QA-NUM-001 on figures + SAFE on speculative claims).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | Missing financials input — block until CFO provides |
| 1 | HITL | PLAN approval of outline by CEO |
| 2 | needs_human | QA-NUM-001 fires on unsourced number — operator supplies source |
| 2 | EXHAUSTED | Escalate to CEO for manual revision |

## Cross-references
- `../README.md` §5.3 — output type "board deck (quarterly)"
- `../../../docs/The C-Suite Reference.md` §5.1
- `../../../skill/board-deck-{author,audit}/SKILL.md`
