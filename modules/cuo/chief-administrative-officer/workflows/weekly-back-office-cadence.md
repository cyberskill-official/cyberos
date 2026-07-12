---
workflow_id: chief-administrative-officer/weekly-back-office-cadence
workflow_version: 1.0.0
purpose: Maintain back-office operating rhythm — function-head sync, ticket queues, escalation triage, vendor coordination.
persona: cuo/chief-administrative-officer
cadence: weekly
status: shipped

inputs:
  - { name: prior_rob,             source: last week's rhythm-of-business@1, format: rhythm-of-business@1 }
  - { name: ticket_queues,         source: IT / HR / Finance / Legal ticket data, format: csv }
  - { name: function_inputs,       source: back-office function heads, format: markdown briefs }

outputs:
  - { name: back_office_rob,       format: rhythm-of-business@1, recipient: cuo/cao-admin + back-office function heads + cuo/coo }

skill_chain:
  - { step: 1, skill: rhythm-of-business-author, inputs_from: { prior_rob: prior_rob, ticket_queues: ticket_queues, function_inputs: function_inputs }, outputs_to: rob_draft }
  - { step: 2, skill: rhythm-of-business-audit,  inputs_from: rob_draft, outputs_to: back_office_rob }

audit_hooks:
  - workflow_complete row on PASS with back_office_rob hash
  - HITL pause at step 2 on QA-DECISION-AGE-001
---

# Weekly back-office cadence — `chief-administrative-officer/weekly-back-office-cadence`

CAO-Admin's weekly back-office operating cadence per First Round + Y Combinator ops-leader playbook.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.1
- `../../chief-of-staff/workflows/weekly-rhythm-of-business.md` — broader peer
- `../../../skill/rhythm-of-business-{author,audit}/SKILL.md`
