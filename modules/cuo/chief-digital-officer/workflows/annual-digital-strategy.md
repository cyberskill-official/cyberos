---
workflow_id: chief-digital-officer/annual-digital-strategy
workflow_version: 1.0.0
purpose: Author the annual digital strategy — digital ambition, business-model implications, technology bets, organizational capabilities.
persona: cuo/chief-digital-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (digital chapter), format: strategy-document@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: digital_quotient,      source: McKinsey DQ benchmark or internal, format: markdown }

outputs:
  - { name: digital_strategy,      format: strategy-doc@1, recipient: cuo/chief-digital-officer + cuo/ceo + Board (annual digital chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, digital_quotient: digital_quotient }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: digital_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes business-model change" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "platform implications" }
  - { persona: cuo/chief-data-officer,       when: "data foundations" }
  - { persona: cuo/chief-ai-officer,           when: "AI bets" }

audit_hooks:
  - workflow_complete row on PASS with digital_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual digital strategy — `chief-digital-officer/annual-digital-strategy`

CDO-Digital's annual digital strategy per Rumelt + McKinsey DQ + MIT CISR digital-mastery + Gartner DBT.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
