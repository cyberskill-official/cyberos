---
workflow_id: chief-learning-officer/per-leadership-development-program
workflow_version: 1.0.0
purpose: Charter a leadership-development cohort — emerging-leader / mid-manager / executive-readiness program.
persona: cuo/chief-learning-officer
cadence: per-event
status: shipped

inputs:
  - { name: cohort_brief,          source: CHRO + CEO sponsor, format: markdown }
  - { name: succession_plan,       source: succession-planning artefact, format: markdown }
  - { name: learning_program,      source: cuo/chief-learning-officer/annual-learning-program, format: onboarding-pack@1 }

outputs:
  - { name: leadership_dev_charter, format: program-charter@1, recipient: cuo/clo-learning + cuo/chro + cuo/ceo + cohort sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { cohort_brief: cohort_brief, succession_plan: succession_plan, learning_program: learning_program }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: leadership_dev_charter }

audit_hooks:
  - workflow_complete row on PASS with leadership_dev_charter hash
  - HITL pause at step 2 on QA-OWNER-001 or QA-OUTCOME-001
---

# Per leadership development program — `chief-learning-officer/per-leadership-development-program`

CLO-Learning's per-cohort leadership-development charter per CCL Center for Creative Leadership + Korn Ferry leadership-architect + DDI Development Dimensions International.

## Cross-references
- `../../../../modules/cuo/README.md` §5.5
- `../../../skill/program-charter-{author,audit}/SKILL.md`
