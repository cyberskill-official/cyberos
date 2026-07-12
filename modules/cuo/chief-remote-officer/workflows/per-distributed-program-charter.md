---
workflow_id: chief-remote-officer/per-distributed-program-charter
workflow_version: 1.0.0
purpose: Charter a distributed-team initiative — async-process rollout, remote-onboarding redesign, virtual-offsite, hub-and-spoke launch.
persona: cuo/chief-remote-officer
cadence: per-event
status: shipped

inputs:
  - { name: initiative_brief,      source: requestor, format: markdown }
  - { name: remote_policy,         source: cuo/chief-remote-officer/annual-remote-policy, format: remote-policy@1 }

outputs:
  - { name: distributed_charter,   format: program-charter@1, recipient: cuo/chief-remote-officer + program sponsor + cuo/chro }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { initiative_brief: initiative_brief, remote_policy: remote_policy }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: distributed_charter }

audit_hooks:
  - workflow_complete row on PASS with distributed_charter hash
  - HITL pause at step 2 on QA-OWNER-001
---

# Per distributed program charter — `chief-remote-officer/per-distributed-program-charter`

Chief Remote Officer's per-initiative charter for distributed-team programs.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../../skill/program-charter-{author,audit}/SKILL.md`
