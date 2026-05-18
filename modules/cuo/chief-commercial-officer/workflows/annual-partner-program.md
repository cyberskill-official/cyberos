---
workflow_id: chief-commercial-officer/annual-partner-program
workflow_version: 1.0.0
purpose: Author the annual partner program — partner taxonomy, tiering, enablement, economics, joint go-to-market.
persona: cuo/chief-commercial-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,         source: last year's partner-program@1, format: partner-program@1 }
  - { name: gtm_plan,              source: cuo/chief-sales-officer/annual-gtm-plan, format: go-to-market-plan@1 }
  - { name: partner_performance,   source: per-partner pipeline + revenue contribution, format: csv }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: partner_program,       format: partner-program@1, recipient: cuo/cco-commercial + cuo/cso-sales + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: partner-program-author, inputs_from: { prior_program: prior_program, gtm_plan: gtm_plan, partner_performance: partner_performance, ceo_priorities: ceo_priorities }, outputs_to: program_draft }
  - { step: 2, skill: partner-program-audit,  inputs_from: program_draft, outputs_to: partner_program }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "program proposes channel-mix shift > 20% YoY" }

consults:
  - { persona: cuo/chief-sales-officer,      when: "channel-conflict policy" }
  - { persona: cuo/chief-marketing-officer,            when: "co-marketing programs" }

audit_hooks:
  - workflow_complete row on PASS with partner_program hash + tier count + partner count
  - HITL pause at step 2 on QA-TIER-001 or QA-ECON-001
---

# Annual partner program — `chief-commercial-officer/annual-partner-program`

CCO-Commercial's annual partner program per Crossbeam + Impartner partner-program standards + TSIA partner-economics framework.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — CCO-Commercial role profile
- `../../chief-sales-officer/workflows/annual-gtm-plan.md` — upstream peer
- `../../../skill/partner-program-{author,audit}/SKILL.md`
