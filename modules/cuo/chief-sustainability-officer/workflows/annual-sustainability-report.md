---
workflow_id: chief-sustainability-officer/annual-sustainability-report
workflow_version: 1.0.0
purpose: Author the annual sustainability-specific report — environmental performance, water/waste/biodiversity, supply-chain, climate adaptation.
persona: cuo/chief-sustainability-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_report,          source: last year's sustainability-report@1 (env-specific), format: sustainability-report@1 }
  - { name: emissions_inventory,   source: cuo/chief-sustainability-officer/annual-emissions-inventory, format: emissions-inventory@1 }
  - { name: environmental_data,    source: water + waste + biodiversity + chemical data, format: csv }
  - { name: target_progress,       source: SBTi / science-based targets tracking, format: markdown }

outputs:
  - { name: sustainability_report, format: sustainability-report@1, recipient: cuo/cso-sustainability + cuo/chief-esg-officer + investors + CDP }

skill_chain:
  - { step: 1, skill: sustainability-report-author, inputs_from: { prior_report: prior_report, emissions_inventory: emissions_inventory, environmental_data: environmental_data, target_progress: target_progress }, outputs_to: report_draft }
  - { step: 2, skill: sustainability-report-audit,  inputs_from: report_draft, outputs_to: sustainability_report }

escalates_to:
  - { persona: cuo/chief-esg-officer, when: "target miss requires escalation" }

audit_hooks:
  - workflow_complete row on PASS with sustainability_report hash
  - HITL pause at step 2 on QA-TARGET-001 (target progress unsourced)
---

# Annual sustainability report — `chief-sustainability-officer/annual-sustainability-report`

CSO-Sustainability's annual env-specific report per GHG Protocol + SBTi + CDP + TCFD scenario analysis + GRI environmental standards.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `./annual-emissions-inventory.md` — upstream feeder
- `../../../skill/sustainability-report-{author,audit}/SKILL.md`
