---
workflow_id: chief-commercial-officer/annual-channel-strategy
workflow_version: 1.0.0
purpose: Author the annual channel strategy — direct vs partner vs marketplace mix, channel economics, conflict resolution policy.
persona: cuo/chief-commercial-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-doc@1 (channel chapter), format: strategy-doc@1 }
  - { name: gtm_plan,              source: cuo/chief-sales-officer/annual-gtm-plan, format: gtm-plan@1 }
  - { name: partner_program,       source: cuo/chief-commercial-officer/annual-partner-program, format: partner-program@1 }

outputs:
  - { name: channel_strategy,      format: strategy-doc@1, recipient: cuo/cco-commercial + cuo/cso-sales + cuo/ceo + Board (commercial chapter) }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_strategy: prior_strategy, gtm_plan: gtm_plan, partner_program: partner_program }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: strategy_draft, outputs_to: channel_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes channel-mix shift > 30%" }

audit_hooks:
  - workflow_complete row on PASS with channel_strategy hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual channel strategy — `chief-commercial-officer/annual-channel-strategy`

CCO-Commercial's annual channel strategy per Rumelt + TSIA channel-economics + Crossbeam ecosystem-led growth framework.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4
- `../../chief-sales-officer/workflows/annual-gtm-plan.md` — peer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
