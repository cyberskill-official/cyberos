---
workflow_id: chief-medical-officer/annual-medical-strategy
workflow_version: 1.0.0
purpose: Author the annual medical strategy — therapeutic-area focus, evidence-generation plan, real-world evidence, KOL engagement.
persona: cuo/chief-medical-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (medical chapter), format: strategy-doc@1 }
  - { name: pipeline,              source: clinical pipeline status, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: medical_strategy,      format: strategy-doc@1, recipient: cuo/chief-medical-officer + cuo/ceo + Board (annual medical chapter) }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, pipeline: pipeline, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: medical_strategy }

audit_hooks:
  - workflow_complete row on PASS with medical_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual medical strategy — `chief-medical-officer/annual-medical-strategy`

Chief Medical Officer's annual strategy per Rumelt + pharma medical-affairs framework (Cello Health, ISMPP, MAPS guidance).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
