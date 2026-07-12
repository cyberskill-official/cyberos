---
workflow_id: chief-information-officer/quarterly-it-operating-review
workflow_version: 1.0.0
purpose: Review IT operations — uptime / SLA / incident MTTR / change-success / spend trends.
persona: cuo/chief-information-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_review,          source: last quarter's review, format: operating-model@1 (IT chapter) }
  - { name: itsm_metrics,          source: ServiceNow / Jira Service Mgmt / Zendesk, format: csv }
  - { name: spend_actuals,         source: AP IT-line, format: csv }

outputs:
  - { name: it_operating_review,   format: operating-model@1 (IT quarterly chapter), recipient: cuo/cio-information + cuo/cao-admin + cuo/cfo }

skill_chain:
  - { step: 1, skill: operating-model-author, inputs_from: { prior_review: prior_review, itsm_metrics: itsm_metrics, spend_actuals: spend_actuals }, outputs_to: review_draft }
  - { step: 2, skill: operating-model-audit,  inputs_from: review_draft, outputs_to: it_operating_review }

audit_hooks:
  - workflow_complete row on PASS with it_operating_review hash
  - HITL pause at step 2 on QA-SLA-001
---

# Quarterly IT operating review — `chief-information-officer/quarterly-it-operating-review`

CIO-Information's quarterly IT operating review per ITIL 4 + COBIT 2019 service-management metrics.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3
- `../../../skill/operating-model-{author,audit}/SKILL.md`
