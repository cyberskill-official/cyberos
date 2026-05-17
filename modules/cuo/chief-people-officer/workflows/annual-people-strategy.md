---
workflow_id: chief-people-officer/annual-people-strategy
workflow_version: 1.0.0
purpose: Author the annual people strategy (synonym of CHRO strategic refresh) — capability + comp + DEI + wellbeing + workforce vision.
persona: cuo/chief-people-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (people chapter), format: strategy-doc@1 }
  - { name: workforce_plan,        source: cuo/chief-human-resources-officer/quarterly-workforce-plan, format: workforce-plan@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: people_strategy,       format: strategy-doc@1, recipient: cuo/cpo-people + cuo/chro + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, workforce_plan: workforce_plan, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: people_strategy }

audit_hooks:
  - workflow_complete row on PASS with people_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual people strategy — `chief-people-officer/annual-people-strategy`

CPO-People's annual people strategy — synonym variant of CHRO strategic-refresh. Use when firm's nomenclature is "Chief People Officer".

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../chro/` — canonical implementation
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
