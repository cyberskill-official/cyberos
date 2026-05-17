---
workflow_id: chief-learning-officer/annual-learning-program
workflow_version: 1.0.0
purpose: Author the annual L&D program — capability framework, learning paths, manager training, leadership pipeline, budget.
persona: cuo/chief-learning-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,         source: last year's onboarding-pack@1 + learning materials, format: onboarding-pack@1 (extended) }
  - { name: workforce_plan,        source: cuo/chief-human-resources-officer/quarterly-workforce-plan, format: workforce-plan@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: learning_program,      format: onboarding-pack@1 (annual L&D extension), recipient: cuo/clo-learning + cuo/chro + cuo/ceo + Board (L&D chapter) }

skill_chain:
  - { step: 1, skill: onboarding-pack-author, inputs_from: { prior_program: prior_program, workforce_plan: workforce_plan, ceo_priorities: ceo_priorities }, outputs_to: program_draft }
  - { step: 2, skill: onboarding-pack-audit,  inputs_from: program_draft, outputs_to: learning_program }

audit_hooks:
  - workflow_complete row on PASS with learning_program hash
  - HITL pause at step 2 on QA-GOALS-001
---

# Annual learning program — `chief-learning-officer/annual-learning-program`

CLO-Learning's annual L&D program per ATD competency model + 70-20-10 learning framework + Kirkpatrick four-level evaluation.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5 — CLO-Learning role profile
- `../../chief-human-resources-officer/workflows/new-hire-onboarding.md` — per-hire peer
- `../../../skill/onboarding-pack-{author,audit}/SKILL.md`
