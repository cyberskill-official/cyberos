---
workflow_id: chief-automation-officer/quarterly-automation-portfolio-review
workflow_version: 1.0.0
purpose: Review automation-program portfolio — bot health, ROI realization, failure modes, optimization opportunities.
persona: cuo/chief-automation-officer
cadence: quarterly
status: shipped

inputs:
  - { name: roadmap,               source: cuo/chief-automation-officer/annual-automation-roadmap, format: automation-roadmap@1 }
  - { name: bot_telemetry,         source: RPA platform telemetry (UiPath Orchestrator / etc.), format: csv }
  - { name: roi_actuals,           source: per-process ROI realization data, format: csv }

outputs:
  - { name: portfolio_review,      format: automation-roadmap@1 (quarterly chapter), recipient: cuo/chief-automation-officer + cuo/coo + cuo/ceo + Board (automation chapter) }

skill_chain:
  - { step: 1, skill: automation-roadmap-author, inputs_from: { roadmap: roadmap, bot_telemetry: bot_telemetry, roi_actuals: roi_actuals }, outputs_to: review_draft }
  - { step: 2, skill: automation-roadmap-audit,  inputs_from: review_draft, outputs_to: portfolio_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "ROI realization < 50% of plan OR > 3 bots failed > 1 quarter" }

audit_hooks:
  - workflow_complete row on PASS with portfolio_review hash + bot-failure count + ROI %
  - HITL pause at step 2 on QA-FAILURE-001
---

# Quarterly automation portfolio review — `chief-automation-officer/quarterly-automation-portfolio-review`

Chief Automation Officer's quarterly review per UiPath Center of Excellence playbook + Gartner Hyper-automation tracking.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./annual-automation-roadmap.md` — upstream parent
- `../../../skill/automation-roadmap-{author,audit}/SKILL.md`
