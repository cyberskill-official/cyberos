---
workflow_id: chief-human-resources-officer/quarterly-dei-program-review
workflow_version: 1.0.0
purpose: Run the quarterly DEI program review — representation metrics, pipeline diversity, pay-equity audit, inclusion-survey synthesis, action plan.
persona: cuo/chief-human-resources-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_program,      source: last quarter's dei-program@1,             format: dei-program@1 }
  - { name: hr_demographics,    source: HRIS demographic + tenure + level data,   format: csv (anonymized aggregates) }
  - { name: pipeline_data,      source: ATS demographic data per stage,           format: csv }
  - { name: inclusion_survey,   source: Culture Amp / Lattice DEI-pulse module,   format: csv export }

outputs:
  - { name: dei_program,        format: dei-program@1, recipient: cuo/chro + cuo/ceo + Board (DEI chapter) + cuo/cdo-diversity (if exists) }

skill_chain:
  - { step: 1, skill: dei-program-author, inputs_from: { prior_program: prior_program, hr_demographics: hr_demographics, pipeline_data: pipeline_data, inclusion_survey: inclusion_survey }, outputs_to: program_draft }
  - { step: 2, skill: dei-program-audit,  inputs_from: program_draft, outputs_to: dei_program }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "representation drops > 5pts in any reported segment OR pay-equity gap widens" }
  - { persona: cuo/chief-legal-officer,   when: "pay-equity gap may trigger EEO / pay-transparency disclosure obligation" }

consults:
  - { persona: cuo/chief-diversity-officer, when: "this persona exists; otherwise CHRO owns directly" }
  - { persona: cuo/chief-communications-officer, when: "external DEI report needs PR positioning" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with dei_program hash + representation-deltas + pay-equity gap + action-count
  - HITL pause at step 2 on QA-PAY-EQUITY-001 (gap unexplained) or QA-PIPELINE-001 (stage-by-stage funnel leakage unflagged)
---

# Quarterly DEI program review — `chief-human-resources-officer/quarterly-dei-program-review`

CHRO's quarterly DEI program-review workflow. Combines prior program + HR demographics + pipeline data + inclusion-survey into representation + pipeline-diversity + pay-equity + inclusion analysis + action plan. Per SHRM DEI framework + McKinsey Diversity Matters research + EEOC reporting standards.

## When to invoke

- "Run the Q<n> DEI program review"
- "Quarterly DEI metrics"
- "Diversity + inclusion progress check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-human-resources-officer/quarterly-dei-program-review \
  --input prior_program=./hr/2026-Q1/dei.md \
  --input hr_demographics=./hr/2026-Q2/demographics.csv \
  --input pipeline_data=./hr/2026-Q2/ats-demographics.csv \
  --input inclusion_survey=./hr/2026-Q2/inclusion-survey.csv \
  --output-dir ./hr/2026-Q2/dei/
```

## Expected duration

- **Happy path:** 1-3 hours runtime + 1-2 weeks for survey analysis + manager round-trip
- **Worst case:** pay-equity gap discovery triggers remediation + 1-quarter intervention program

## Skill chain

- **Step 1 `dei-program-author`** — drafts per SHRM DEI framework + McKinsey Diversity Matters + EEOC.
- **Step 2 `dei-program-audit`** — validates per `dei_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PAY-EQUITY-001 | Gap unexplained | Escalate to CLO-Legal for disclosure assessment |
| 2 | QA-PIPELINE-001 | Funnel leakage unflagged | Operator surfaces |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5 — CHRO role profile
- `../../cdo-diversity/README.md` — peer/dedicated DEI persona where exists
- `../../../skill/dei-program-{author,audit}/SKILL.md`
