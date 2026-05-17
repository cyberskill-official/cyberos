---
workflow_id: chief-learning-officer/quarterly-learning-effectiveness
workflow_version: 1.0.0
purpose: Review learning-effectiveness data — completion rates, satisfaction, behavior change, business impact (Kirkpatrick levels 1-4).
persona: cuo/chief-learning-officer
cadence: quarterly
status: shipped

inputs:
  - { name: lms_data,              source: LMS (Workday Learning / Cornerstone / Docebo / 360Learning) telemetry, format: csv }
  - { name: enps_data,             source: cuo/chief-human-resources-officer/quarterly-enps-pulse (learning slices), format: enps-program@1 }
  - { name: prior_review,          source: last quarter's effectiveness review, format: enps-program@1 (learning chapter) }

outputs:
  - { name: learning_effectiveness, format: enps-program@1 (learning chapter), recipient: cuo/clo-learning + cuo/chro }

skill_chain:
  - { step: 1, skill: enps-program-author, inputs_from: { lms_data: lms_data, enps_data: enps_data, prior_review: prior_review }, outputs_to: review_draft }
  - { step: 2, skill: enps-program-audit,  inputs_from: review_draft, outputs_to: learning_effectiveness }

audit_hooks:
  - workflow_complete row on PASS with learning_effectiveness hash
  - HITL pause at step 2 on QA-KIRKPATRICK-001 (Level 3-4 evidence missing)
---

# Quarterly learning effectiveness — `chief-learning-officer/quarterly-learning-effectiveness`

CLO-Learning's quarterly effectiveness review per Kirkpatrick model + Phillips ROI methodology + ATD impact measurement.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../../skill/enps-program-{author,audit}/SKILL.md`
