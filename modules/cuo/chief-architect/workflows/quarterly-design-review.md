---
workflow_id: chief-architect/quarterly-design-review
workflow_version: 1.0.0
purpose: Review SDDs across active projects — design coherence, ADR alignment, NFR coverage, architecture-debt assessment.
persona: cuo/chief-architect
cadence: quarterly
status: shipped

inputs:
  - { name: active_sdds,           source: engineering team (all in-flight SDDs), format: software-design-document@1 (set) }
  - { name: architecture_vision,   source: cuo/chief-architect/annual-architecture-vision, format: strategy-document@1 }
  - { name: adr_history,           source: ADR repository, format: architecture-decision-record@1 (set) }

outputs:
  - { name: design_review,         format: software-design-document@1 (review chapter), recipient: cuo/chief-architect + cuo/cto + engineering leads }

skill_chain:
  - { step: 1, skill: software-design-document-author, inputs_from: { active_sdds: active_sdds, architecture_vision: architecture_vision, adr_history: adr_history }, outputs_to: review_draft }
  - { step: 2, skill: software-design-document-audit,  inputs_from: review_draft, outputs_to: design_review }

audit_hooks:
  - workflow_complete row on PASS with design_review hash
  - HITL pause at step 2 on QA-ADR-001 (SDD contradicts accepted ADR)
---

# Quarterly design review — `chief-architect/quarterly-design-review`

Chief-Architect's quarterly cross-SDD review per IEEE 1016 + ISO/IEC 42010 + Software Engineering Institute Architecture Tradeoff Analysis Method (ATAM).

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3
- `../../../skill/sdd-{author,audit}/SKILL.md`
