---
workflow_id: chief-legal-officer/quarterly-board-legal-chapter
workflow_version: 1.0.0
purpose: Author the legal chapter of the quarterly board deck — litigation status, regulatory matters, M&A legal, governance updates.
persona: cuo/chief-legal-officer
cadence: quarterly
status: shipped

inputs:
  - { name: matter_list,          source: cuo/clo-legal's matter-management tool (Litify / iManage),  format: csv export }
  - { name: counsel_bills,        source: AP system (filtered for outside-counsel),                   format: csv extract }
  - { name: regulatory_summary,   source: cuo/chief-legal-officer/quarterly-regulatory-cycle,                   format: regulatory-filing@1 set summary }
  - { name: ma_status,            source: cuo/cso-strategy + cuo/clo-legal (active M&A pipeline),    format: markdown brief }

outputs:
  - { name: legal_chapter,        format: litigation-management-update@1 + board-deck@1 chapter, recipient: cuo/ceo (for inclusion in quarterly-board-update) + Board }

skill_chain:
  - { step: 1, skill: litigation-management-update-author, inputs_from: { matter_list: matter_list, counsel_bills: counsel_bills, regulatory_summary: regulatory_summary, ma_status: ma_status }, outputs_to: litigation_update_draft }
  - { step: 2, skill: litigation-management-update-audit,  inputs_from: litigation_update_draft, outputs_to: litigation_update }
  - { step: 3, skill: board-deck-author,             inputs_from: { litigation_update: litigation_update, chapter_mode: "legal" }, outputs_to: legal_chapter_draft }
  - { step: 4, skill: board-deck-audit,              inputs_from: legal_chapter_draft, outputs_to: legal_chapter }

escalates_to:
  - { persona: cuo/chief-executive-officer,           when: "exposure quantification triggers material-event disclosure under 8-K Item 8.01" }
  - { persona: cuo/chief-financial-officer,           when: "reserve recommendation diverges from accounting reserve by > materiality threshold" }

consults:
  - { persona: cuo/chief-communications-officer, when: "litigation matters have PR-positioning implications" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with legal_chapter hash + total-exposure + counsel-cost-vs-budget
  - HITL pause at step 2 on QA-EXPOSURE-001 (matter lacks quantified exposure) or QA-COSTBENCHMARK-001 (counsel cost > ACC Value-Challenge p90)
---

# Quarterly board legal chapter — `chief-legal-officer/quarterly-board-legal-chapter`

CLO-Legal's contribution to the quarterly board deck. Two-stage chain: first author the underlying litigation-management update with exposure quantification + reserve recommendation; then author the board-chapter rendering of it for inclusion in `chief-executive-officer/quarterly-board-update`.

## When to invoke

- "Write the legal chapter for Q<n> board"
- "CLO contribution to the board deck"
- "Prep the legal section for next board meeting"

## How to invoke

```bash
cyberos-cuo run cuo/chief-legal-officer/quarterly-board-legal-chapter \
  --input matter_list=./legal/matters/2026-Q1-export.csv \
  --input counsel_bills=./ap/2026-Q1/counsel.csv \
  --input regulatory_summary=./regulatory/2026-Q1/summary.md \
  --input ma_status=./ma/2026-Q1/active-deals.md \
  --output-dir ./board/2026-Q1/legal-chapter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 business day operator review
- **Worst case:** material-event disclosure triggers same-day CEO+CFO+CCO-Communications coordination

## Skill chain

- **Step 1 `litigation-management-update-author`** — drafts per Litify model: active matters / exposure / settlement posture / counsel costs / reserve.
- **Step 2 `litigation-management-update-audit`** — validates per `litigation_mgmt_update_rubric@1.0`.
- **Step 3 `board-deck-author`** — renders legal-chapter view of the litigation update for board consumption.
- **Step 4 `board-deck-audit`** — validates per `board_deck_rubric@1.0` chapter-mode rules.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-EXPOSURE-001 | Matter lacks quantified exposure | Operator commissions outside-counsel estimate |
| 2 | QA-COSTBENCHMARK-001 | Counsel cost > ACC p90 | Operator documents rationale or renegotiates |
| 4 | QA-DISCLOSURE-001 | Material event implied by exposure but not flagged | Escalate to CEO/CFO for 8-K decision |

## Cross-references
- `../README.md` §5 (Communication) — "board legal chapter"
- `../../../../modules/cuo/docs/module.md` §5.2
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — peer workflow that consumes this output
- `../../../skill/{litigation-mgmt-update,board-deck}-{author,audit}/SKILL.md`
