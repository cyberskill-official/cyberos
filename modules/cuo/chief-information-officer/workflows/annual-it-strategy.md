---
workflow_id: chief-information-officer/annual-it-strategy
workflow_version: 1.0.0
purpose: Author the annual IT strategy — service catalog, infrastructure roadmap, vendor stack, security posture, cost optimization.
persona: cuo/chief-information-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (IT chapter), format: strategy-document@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: business_needs,        source: function-head IT needs, format: markdown }

outputs:
  - { name: it_strategy,           format: strategy-doc@1, recipient: cuo/cio-information + cuo/cto + cuo/cao-admin + cuo/ceo + Board (IT chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, business_needs: business_needs }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: it_strategy }

audit_hooks:
  - workflow_complete row on PASS with it_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual IT strategy — `chief-information-officer/annual-it-strategy`

CIO-Information's annual IT strategy per ITIL 4 + COBIT 2019 + Gartner Bimodal IT framework.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CIO-Information role profile
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
