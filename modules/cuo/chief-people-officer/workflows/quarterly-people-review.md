---
workflow_id: chief-people-officer/quarterly-people-review
workflow_version: 1.0.0
purpose: Quarterly people review (synonym variant) — workforce + engagement + DEI + comp-equity in one consolidated view.
persona: cuo/chief-people-officer
cadence: quarterly
status: shipped

inputs:
  - { name: workforce_plan,        source: cuo/chief-human-resources-officer/quarterly-workforce-plan, format: workforce-plan@1 }
  - { name: enps_data,             source: cuo/chief-human-resources-officer/quarterly-enps-pulse, format: employee-net-promoter-score-program@1 }
  - { name: dei_progress,          source: cuo/chief-diversity-officer/quarterly-dei-progress-review, format: diversity-equity-inclusion-program@1 (quarterly) }

outputs:
  - { name: people_review,         format: rhythm-of-business@1 (people chapter), recipient: cuo/cpo-people + cuo/ceo + Board (people chapter) }

skill_chain:
  - { step: 1, skill: rhythm-of-business-author, inputs_from: { workforce_plan: workforce_plan, enps_data: enps_data, dei_progress: dei_progress }, outputs_to: review_draft }
  - { step: 2, skill: rhythm-of-business-audit,  inputs_from: review_draft, outputs_to: people_review }

audit_hooks:
  - workflow_complete row on PASS with people_review hash
---

# Quarterly people review — `chief-people-officer/quarterly-people-review`

CPO-People's consolidated quarterly people review across workforce + engagement + DEI.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.5
- `../../chief-human-resources-officer/workflows/` — upstream feeders
- `../../../skill/rhythm-of-business-{author,audit}/SKILL.md`
