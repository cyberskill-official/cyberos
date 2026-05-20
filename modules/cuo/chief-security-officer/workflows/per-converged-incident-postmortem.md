---
workflow_id: chief-security-officer/per-converged-incident-postmortem
workflow_version: 1.0.0
purpose: Author converged-security postmortem — physical + info-sec + supply-chain root-cause analysis, controls assessment.
persona: cuo/chief-security-officer
cadence: per-event
status: shipped

inputs:
  - { name: incident_brief,        source: security operations, format: markdown }
  - { name: ciso_pm,               source: if applicable, cuo/ciso info-sec postmortem, format: postmortem@1 }
  - { name: converged_strategy,    source: cuo/chief-security-officer/annual-converged-security-strategy, format: security-strategy@1 }

outputs:
  - { name: converged_postmortem,  format: postmortem@1, recipient: cuo/cso-security + cuo/ciso + cuo/clo-legal + cuo/cao-admin }

skill_chain:
  - { step: 1, skill: postmortem-author, inputs_from: { incident_brief: incident_brief, ciso_pm: ciso_pm, converged_strategy: converged_strategy }, outputs_to: pm_draft }
  - { step: 2, skill: postmortem-audit,  inputs_from: pm_draft, outputs_to: converged_postmortem }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "incident material (physical breach + data breach + brand impact)" }

audit_hooks:
  - workflow_complete row on PASS with converged_postmortem hash
  - HITL pause at step 2 on QA-ROOT-001 or QA-CONTROL-001
---

# Per converged incident postmortem — `chief-security-officer/per-converged-incident-postmortem`

CSO-Security's per-incident converged postmortem per ASIS ESRM incident-response + ICS (Incident Command System).

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-technology-officer/workflows/post-incident-review.md` — engineering peer
- `../../../skill/postmortem-{author,audit}/SKILL.md`
