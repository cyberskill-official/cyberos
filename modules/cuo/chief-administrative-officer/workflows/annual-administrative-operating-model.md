---
workflow_id: chief-administrative-officer/annual-administrative-operating-model
workflow_version: 1.0.0
purpose: Author the annual administrative operating model — back-office org, processes, governance, shared-services architecture.
persona: cuo/chief-administrative-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_model,           source: last year's operating-model@1 (admin chapter), format: operating-model@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: cost_baseline,         source: G&A spend analysis, format: csv }

outputs:
  - { name: admin_operating_model, format: operating-model@1, recipient: cuo/cao-admin + cuo/coo + cuo/cfo + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: operating-model-author, inputs_from: { prior_model: prior_model, ceo_priorities: ceo_priorities, cost_baseline: cost_baseline }, outputs_to: model_draft }
  - { step: 2, skill: operating-model-audit,  inputs_from: model_draft, outputs_to: admin_operating_model }

audit_hooks:
  - workflow_complete row on PASS with admin_operating_model hash
  - HITL pause at step 2 on QA-RACI-001
---

# Annual administrative operating model — `chief-administrative-officer/annual-administrative-operating-model`

CAO-Admin's annual operating model for back-office functions per McKinsey 7S + Bain back-office-excellence framework.

## Cross-references
- `../../../../modules/cuo/README.md` §5.1 — CAO-Admin role profile
- `../../chief-operating-officer/workflows/annual-operating-model.md` — broader peer
- `../../../skill/operating-model-{author,audit}/SKILL.md`
