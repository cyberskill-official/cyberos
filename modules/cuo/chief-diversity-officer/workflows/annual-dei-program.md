---
workflow_id: chief-diversity-officer/annual-dei-program
workflow_version: 1.0.0
purpose: Author the annual DEI program — representation goals, pipeline diversity, inclusion metrics, ERG programs, equity audit.
persona: cuo/chief-diversity-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,         source: last year's dei-program@1, format: diversity-equity-inclusion-program@1 }
  - { name: hr_demographics,       source: HRIS demographic data, format: csv }
  - { name: pay_equity_audit,      source: prior-year pay-equity analysis, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: dei_program,           format: dei-program@1, recipient: cuo/cdo-diversity + cuo/chro + cuo/ceo + Board (annual DEI chapter) }

skill_chain:
  - { step: 1, skill: diversity-equity-inclusion-program-author, inputs_from: { prior_program: prior_program, hr_demographics: hr_demographics, pay_equity_audit: pay_equity_audit, ceo_priorities: ceo_priorities }, outputs_to: program_draft }
  - { step: 2, skill: diversity-equity-inclusion-program-audit,  inputs_from: program_draft, outputs_to: dei_program }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "pay-equity gap may trigger disclosure obligation" }

audit_hooks:
  - workflow_complete row on PASS with dei_program hash
  - HITL pause at step 2 on QA-PAY-EQUITY-001
---

# Annual DEI program — `chief-diversity-officer/annual-dei-program`

CDO-Diversity's annual DEI program per SHRM DEI framework + McKinsey Diversity Matters + EEOC + Catalyst CDO playbook.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5 — CDO-Diversity role profile
- `../../chief-human-resources-officer/workflows/quarterly-dei-program-review.md` — CHRO quarterly peer
- `../../../skill/dei-program-{author,audit}/SKILL.md`
