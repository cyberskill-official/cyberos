---
workflow_id: chief-happiness-officer/quarterly-happiness-program
workflow_version: 1.0.0
purpose: Refresh the quarterly happiness program — engagement surveys, recognition programs, well-being initiatives, action plans.
persona: cuo/chief-happiness-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_program,         source: last quarter's happiness-program@1, format: happiness-program@1 }
  - { name: enps_data,             source: cuo/chief-human-resources-officer/quarterly-enps-pulse, format: employee-net-promoter-score-program@1 }
  - { name: wellbeing_metrics,     source: Officevibe / TINYpulse / Culture Amp wellbeing module, format: csv }

outputs:
  - { name: happiness_program,     format: happiness-program@1, recipient: cuo/chief-happiness-officer + cuo/chro + cuo/ceo + all managers }

skill_chain:
  - { step: 1, skill: happiness-program-author, inputs_from: { prior_program: prior_program, enps_data: enps_data, wellbeing_metrics: wellbeing_metrics }, outputs_to: program_draft }
  - { step: 2, skill: happiness-program-audit,  inputs_from: program_draft, outputs_to: happiness_program }

escalates_to:
  - { persona: cuo/chief-human-resources-officer,           when: "wellbeing index drops > 10pts QoQ" }

audit_hooks:
  - workflow_complete row on PASS with happiness_program hash + action count
  - HITL pause at step 2 on QA-ACTION-001
---

# Quarterly happiness program — `chief-happiness-officer/quarterly-happiness-program`

Chief Happiness Officer's quarterly program per Shawn Achor + Officevibe + TINYpulse + Culture Amp wellbeing standards. Critical for Series-A scale-up where culture is the strongest retention lever.

## Cross-references
- `../../../../modules/cuo/README.md` §5.5 — Chief Happiness Officer role profile
- `../../chief-human-resources-officer/workflows/quarterly-enps-pulse.md` — upstream feeder
- `../../../skill/happiness-program-{author,audit}/SKILL.md`
