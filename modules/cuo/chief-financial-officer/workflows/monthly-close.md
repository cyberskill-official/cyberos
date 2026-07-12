---
workflow_id: chief-financial-officer/monthly-close
workflow_version: 1.0.0
purpose: Close the books for the month — reconcile, post journal entries, generate P&L + BS + CF, package the close report.
persona: cuo/chief-financial-officer
cadence: monthly
status: shipped

inputs:
  - { name: prior_close,        source: last month's monthly-close@1, format: monthly-close@1 }
  - { name: subledger_extract,  source: NetSuite / SAP / Oracle ERP,  format: csv extracts }
  - { name: bank_statements,    source: treasury,                     format: csv / pdf }
  - { name: payroll_register,   source: Gusto / ADP / Rippling,       format: csv extract }

outputs:
  - { name: monthly_close,      format: monthly-close@1, recipient: cuo/cfo + cuo/ceo + Board (quarterly roll-up) }

skill_chain:
  - { step: 1, skill: monthly-close-author, inputs_from: { prior_close: prior_close, subledger_extract: subledger_extract, bank_statements: bank_statements, payroll_register: payroll_register }, outputs_to: close_draft }
  - { step: 2, skill: monthly-close-audit,  inputs_from: close_draft, outputs_to: monthly_close }

escalates_to:
  - { persona: cuo/chief-legal-officer,   when: "monthly-close-audit fires QA-VAR-001 — material variance triggers SOX/SOC-2 reportable event" }

consults:
  - { persona: cuo/chief-accounting-officer, when: "GL recon delta > materiality threshold — Controller validates classification" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with monthly_close hash + close-day count
  - HITL pause at step 2 on QA-RECON-001 (recon unresolved beyond materiality)
---

# Monthly close — `chief-financial-officer/monthly-close`

CFO's monthly book-close workflow. Reconciles subledgers, posts adjusting JEs, generates P&L + BS + CF, packages the close-day report per GAAP/IFRS (whichever the company reports under). Targets ≤7 business-day close for scale-ups, ≤5 for mature finance orgs.

## When to invoke

- "Close the books for [month]"
- "Run the monthly close"
- "Time to post May close"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/monthly-close \
  --input prior_close=./close/2026-04/monthly-close.md \
  --input subledger_extract=./close/2026-05/extracts/ \
  --input bank_statements=./close/2026-05/bank/ \
  --input payroll_register=./close/2026-05/payroll.csv \
  --output-dir ./close/2026-05/
```

## Expected duration

- **Happy path:** 1-3 hours runtime + 3-5 business days for human recon
- **Worst case:** material variance or unresolved recon escalation may push close to 10+ days

## Skill chain

- **Step 1 `monthly-close-author`** — drafts close package per GAAP/IFRS structure (P&L → BS → CF → MD&A); pause for PLAN approval on materiality threshold.
- **Step 2 `monthly-close-audit`** — validates per `monthly_close_rubric@1.0` (FM + SEC + QA-RECON-001 (recon to subledger + bank) + QA-VAR-001 (variance vs forecast + prior period)).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | Missing subledger extract | Operator pulls from ERP; resume |
| 2 | QA-RECON-001 | Recon delta > materiality | Consult Controller (cao-accounting) |
| 2 | QA-VAR-001 | Material variance vs forecast | Escalate to CFO for variance commentary |

## Cross-references
- `../README.md` §5 (Operational) — output type "monthly close"
- `../../../../modules/cuo/docs/module.md` §5.2
- `../../../skill/monthly-close-{author,audit}/SKILL.md`
