---
workflow_id: chief-diversity-officer/per-erg-charter
workflow_version: 1.0.0
purpose: Charter an Employee Resource Group (ERG) — mission, sponsor, membership, budget, success criteria.
persona: cuo/chief-diversity-officer
cadence: per-event
status: shipped

inputs:
  - { name: erg_brief,             source: ERG founder, format: markdown }
  - { name: dei_program,           source: cuo/chief-diversity-officer/annual-dei-program, format: diversity-equity-inclusion-program@1 }

outputs:
  - { name: erg_charter,           format: program-charter@1, recipient: cuo/cdo-diversity + ERG sponsor + cuo/chro }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { erg_brief: erg_brief, dei_program: dei_program }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: erg_charter }

audit_hooks:
  - workflow_complete row on PASS with erg_charter hash
  - HITL pause at step 2 on QA-OWNER-001
---

# Per ERG charter — `chief-diversity-officer/per-erg-charter`

CDO-Diversity's per-ERG charter per Catalyst ERG-best-practices + McKinsey/LeanIn Women-in-the-Workplace.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../../skill/program-charter-{author,audit}/SKILL.md`
