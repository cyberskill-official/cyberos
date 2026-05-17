---
workflow_id: chief-sustainability-officer/quarterly-target-tracking
workflow_version: 1.0.0
purpose: Track quarterly progress against sustainability targets — net-zero, science-based targets, renewable energy, circular economy.
persona: cuo/chief-sustainability-officer
cadence: quarterly
status: shipped

inputs:
  - { name: emissions_inventory,   source: cuo/chief-sustainability-officer/annual-emissions-inventory, format: emissions-inventory@1 }
  - { name: prior_tracking,        source: last quarter's tracking, format: emissions-inventory@1 (quarterly chapter) }
  - { name: target_baselines,      source: SBTi-validated targets + internal commitments, format: markdown }

outputs:
  - { name: target_tracking,       format: emissions-inventory@1 (quarterly chapter), recipient: cuo/cso-sustainability + cuo/chief-esg-officer + cuo/ceo (if material miss) }

skill_chain:
  - { step: 1, skill: emissions-inventory-author, inputs_from: { emissions_inventory: emissions_inventory, prior_tracking: prior_tracking, target_baselines: target_baselines }, outputs_to: tracking_draft }
  - { step: 2, skill: emissions-inventory-audit,  inputs_from: tracking_draft, outputs_to: target_tracking }

escalates_to:
  - { persona: cuo/chief-esg-officer, when: "target trajectory off > 5% from plan" }

audit_hooks:
  - workflow_complete row on PASS with target_tracking hash
  - HITL pause at step 2 on QA-TARGET-001
---

# Quarterly target tracking — `chief-sustainability-officer/quarterly-target-tracking`

CSO-Sustainability's quarterly target-progress tracking per SBTi monitoring + CDP scoring.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/emissions-inventory-{author,audit}/SKILL.md`
