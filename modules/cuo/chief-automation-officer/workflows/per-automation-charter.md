---
workflow_id: chief-automation-officer/per-automation-charter
workflow_version: 1.0.0
purpose: Charter a per-process automation initiative — process-mining baseline, automation design, ROI hypothesis, change-management plan.
persona: cuo/chief-automation-officer
cadence: per-event
status: shipped

inputs:
  - { name: process_brief,         source: process owner, format: markdown }
  - { name: roadmap_context,       source: cuo/chief-automation-officer/annual-automation-roadmap, format: automation-roadmap@1 }
  - { name: process_mining_data,   source: Celonis / UiPath Process Mining / ABBYY Timeline, format: csv }

outputs:
  - { name: automation_charter,    format: program-charter@1, recipient: cuo/chief-automation-officer + process owner + cuo/coo + cuo/chro }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { process_brief: process_brief, roadmap_context: roadmap_context, process_mining_data: process_mining_data }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: automation_charter }

escalates_to:
  - { persona: cuo/chief-human-resources-officer,           when: "automation affects > 5 FTE" }
  - { persona: cuo/chief-compliance-officer, when: "automation touches compliance-controlled process" }

audit_hooks:
  - workflow_complete row on PASS with automation_charter hash + ROI hypothesis
  - HITL pause at step 2 on QA-OWNER-001 or QA-ROI-001
---

# Per automation charter — `chief-automation-officer/per-automation-charter`

Chief Automation Officer's per-process charter per Celonis process-mining + UiPath automation lifecycle + Gartner DigitalOps.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `./annual-automation-roadmap.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
