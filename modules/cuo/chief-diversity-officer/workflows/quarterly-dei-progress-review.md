---
workflow_id: chief-diversity-officer/quarterly-dei-progress-review
workflow_version: 1.0.0
purpose: Quarterly DEI progress review — representation deltas, pipeline funnel, inclusion-pulse, action-plan progress.
persona: cuo/chief-diversity-officer
cadence: quarterly
status: shipped

inputs:
  - { name: dei_program,           source: cuo/chief-diversity-officer/annual-dei-program, format: diversity-equity-inclusion-program@1 }
  - { name: prior_review,          source: last quarter's review, format: diversity-equity-inclusion-program@1 (quarterly chapter) }
  - { name: hr_demographics,       source: HRIS current demographics, format: csv }

outputs:
  - { name: dei_progress,          format: diversity-equity-inclusion-program@1 (quarterly chapter), recipient: cuo/cdo-diversity + cuo/chro + cuo/ceo + Board }

skill_chain:
  - { step: 1, skill: diversity-equity-inclusion-program-author, inputs_from: { dei_program: dei_program, prior_review: prior_review, hr_demographics: hr_demographics }, outputs_to: progress_draft }
  - { step: 2, skill: diversity-equity-inclusion-program-audit,  inputs_from: progress_draft, outputs_to: dei_progress }

audit_hooks:
  - workflow_complete row on PASS with dei_progress hash
  - HITL pause at step 2 on QA-PIPELINE-001
---

# Quarterly DEI progress review — `chief-diversity-officer/quarterly-dei-progress-review`

CDO-Diversity's quarterly progress review (broader than CHRO's pay-equity-focused peer).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `./annual-dei-program.md` — upstream parent
- `../../../skill/dei-program-{author,audit}/SKILL.md`
