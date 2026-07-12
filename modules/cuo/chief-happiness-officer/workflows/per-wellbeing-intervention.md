---
workflow_id: chief-happiness-officer/per-wellbeing-intervention
workflow_version: 1.0.0
purpose: Charter a wellbeing intervention — burnout-prevention program, mental-health support, recognition campaign, culture-fix initiative.
persona: cuo/chief-happiness-officer
cadence: per-event
status: shipped

inputs:
  - { name: intervention_brief,    source: requestor (manager / HR / employee), format: markdown }
  - { name: prior_interventions,   source: similar prior program-charter@1, format: program-charter@1 (set) }

outputs:
  - { name: wellbeing_charter,     format: program-charter@1, recipient: cuo/chief-happiness-officer + cuo/chro + program sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { intervention_brief: intervention_brief, prior_interventions: prior_interventions }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: wellbeing_charter }

audit_hooks:
  - workflow_complete row on PASS with wellbeing_charter hash
  - HITL pause at step 2 on QA-OWNER-001
---

# Per wellbeing intervention — `chief-happiness-officer/per-wellbeing-intervention`

Chief Happiness Officer's per-intervention charter per Shawn Achor Happiness Advantage + Adam Grant + Mind Share Partners workplace-mental-health framework.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.5
- `../../../skill/program-charter-{author,audit}/SKILL.md`
