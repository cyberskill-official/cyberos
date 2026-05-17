---
workflow_id: chief-risk-officer/quarterly-board-risk-chapter
workflow_version: 1.0.0
purpose: Author the quarterly risk chapter of the board deck — ERM posture, KRI breaches, material incidents, top risks, mitigations.
persona: cuo/chief-risk-officer
cadence: quarterly
status: shipped

inputs:
  - { name: erm_framework,         source: cuo/chief-risk-officer/annual-erm-framework, format: enterprise-risk-framework@1 }
  - { name: kri_dashboard,         source: cuo/chief-risk-officer/quarterly-kri-dashboard, format: kri-dashboard@1 }
  - { name: incident_corpus,       source: quarter's risk-postmortems, format: postmortem@1 (multiple) }
  - { name: regulator_corpus,      source: cuo/chief-legal-officer/quarterly-regulatory-cycle filings, format: regulatory-filing@1 set }

outputs:
  - { name: board_risk_chapter,    format: board-deck@1 chapter (risk), recipient: cuo/ceo (for inclusion in quarterly-board-update) + Board }

skill_chain:
  - { step: 1, skill: board-deck-author, inputs_from: { erm_framework: erm_framework, kri_dashboard: kri_dashboard, incident_corpus: incident_corpus, regulator_corpus: regulator_corpus }, outputs_to: chapter_draft }
  - { step: 2, skill: board-deck-audit,  inputs_from: chapter_draft, outputs_to: board_risk_chapter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "chapter surfaces material risk needing 8-K disclosure" }
  - { persona: cuo/chief-legal-officer,      when: "chapter triggers regulatory-disclosure obligations" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "financial-risk classification needs CFO alignment" }
  - { persona: cuo/chief-information-security-officer,           when: "cyber-risk classification needs CISO alignment" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with board_risk_chapter hash + top-N risks + KRI-breach count
  - HITL pause at step 2 on QA-DISCLOSURE-001 (material risk without disclosure plan)
---

# Quarterly board risk chapter — `chief-risk-officer/quarterly-board-risk-chapter`

CRO-Risk's contribution to the quarterly board deck. Combines ERM framework + KRI dashboard + incident corpus + regulator corpus into the risk-chapter view for board consumption. Feeds `chief-executive-officer/quarterly-board-update`.

## When to invoke

- "Write the risk chapter for Q<n> board"
- "Board risk update"
- "CRO contribution to board deck"

## How to invoke

```bash
cyberos-cuo run cuo/chief-risk-officer/quarterly-board-risk-chapter \
  --input erm_framework=./risk/2026/erm/framework.md \
  --input kri_dashboard=./risk/2026-Q1/kri/dashboard.md \
  --input incident_corpus=./risk/2026-Q1/incidents/ \
  --input regulator_corpus=./regulatory/2026-Q1/ \
  --output-dir ./board/2026-Q1/risk-chapter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 business day operator review
- **Worst case:** material risk disclosure escalation adds 1-3 days

## Skill chain

- **Step 1 `board-deck-author`** — drafts risk-chapter view.
- **Step 2 `board-deck-audit`** — validates per `board_deck_rubric@1.0` chapter-mode rules.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-DISCLOSURE-001 | Material risk no disclosure plan | Escalate to CEO + CLO-Legal |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CRO-Risk role profile
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — board-deck consumer
- `../../../skill/board-deck-{author,audit}/SKILL.md`
