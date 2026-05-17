---
workflow_id: chief-commercial-officer/per-strategic-partnership
workflow_version: 1.0.0
purpose: Charter a strategic partnership — partner profile, value hypothesis, joint-investment plan, governance.
persona: cuo/chief-commercial-officer
cadence: per-event
status: shipped

inputs:
  - { name: partnership_brief,     source: partnerships team, format: markdown }
  - { name: program_context,       source: cuo/chief-commercial-officer/annual-partner-program, format: partner-program@1 }
  - { name: prior_partnerships,    source: similar prior partnership charters, format: program-charter@1 (set) }

outputs:
  - { name: partnership_charter,   format: program-charter@1, recipient: cuo/cco-commercial + cuo/clo-legal + partnership sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { partnership_brief: partnership_brief, program_context: program_context, prior_partnerships: prior_partnerships }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: partnership_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "partnership > $1M annual commitment OR exclusivity terms" }
  - { persona: cuo/chief-legal-officer,      when: "partnership requires custom legal structure" }

audit_hooks:
  - workflow_complete row on PASS with partnership_charter hash
  - HITL pause at step 2 on QA-VALUE-001 or QA-OWNER-001
---

# Per strategic partnership — `chief-commercial-officer/per-strategic-partnership`

CCO-Commercial's per-partnership charter per Crossbeam partner-economics + TSIA value-engineering framework.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4
- `./annual-partner-program.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
