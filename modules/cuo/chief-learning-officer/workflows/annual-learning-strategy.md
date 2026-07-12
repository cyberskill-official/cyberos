---
workflow_id: chief-learning-officer/annual-learning-strategy
workflow_version: 1.0.0
purpose: Author the annual L&D strategy — capability vision, skills-of-the-future investment, leadership pipeline, learning-tech stack.
persona: cuo/chief-learning-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (L&D chapter), format: strategy-document@1 }
  - { name: workforce_plan,        source: cuo/chief-human-resources-officer/quarterly-workforce-plan, format: workforce-plan@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: learning_strategy,     format: strategy-doc@1, recipient: cuo/clo-learning + cuo/chro + cuo/ceo + Board (annual L&D chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, workforce_plan: workforce_plan, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: learning_strategy }

audit_hooks:
  - workflow_complete row on PASS with learning_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual learning strategy — `chief-learning-officer/annual-learning-strategy`

CLO-Learning's annual L&D strategy per ATD State-of-the-Industry + LinkedIn Workplace Learning Report + Brandon Hall research.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.5
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
