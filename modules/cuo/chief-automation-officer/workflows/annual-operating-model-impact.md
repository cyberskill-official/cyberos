---
workflow_id: chief-automation-officer/annual-operating-model-impact
workflow_version: 1.0.0
purpose: Annual assessment of automation impact on operating model — role evolution, decision-rights changes, process redesign opportunities.
persona: cuo/chief-automation-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_assessment,      source: last year's operating-model@1 (automation lens), format: operating-model@1 }
  - { name: coo_operating_model,   source: cuo/chief-operating-officer/annual-operating-model, format: operating-model@1 }
  - { name: automation_inventory,  source: production bot + AI-augmentation inventory, format: csv }

outputs:
  - { name: operating_model_impact, format: operating-model@1 (automation chapter), recipient: cuo/chief-automation-officer + cuo/coo + cuo/chro + cuo/ceo }

skill_chain:
  - { step: 1, skill: operating-model-author, inputs_from: { prior_assessment: prior_assessment, coo_operating_model: coo_operating_model, automation_inventory: automation_inventory }, outputs_to: assessment_draft }
  - { step: 2, skill: operating-model-audit,  inputs_from: assessment_draft, outputs_to: operating_model_impact }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "impact suggests > 10% workforce restructure" }

audit_hooks:
  - workflow_complete row on PASS with operating_model_impact hash
  - HITL pause at step 2 on QA-RACI-001
---

# Annual operating model impact — `chief-automation-officer/annual-operating-model-impact`

Chief Automation Officer's annual operating-model-impact assessment per McKinsey 7S applied to automated workforce + Gartner future-of-work.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../chief-operating-officer/workflows/annual-operating-model.md` — upstream peer
- `../../../skill/operating-model-{author,audit}/SKILL.md`
