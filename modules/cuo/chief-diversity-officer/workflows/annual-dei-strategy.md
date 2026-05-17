---
workflow_id: chief-diversity-officer/annual-dei-strategy
workflow_version: 1.0.0
purpose: Author the annual DEI strategy — equity vision, intersectional priorities, manager-enablement, supplier diversity, external commitments.
persona: cuo/chief-diversity-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (DEI chapter), format: strategy-doc@1 }
  - { name: dei_program_state,     source: cuo/chief-diversity-officer/annual-dei-program, format: dei-program@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: dei_strategy,          format: strategy-doc@1, recipient: cuo/cdo-diversity + cuo/chro + cuo/ceo + Board (DEI strategic chapter) }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, dei_program_state: dei_program_state, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: dei_strategy }

audit_hooks:
  - workflow_complete row on PASS with dei_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual DEI strategy — `chief-diversity-officer/annual-dei-strategy`

CDO-Diversity's annual DEI strategy per Rumelt + Catalyst + McKinsey Diversity Wins + Project Include.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
