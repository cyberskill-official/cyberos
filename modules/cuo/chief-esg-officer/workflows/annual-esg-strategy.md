---
workflow_id: chief-esg-officer/annual-esg-strategy
workflow_version: 1.0.0
purpose: Author the annual ESG strategy — materiality assessment, targets (net-zero / DEI / governance), investment plan, stakeholder engagement.
persona: cuo/chief-esg-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (ESG chapter), format: strategy-document@1 }
  - { name: materiality_assessment, source: ESG materiality study (GRI / SASB lens), format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: esg_strategy,          format: strategy-doc@1, recipient: cuo/chief-esg-officer + cuo/ceo + cuo/cso-sustainability + Board (annual ESG chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, materiality_assessment: materiality_assessment, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: esg_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes net-zero acceleration OR divestment of non-ESG-aligned BUs" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "ESG investment envelope" }
  - { persona: cuo/chief-diversity-officer,  when: "DEI targets" }

audit_hooks:
  - workflow_complete row on PASS with esg_strategy hash + materiality issues count + target count
  - HITL pause at step 2 on QA-MATERIALITY-001 or QA-TARGET-001 (target without baseline)
---

# Annual ESG strategy — `chief-esg-officer/annual-esg-strategy`

Chief ESG Officer's annual strategy per GRI Materiality + SASB + ISSB + SBTi targets framework + TCFD scenario analysis.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./annual-esg-report.md` — execution feeder
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
