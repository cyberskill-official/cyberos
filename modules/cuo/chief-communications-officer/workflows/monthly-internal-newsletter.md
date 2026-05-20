---
workflow_id: chief-communications-officer/monthly-internal-newsletter
workflow_version: 1.0.0
purpose: Author the monthly all-hands internal newsletter — wins, decisions, OKR progress, people moves, asks.
persona: cuo/chief-communications-officer
cadence: monthly
status: shipped

inputs:
  - { name: prior_newsletter,      source: last month's internal-newsletter@1, format: internal-newsletter@1 }
  - { name: rob_summary,           source: cuo/chief-of-staff/weekly-rhythm-of-business (4 weeks), format: rhythm-of-business@1 (4 weeks) }
  - { name: people_moves,          source: cuo/chro (hires / promotions / departures), format: markdown }
  - { name: ceo_message,           source: cuo/ceo (CEO message section), format: markdown }

outputs:
  - { name: internal_newsletter,   format: internal-newsletter@1, recipient: all employees }

skill_chain:
  - { step: 1, skill: internal-newsletter-author, inputs_from: { prior_newsletter: prior_newsletter, rob_summary: rob_summary, people_moves: people_moves, ceo_message: ceo_message }, outputs_to: newsletter_draft }
  - { step: 2, skill: internal-newsletter-audit,  inputs_from: newsletter_draft, outputs_to: internal_newsletter }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "newsletter contains material business information not yet publicly disclosed (Reg FD concern)" }

consults:
  - { persona: cuo/chief-human-resources-officer,           when: "people-moves section needs sensitivity review" }
  - { persona: cuo/chief-executive-officer,            when: "CEO message needs voice/length refinement" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with internal_newsletter hash + section count + length
  - HITL pause at step 2 on QA-REGFD-001 (material info pre-public)
---

# Monthly internal newsletter — `chief-communications-officer/monthly-internal-newsletter`

CCO-Communications' monthly all-hands newsletter workflow. Per IABC + Edelman employee-comms best practices. Combines RoB summary + people moves + CEO message into structured monthly newsletter. Reg FD discipline critical for public companies (material info goes external before internal in pre-disclosure window).

## When to invoke

- "Write the May internal newsletter"
- "Monthly all-hands update"
- "Internal newsletter draft"

## How to invoke

```bash
cyberos-cuo run cuo/chief-communications-officer/monthly-internal-newsletter \
  --input prior_newsletter=./internal-comms/2026-04/newsletter.md \
  --input rob_summary=./rob/2026-W17-W20/ \
  --input people_moves=./hr/2026-05/people-moves.md \
  --input ceo_message=./chief-executive-officer/2026-05/message.md \
  --output-dir ./internal-comms/2026-05/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + same-day CEO + CHRO review
- **Worst case:** Reg FD flag may delay distribution until disclosure

## Skill chain

- **Step 1 `internal-newsletter-author`** — drafts per IABC + Edelman.
- **Step 2 `internal-newsletter-audit`** — validates per `internal_newsletter_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-REGFD-001 | Material info pre-public | Escalate to CLO-Legal |

## Cross-references
- `../../../../modules/cuo/README.md` §5.4 — CCO-Communications role profile
- `../../chief-of-staff/workflows/weekly-rhythm-of-business.md` — upstream feeder
- `../../../skill/internal-newsletter-{author,audit}/SKILL.md`
