---
workflow_id: chief-security-officer/per-physical-security-charter
workflow_version: 1.0.0
purpose: Charter a physical-security program — facility upgrade, executive protection, supply-chain security, insider-threat monitoring.
persona: cuo/chief-security-officer
cadence: per-event
status: shipped

inputs:
  - { name: program_brief,         source: requestor, format: markdown }
  - { name: converged_strategy,    source: cuo/chief-security-officer/annual-converged-security-strategy, format: security-strategy@1 }

outputs:
  - { name: physical_security_charter, format: program-charter@1, recipient: cuo/cso-security + cuo/cao-admin + cuo/clo-legal + program sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { program_brief: program_brief, converged_strategy: converged_strategy }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: physical_security_charter }

audit_hooks:
  - workflow_complete row on PASS with physical_security_charter hash
  - HITL pause at step 2 on QA-OWNER-001
---

# Per physical security charter — `chief-security-officer/per-physical-security-charter`

CSO-Security's per-program physical-security charter per ASIS standards + CPP (Certified Protection Professional) practices.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../../skill/program-charter-{author,audit}/SKILL.md`
