---
workflow_id: chief-medical-officer/per-medical-affairs-charter
workflow_version: 1.0.0
purpose: Charter a medical-affairs program — MSL initiative, evidence-generation study, KOL advisory board, medical-education program.
persona: cuo/chief-medical-officer
cadence: per-event
status: shipped

inputs:
  - { name: program_brief,         source: medical-affairs team, format: markdown }
  - { name: medical_strategy,      source: cuo/chief-medical-officer/annual-medical-strategy, format: strategy-doc@1 }

outputs:
  - { name: medical_affairs_charter, format: program-charter@1, recipient: cuo/chief-medical-officer + cuo/clo-legal (compliance) + program sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { program_brief: program_brief, medical_strategy: medical_strategy }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: medical_affairs_charter }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "KOL engagement triggers Sunshine Act / Physician Payments transparency" }

audit_hooks:
  - workflow_complete row on PASS with medical_affairs_charter hash
  - HITL pause at step 2 on QA-COMPLIANCE-001 (Sunshine Act applicability)
---

# Per medical affairs charter — `chief-medical-officer/per-medical-affairs-charter`

Chief Medical Officer's per-program charter for MSL / evidence / KOL / education initiatives per MAPS Medical Affairs Professional Society standards + Sunshine Act compliance.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/program-charter-{author,audit}/SKILL.md`
