---
workflow_id: chief-sustainability-officer/annual-sustainability-strategy
workflow_version: 1.0.0
purpose: Author the annual sustainability strategy — climate ambition, supply-chain transformation, product-circularity, regenerative practices.
persona: cuo/chief-sustainability-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (sustainability chapter), format: strategy-doc@1 }
  - { name: esg_strategy,          source: cuo/chief-esg-officer/annual-esg-strategy, format: strategy-doc@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: sustainability_strategy, format: strategy-doc@1, recipient: cuo/cso-sustainability + cuo/chief-esg-officer + cuo/ceo + Board (annual chapter) }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, esg_strategy: esg_strategy, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: sustainability_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes net-zero acceleration OR major capex commitment" }

audit_hooks:
  - workflow_complete row on PASS with sustainability_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual sustainability strategy — `chief-sustainability-officer/annual-sustainability-strategy`

CSO-Sustainability's annual strategy per SBTi + Project Drawdown + Ellen MacArthur Foundation circular-economy framework.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../chief-esg-officer/workflows/annual-esg-strategy.md` — peer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
