---
workflow_id: chief-administrative-officer/annual-ga-strategy
workflow_version: 1.0.0
purpose: Author the annual G&A (general & administrative) strategy — function priorities, automation roadmap, shared-services architecture, cost targets.
persona: cuo/chief-administrative-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (G&A chapter), format: strategy-document@1 }
  - { name: budget,                source: cuo/chief-financial-officer/annual-budget G&A line, format: budget@1 chapter }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: ga_strategy,           format: strategy-doc@1, recipient: cuo/cao-admin + cuo/cfo + cuo/ceo + Board (G&A chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, budget: budget, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: ga_strategy }

audit_hooks:
  - workflow_complete row on PASS with ga_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual G&A strategy — `chief-administrative-officer/annual-ga-strategy`

CAO-Admin's annual G&A strategy per Bain G&A-optimization + Hackett shared-services + KPMG G&A-benchmarking.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.1
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
