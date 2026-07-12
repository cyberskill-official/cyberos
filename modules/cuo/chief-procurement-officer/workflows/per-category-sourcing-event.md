---
workflow_id: chief-procurement-officer/per-category-sourcing-event
workflow_version: 1.0.0
purpose: Charter a major category sourcing event — RFx scope, evaluation criteria, supplier shortlist, timeline.
persona: cuo/chief-procurement-officer
cadence: per-event
status: shipped

inputs:
  - { name: category_brief,        source: category manager, format: markdown }
  - { name: procurement_strategy,  source: cuo/chief-procurement-officer/annual-procurement-strategy, format: procurement-strategy@1 }
  - { name: incumbent_data,        source: current supplier performance + spend, format: markdown }

outputs:
  - { name: sourcing_charter,      format: program-charter@1, recipient: cuo/cpo-procurement + category manager + cuo/clo-legal + cuo/cfo }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { category_brief: category_brief, procurement_strategy: procurement_strategy, incumbent_data: incumbent_data }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: sourcing_charter }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "category spend > $1M annually" }
  - { persona: cuo/chief-legal-officer,      when: "RFx requires new contract templates or risk-allocation framework" }

audit_hooks:
  - workflow_complete row on PASS with sourcing_charter hash + RFx scope
  - HITL pause at step 2 on QA-OWNER-001 or QA-EVAL-001 (evaluation criteria not weighted)
---

# Per category sourcing event — `chief-procurement-officer/per-category-sourcing-event`

CPO-Procurement's per-category sourcing event per CIPS + ISM strategic-sourcing methodology. Triggered per major category re-bid or strategic-supplier rotation.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `./annual-procurement-strategy.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
