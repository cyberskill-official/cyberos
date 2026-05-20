---
workflow_id: chief-procurement-officer/annual-procurement-strategy
workflow_version: 1.0.0
purpose: Author the annual procurement strategy — spend taxonomy, category playbooks, supplier-base strategy, savings targets.
persona: cuo/chief-procurement-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's procurement-strategy@1, format: procurement-strategy@1 }
  - { name: spend_cube,            source: AP + ERP spend analysis (CIPS spend-cube format), format: csv }
  - { name: budget_envelope,       source: cuo/cfo, format: budget@1 chapter }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: procurement_strategy,  format: procurement-strategy@1, recipient: cuo/cpo-procurement + cuo/cfo + cuo/ceo + Board (procurement chapter) }

skill_chain:
  - { step: 1, skill: procurement-strategy-author, inputs_from: { prior_strategy: prior_strategy, spend_cube: spend_cube, budget_envelope: budget_envelope, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: procurement-strategy-audit,  inputs_from: strategy_draft, outputs_to: procurement_strategy }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "savings target > 5% of opex OR strategy proposes single-supplier exposure" }

consults:
  - { persona: cuo/chief-legal-officer,      when: "contract-template / risk allocation" }
  - { persona: cuo/chief-sustainability-officer, when: "Scope 3 supplier emissions strategy" }

audit_hooks:
  - workflow_complete row on PASS with procurement_strategy hash + category count + savings target
  - HITL pause at step 2 on QA-KRALJIC-001 (categories not classified per Kraljic matrix)
---

# Annual procurement strategy — `chief-procurement-officer/annual-procurement-strategy`

CPO-Procurement's annual strategy per CIPS Global Standard + Kraljic strategic-sourcing matrix + ISM (Institute for Supply Management) + Hackett spend-analytics framework.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-financial-officer/workflows/annual-budget.md` — budget envelope upstream
- `../../../skill/procurement-strategy-{author,audit}/SKILL.md`
