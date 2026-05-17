---
workflow_id: chief-restructuring-officer/per-stakeholder-communication
workflow_version: 1.0.0
purpose: Coordinate stakeholder communication during distress — lenders, employees, customers, suppliers, board.
persona: cuo/chief-restructuring-officer
cadence: per-event
status: shipped

inputs:
  - { name: communication_brief,   source: CRO situation update, format: markdown }
  - { name: stakeholder_map,       source: stakeholder register, format: markdown }
  - { name: prior_comms,           source: similar prior decision-log@1, format: decision-log@1 (set) }

outputs:
  - { name: stakeholder_decision,  format: decision-log@1, recipient: cuo/cro-restructuring + cuo/clo-legal + cuo/cco-communications + Board }

skill_chain:
  - { step: 1, skill: decision-log-author, inputs_from: { communication_brief: communication_brief, stakeholder_map: stakeholder_map, prior_comms: prior_comms }, outputs_to: log_draft }
  - { step: 2, skill: decision-log-audit,  inputs_from: log_draft, outputs_to: stakeholder_decision }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "communication touches Reg FD / lender notification obligations" }

audit_hooks:
  - workflow_complete row on PASS with stakeholder_decision hash
  - HITL pause at step 2 on QA-OWNER-001 or QA-LEGAL-001
---

# Per stakeholder communication — `chief-restructuring-officer/per-stakeholder-communication`

CRO-Restructuring's per-event stakeholder communication during distress. Decisions logged for fiduciary-duty record per Delaware Chancery zone-of-insolvency case law.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/decision-log-{author,audit}/SKILL.md`
