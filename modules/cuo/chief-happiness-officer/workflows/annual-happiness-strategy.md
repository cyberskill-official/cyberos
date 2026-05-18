---
workflow_id: chief-happiness-officer/annual-happiness-strategy
workflow_version: 1.0.0
purpose: Author the annual happiness strategy — wellbeing pillars, programs portfolio, manager-enablement, measurement framework.
persona: cuo/chief-happiness-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (happiness chapter), format: strategy-document@1 }
  - { name: program_history,       source: 4 quarters of happiness-program@1, format: happiness-program@1 (4Q) }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: happiness_strategy,    format: strategy-doc@1, recipient: cuo/chief-happiness-officer + cuo/chro + cuo/ceo + Board (annual culture chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, program_history: program_history, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: happiness_strategy }

audit_hooks:
  - workflow_complete row on PASS with happiness_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual happiness strategy — `chief-happiness-officer/annual-happiness-strategy`

Chief Happiness Officer's annual strategy per Rumelt + positive-psychology research (Shawn Achor / Martin Seligman PERMA).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
