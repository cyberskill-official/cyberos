---
workflow_id: chief-digital-officer/quarterly-digital-portfolio-review
workflow_version: 1.0.0
purpose: Review the digital-program portfolio — program health, channel performance, customer experience metrics, value realization.
persona: cuo/chief-digital-officer
cadence: quarterly
status: shipped

inputs:
  - { name: roadmap,               source: cuo/chief-digital-officer/annual-digital-transformation-roadmap, format: transformation-roadmap@1 }
  - { name: program_status,        source: program leads, format: markdown }
  - { name: digital_metrics,       source: channel telemetry, format: csv }

outputs:
  - { name: digital_portfolio_review, format: transformation-roadmap@1 (quarterly chapter), recipient: cuo/chief-digital-officer + cuo/ceo + Board (digital chapter) }

skill_chain:
  - { step: 1, skill: transformation-roadmap-author, inputs_from: { roadmap: roadmap, program_status: program_status, digital_metrics: digital_metrics }, outputs_to: review_draft }
  - { step: 2, skill: transformation-roadmap-audit,  inputs_from: review_draft, outputs_to: digital_portfolio_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "red program no recovery OR channel CX degradation > 10%" }

audit_hooks:
  - workflow_complete row on PASS with digital_portfolio_review hash + red-count
  - HITL pause at step 2 on QA-RED-001
---

# Quarterly digital portfolio review — `chief-digital-officer/quarterly-digital-portfolio-review`

CDO-Digital's quarterly portfolio review per Bain digital-program-management framework. Aggregates program health + channel performance + CX metrics.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./annual-digital-transformation-roadmap.md` — upstream parent
- `../../../skill/transformation-roadmap-{author,audit}/SKILL.md`
