---
workflow_id: chief-procurement-officer/quarterly-savings-tracker
workflow_version: 1.0.0
purpose: Track quarterly procurement savings — realized vs target, by category and savings type.
persona: cuo/chief-procurement-officer
cadence: quarterly
status: shipped

inputs:
  - { name: strategy,              source: cuo/chief-procurement-officer/annual-procurement-strategy, format: procurement-strategy@1 }
  - { name: spend_actuals,         source: AP YoY data, format: csv }
  - { name: prior_tracking,        source: last quarter's tracker, format: procurement-strategy@1 (quarterly chapter) }

outputs:
  - { name: savings_tracker,       format: procurement-strategy@1 (quarterly chapter), recipient: cuo/cpo-procurement + cuo/cfo + cuo/ceo (if material miss) }

skill_chain:
  - { step: 1, skill: procurement-strategy-author, inputs_from: { strategy: strategy, spend_actuals: spend_actuals, prior_tracking: prior_tracking }, outputs_to: tracker_draft }
  - { step: 2, skill: procurement-strategy-audit,  inputs_from: tracker_draft, outputs_to: savings_tracker }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "savings realization < 70% of plan" }

audit_hooks:
  - workflow_complete row on PASS with savings_tracker hash + realized vs target %
  - HITL pause at step 2 on QA-METHODOLOGY-001 (savings calc method unclear)
---

# Quarterly savings tracker — `chief-procurement-officer/quarterly-savings-tracker`

CPO-Procurement's quarterly savings tracker per Hackett spend-analytics + APQC procurement-benchmarking + ProcureCon savings-classification.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./annual-procurement-strategy.md` — upstream parent
- `../../../skill/procurement-strategy-{author,audit}/SKILL.md`
