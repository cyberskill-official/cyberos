---
workflow_id: chief-experience-officer/per-journey-charter
workflow_version: 1.0.0
purpose: Charter a journey-redesign program — current-state mapping, friction analysis, future-state vision, ownership across functions.
persona: cuo/chief-experience-officer
cadence: per-event
status: shipped

inputs:
  - { name: journey_brief,         source: requestor, format: markdown }
  - { name: customer_research,     source: journey research (interviews + telemetry), format: markdown }
  - { name: cx_strategy,           source: cuo/chief-experience-officer/annual-cx-strategy, format: strategy-document@1 }

outputs:
  - { name: journey_charter,       format: program-charter@1, recipient: cuo/cxo + cross-functional journey owners + cuo/cpo-product + cuo/cco-customer }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { journey_brief: journey_brief, customer_research: customer_research, cx_strategy: cx_strategy }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: journey_charter }

escalates_to:
  - { persona: cuo/chief-product-officer,    when: "journey redesign requires product-roadmap commitment" }

audit_hooks:
  - workflow_complete row on PASS with journey_charter hash
  - HITL pause at step 2 on QA-OWNER-001 (cross-function ownership ambiguous)
---

# Per journey charter — `chief-experience-officer/per-journey-charter`

CXO's per-journey-redesign charter per Forrester customer-journey-mapping + IDEO design-thinking + Nielsen Norman Group UX research.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4
- `../../../skill/program-charter-{author,audit}/SKILL.md`
