---
workflow_id: chief-accounting-officer/monthly-close-execution
workflow_version: 1.0.0
purpose: Execute the monthly close — accruals, reconciliations, technical-accounting reviews, controller sign-offs.
persona: cuo/chief-accounting-officer
cadence: monthly
status: shipped

inputs:
  - { name: cfo_close_artefact,    source: cuo/chief-financial-officer/monthly-close, format: monthly-close@1 }
  - { name: prior_close,           source: last month's close execution, format: monthly-close@1 }
  - { name: technical_issues,      source: technical-accounting team (revenue rec, leases, equity, M&A), format: markdown }

outputs:
  - { name: close_execution,       format: monthly-close@1 (controller execution log), recipient: cuo/cao-accounting + cuo/cfo + external auditors }

skill_chain:
  - { step: 1, skill: monthly-close-author, inputs_from: { cfo_close_artefact: cfo_close_artefact, prior_close: prior_close, technical_issues: technical_issues }, outputs_to: execution_draft }
  - { step: 2, skill: monthly-close-audit,  inputs_from: execution_draft, outputs_to: close_execution }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "technical-accounting decision requires CFO approval (rev rec policy / lease modification / etc.)" }

audit_hooks:
  - workflow_complete row on PASS with close_execution hash + close-day count
  - HITL pause at step 2 on QA-RECON-001 or QA-TECHNICAL-001
---

# Monthly close execution — `chief-accounting-officer/monthly-close-execution`

CAO-Accounting's monthly close execution per US GAAP / IFRS technical-accounting requirements + SOC 1 controls. Sister workflow to CFO's `monthly-close` — CFO orchestrates; CAO-Accounting executes technical-accounting decisions.

## Cross-references
- `../../../../modules/cuo/README.md` §5.2 — CAO-Accounting role profile
- `../../chief-financial-officer/workflows/monthly-close.md` — upstream orchestrator
- `../../../skill/monthly-close-{author,audit}/SKILL.md`
