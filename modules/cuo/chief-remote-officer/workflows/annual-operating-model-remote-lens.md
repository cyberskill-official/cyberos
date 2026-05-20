---
workflow_id: chief-remote-officer/annual-operating-model-remote-lens
workflow_version: 1.0.0
purpose: Annual remote-lens review of operating model — async/sync decision rights, default-to-doc processes, hub-and-spoke geography strategy.
persona: cuo/chief-remote-officer
cadence: annual
status: shipped

inputs:
  - { name: coo_operating_model,   source: cuo/chief-operating-officer/annual-operating-model, format: operating-model@1 }
  - { name: remote_policy,         source: cuo/chief-remote-officer/annual-remote-policy, format: remote-policy@1 }

outputs:
  - { name: operating_model_remote, format: operating-model@1 (remote chapter), recipient: cuo/chief-remote-officer + cuo/coo + cuo/chro + cuo/ceo }

skill_chain:
  - { step: 1, skill: operating-model-author, inputs_from: { coo_operating_model: coo_operating_model, remote_policy: remote_policy }, outputs_to: assessment_draft }
  - { step: 2, skill: operating-model-audit,  inputs_from: assessment_draft, outputs_to: operating_model_remote }

audit_hooks:
  - workflow_complete row on PASS with operating_model_remote hash
  - HITL pause at step 2 on QA-RACI-001
---

# Annual operating-model remote lens — `chief-remote-officer/annual-operating-model-remote-lens`

Chief Remote Officer's annual remote-lens review of the operating model per GitLab handbook + Atlassian Distributed-Work Playbook.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-operating-officer/workflows/annual-operating-model.md` — upstream peer
- `../../../skill/operating-model-{author,audit}/SKILL.md`
