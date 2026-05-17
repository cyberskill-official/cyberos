---
workflow_id: chief-digital-officer/annual-digital-transformation-roadmap
workflow_version: 1.0.0
purpose: Author the annual digital-transformation roadmap — digital experience vision, channel modernization, platform investments, data + AI integration.
persona: cuo/chief-digital-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_roadmap,         source: last year's transformation-roadmap@1 (digital chapter), format: transformation-roadmap@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }
  - { name: digital_metrics,       source: digital-channel telemetry (web/mobile/app/IoT), format: csv }
  - { name: customer_signals,      source: cuo/cco-customer + NPS verbatims, format: markdown }

outputs:
  - { name: digital_transformation_roadmap, format: transformation-roadmap@1, recipient: cuo/chief-digital-officer + cuo/cto + cuo/cpo-product + cuo/ceo + Board (digital chapter) }

skill_chain:
  - { step: 1, skill: transformation-roadmap-author, inputs_from: { prior_roadmap: prior_roadmap, ceo_priorities: ceo_priorities, digital_metrics: digital_metrics, customer_signals: customer_signals }, outputs_to: roadmap_draft }
  - { step: 2, skill: transformation-roadmap-audit,  inputs_from: roadmap_draft, outputs_to: digital_transformation_roadmap }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "roadmap proposes platform replatform > $1M OR channel deprecation" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "platform decisions" }
  - { persona: cuo/chief-product-officer,    when: "product surface overlap" }
  - { persona: cuo/chief-marketing-officer,            when: "digital marketing channels" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with digital_transformation_roadmap hash + program count
  - HITL pause at step 2 on QA-VALUE-001 or QA-CHANNEL-001
---

# Annual digital transformation roadmap — `chief-digital-officer/annual-digital-transformation-roadmap`

CDO-Digital's annual digital transformation per Gartner Digital Business Transformation framework + McKinsey Digital Quotient + MIT CISR digital-mastery framework. Combines prior roadmap + CEO priorities + digital metrics + customer signals into vision + channels + platforms + data/AI integration.

## When to invoke
- "Build the 2026 digital transformation roadmap"
- "Annual digital strategic refresh"

## Skill chain
- **Step 1 `transformation-roadmap-author`** — drafts per Gartner DBT + McKinsey DQ + MIT CISR.
- **Step 2 `transformation-roadmap-audit`** — validates per rubric.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7 — Chief Digital Officer role profile
- `../../chief-transformation-officer/workflows/annual-transformation-roadmap.md` — peer (broader transformation)
- `../../../skill/transformation-roadmap-{author,audit}/SKILL.md`
