---
workflow_id: chief-investment-officer/annual-investment-strategy
workflow_version: 1.0.0
purpose: Author the annual investment strategy — asset allocation, sector tilts, risk budget, manager selection (if FoF).
persona: cuo/chief-investment-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (investment chapter), format: strategy-document@1 }
  - { name: ips,                   source: Investment Policy Statement, format: markdown }
  - { name: macro_outlook,         source: macro research, format: markdown }

outputs:
  - { name: investment_strategy,   format: strategy-doc@1, recipient: cuo/cio-investment + investment committee + LP advisors + Board }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ips: ips, macro_outlook: macro_outlook }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: investment_strategy }

audit_hooks:
  - workflow_complete row on PASS with investment_strategy hash
  - HITL pause at step 2 on QA-IPS-001 (allocation outside IPS bounds)
---

# Annual investment strategy — `chief-investment-officer/annual-investment-strategy`

CIO-Investment's annual investment strategy per Yale Endowment Model + Norway sovereign-wealth model + Swensen Pioneering Portfolio Management.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
