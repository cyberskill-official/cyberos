---
workflow_id: chief-automation-officer/annual-automation-roadmap
workflow_version: 1.0.0
purpose: Author the annual automation roadmap — RPA pipeline, AI-augmented automation, hyper-automation, ROI targets.
persona: cuo/chief-automation-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_roadmap,         source: last year's automation-roadmap@1, format: automation-roadmap@1 }
  - { name: process_inventory,     source: operating-model process catalog, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: ai_strategy_context,   source: cuo/chief-ai-officer/annual-ai-strategy, format: ai-strategy@1 }

outputs:
  - { name: automation_roadmap,    format: automation-roadmap@1, recipient: cuo/chief-automation-officer + cuo/coo + cuo/caio + cuo/cto + Board (automation chapter) }

skill_chain:
  - { step: 1, skill: automation-roadmap-author, inputs_from: { prior_roadmap: prior_roadmap, process_inventory: process_inventory, ceo_priorities: ceo_priorities, ai_strategy_context: ai_strategy_context }, outputs_to: roadmap_draft }
  - { step: 2, skill: automation-roadmap-audit,  inputs_from: roadmap_draft, outputs_to: automation_roadmap }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "roadmap proposes role redesign affecting > 100 FTE" }
  - { persona: cuo/chief-human-resources-officer,           when: "automation displaces roles — change-management needed" }

consults:
  - { persona: cuo/chief-ai-officer,           when: "AI-augmented automation overlaps AI use-case portfolio" }
  - { persona: cuo/chief-technology-officer,            when: "platform integration" }

audit_hooks:
  - workflow_complete row on PASS with automation_roadmap hash + process count + ROI envelope
  - HITL pause at step 2 on QA-ROI-001 (ROI claim not underwritten) or QA-DISPLACEMENT-001 (FTE displacement without change plan)
---

# Annual automation roadmap — `chief-automation-officer/annual-automation-roadmap`

Chief Automation Officer's annual roadmap per Gartner Hyper-automation framework + UiPath / Automation Anywhere / Blue Prism vendor architecture + EY intelligent-automation maturity model.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../chief-ai-officer/workflows/annual-ai-strategy.md` — peer (AI strategy informs automation)
- `../../../skill/automation-roadmap-{author,audit}/SKILL.md`
