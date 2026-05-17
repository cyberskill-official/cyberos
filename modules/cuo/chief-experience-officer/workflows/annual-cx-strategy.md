---
workflow_id: chief-experience-officer/annual-cx-strategy
workflow_version: 1.0.0
purpose: Author the annual CX strategy — experience vision, journey priorities, experience-debt roadmap, measurement framework.
persona: cuo/chief-experience-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (CX chapter), format: strategy-doc@1 }
  - { name: customer_360,          source: cuo/chief-data-officer/annual-customer-360-architecture, format: customer-360@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: cx_strategy,           format: strategy-doc@1, recipient: cuo/cxo + cuo/cpo-product + cuo/cco-customer + cuo/cmo + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, customer_360: customer_360, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: cx_strategy }

audit_hooks:
  - workflow_complete row on PASS with cx_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual CX strategy — `chief-experience-officer/annual-cx-strategy`

CXO's annual experience strategy per Forrester CX framework + Bain Net Promoter System + Don Norman User-Experience principles.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
