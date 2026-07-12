---
workflow_id: chief-executive-officer/monthly-investor-update
workflow_version: 1.0.0
purpose: Author the monthly investor update — financial snapshot, OKR progress, asks, recent wins/losses, lookback/lookforward.
persona: cuo/chief-executive-officer
cadence: monthly
status: shipped

inputs:
  - { name: financials_snapshot, source: cuo/chief-financial-officer/monthly-close,                         format: monthly-close@1 }
  - { name: okr_progress,        source: cuo/chief-of-staff/quarterly-okr-cascade or live OKR tool, format: markdown progress brief }
  - { name: investor_asks,       source: workflow-caller (CEO's 2-3 asks of this month's investors), format: markdown brief }

outputs:
  - { name: investor_update,     format: investor-update@1, recipient: investors (active + prospective) via CRM (Visible / Carta / Affinity) }

skill_chain:
  - { step: 1, skill: investor-update-author, inputs_from: { financials_snapshot: financials_snapshot, okr_progress: okr_progress, investor_asks: investor_asks }, outputs_to: update_draft }
  - { step: 2, skill: investor-update-audit,  inputs_from: update_draft, outputs_to: investor_update }

escalates_to:
  - { persona: cuo/chief-financial-officer,                 when: "financials snapshot is missing the latest monthly-close" }
  - { persona: cuo/chief-legal-officer,           when: "update contains forward-looking statements that need legal review (esp. post-IPO or pre-IPO governance)" }

consults:
  - { persona: cuo/chief-communications-officer,  when: "investor update mentions PR-sensitive items (layoffs, exec departures, customer losses)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with investor_update hash + recipient count
  - HITL pause at step 2 on QA-FORWARD-001 (forward-looking statement without disclaimer)
---

# Monthly investor update — `chief-executive-officer/monthly-investor-update`

CEO's monthly investor-update cadence. Combines CFO's monthly-close, OKR progress, and 2-3 explicit asks into the standard Visible.vc / NFX investor-update template. Audited for unsourced numbers and forward-looking-statement compliance.

## When to invoke

- "Write the May investor update"
- "Draft this month's investor letter"
- "Time for monthly investor email"

## How to invoke

```bash
cyberos-cuo run cuo/chief-executive-officer/monthly-investor-update \
  --input financials_snapshot=./engagements/2026-05/monthly-close.md \
  --input okr_progress=./engagements/2026-Q2/okrs/may-progress.md \
  --input investor_asks=./engagements/2026-05/investor-asks.md \
  --output-dir ./engagements/2026-05/investor-update/
```

## Expected duration

- **Happy path:** 15-30 min runtime + same-day operator review
- **Worst case:** legal-review escalation for forward-looking statements may add 1-2 days

## Skill chain

- **Step 1 `investor-update-author`** — drafts per Visible/NFX template: Highlights / Lowlights / Asks / KPIs / Financials.
- **Step 2 `investor-update-audit`** — validates per `investor_update_rubric@1.0` (FM + SEC + QA-NUM-001 on unsourced figures + QA-FORWARD-001 on FLS without disclaimer).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | Missing financials snapshot — block until CFO provides |
| 2 | QA-FORWARD-001 | Forward-looking statement without safe-harbor disclaimer | Escalate to clo-legal |
| 2 | QA-NUM-001 | Unsourced figure | Operator supplies source |

## Cross-references
- `../README.md` §5.3 — output type "investor updates (quarterly)"
- `../../../../modules/cuo/docs/module.md` §5.1
- `../../../skill/investor-update-{author,audit}/SKILL.md`
