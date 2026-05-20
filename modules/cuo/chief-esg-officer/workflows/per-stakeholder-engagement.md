---
workflow_id: chief-esg-officer/per-stakeholder-engagement
workflow_version: 1.0.0
purpose: Charter a stakeholder-engagement program — investor ESG calls, NGO partnerships, community programs, employee resource groups.
persona: cuo/chief-esg-officer
cadence: per-event
status: shipped

inputs:
  - { name: stakeholder_brief,     source: requestor, format: markdown }
  - { name: esg_strategy_context,  source: cuo/chief-esg-officer/annual-esg-strategy, format: strategy-document@1 }

outputs:
  - { name: stakeholder_charter,   format: program-charter@1, recipient: cuo/chief-esg-officer + program sponsor + cuo/cco-communications }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { stakeholder_brief: stakeholder_brief, esg_strategy_context: esg_strategy_context }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: stakeholder_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "engagement implies major partnership commitment" }

consults:
  - { persona: cuo/chief-communications-officer, when: "external comms coordination" }

audit_hooks:
  - workflow_complete row on PASS with stakeholder_charter hash
  - HITL pause at step 2 on QA-OWNER-001
---

# Per stakeholder engagement — `chief-esg-officer/per-stakeholder-engagement`

Chief ESG Officer's per-engagement charter per AccountAbility AA1000SES stakeholder engagement standard.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../../skill/program-charter-{author,audit}/SKILL.md`
