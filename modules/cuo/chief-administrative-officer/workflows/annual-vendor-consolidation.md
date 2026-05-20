---
workflow_id: chief-administrative-officer/annual-vendor-consolidation
workflow_version: 1.0.0
purpose: Drive annual vendor-consolidation initiative — overlap analysis, contract rationalization, savings opportunities, transition plan.
persona: cuo/chief-administrative-officer
cadence: annual
status: shipped

inputs:
  - { name: vendor_register,       source: AP + procurement system, format: csv }
  - { name: vendor_scorecard,      source: cuo/chief-operating-officer/quarterly-vendor-scorecard latest 4Q, format: vendor-scorecard@1 (4Q) }
  - { name: procurement_strategy,  source: cuo/chief-procurement-officer/annual-procurement-strategy, format: procurement-strategy@1 }

outputs:
  - { name: vendor_consolidation,  format: vendor-scorecard@1 (consolidation chapter), recipient: cuo/cao-admin + cuo/coo + cuo/cpo-procurement + cuo/cfo }

skill_chain:
  - { step: 1, skill: vendor-scorecard-author, inputs_from: { vendor_register: vendor_register, vendor_scorecard: vendor_scorecard, procurement_strategy: procurement_strategy }, outputs_to: consolidation_draft }
  - { step: 2, skill: vendor-scorecard-audit,  inputs_from: consolidation_draft, outputs_to: vendor_consolidation }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "consolidation savings target > $500K annual" }

audit_hooks:
  - workflow_complete row on PASS with vendor_consolidation hash + consolidation count + savings target
  - HITL pause at step 2 on QA-OVERLAP-001
---

# Annual vendor consolidation — `chief-administrative-officer/annual-vendor-consolidation`

CAO-Admin's annual vendor-consolidation initiative per Hackett shared-services + Bain G&A-optimization framework.

## Cross-references
- `../../../../modules/cuo/README.md` §5.1
- `../../chief-operating-officer/workflows/quarterly-vendor-scorecard.md` — upstream feeder
- `../../../skill/vendor-scorecard-{author,audit}/SKILL.md`
