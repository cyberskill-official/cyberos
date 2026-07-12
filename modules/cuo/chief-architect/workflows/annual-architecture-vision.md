---
workflow_id: chief-architect/annual-architecture-vision
workflow_version: 1.0.0
purpose: Author the annual architecture vision — reference architecture, principles, tech-radar, technical-debt portfolio, evolution roadmap.
persona: cuo/chief-architect
cadence: annual
status: shipped

inputs:
  - { name: prior_vision,          source: last year's strategy-document@1 (architecture chapter), format: strategy-document@1 }
  - { name: cto_priorities,        source: cuo/cto, format: markdown }
  - { name: tech_debt_inventory,   source: engineering tech-debt register, format: markdown }

outputs:
  - { name: architecture_vision,   format: strategy-doc@1, recipient: cuo/chief-architect + cuo/cto + engineering leads + Board (technical chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_vision: prior_vision, cto_priorities: cto_priorities, tech_debt_inventory: tech_debt_inventory }, outputs_to: vision_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: vision_draft, outputs_to: architecture_vision }

audit_hooks:
  - workflow_complete row on PASS with architecture_vision hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual architecture vision — `chief-architect/annual-architecture-vision`

Chief-Architect's annual architecture vision per ThoughtWorks Tech Radar + Gartner reference architectures + Mark Richards software-architecture-styles.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
