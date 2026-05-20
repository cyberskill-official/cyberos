---
workflow_id: chief-sustainability-officer/annual-emissions-inventory
workflow_version: 1.0.0
purpose: Author the annual GHG emissions inventory — Scope 1/2/3 calculation, base-year recalculation, target progress, assurance readiness.
persona: cuo/chief-sustainability-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_inventory,       source: last year's emissions-inventory@1, format: emissions-inventory@1 }
  - { name: activity_data,         source: facilities + travel + purchased goods + use-phase data, format: csv }
  - { name: emission_factors,      source: GHG Protocol / IPCC / regional grid factors, format: markdown }
  - { name: assurance_scope,       source: third-party assurance plan, format: markdown }

outputs:
  - { name: emissions_inventory,   format: emissions-inventory@1, recipient: cuo/cso-sustainability + cuo/chief-esg-officer + assurance partner + regulators (CDP / EU ETS / SEC climate) }

skill_chain:
  - { step: 1, skill: emissions-inventory-author, inputs_from: { prior_inventory: prior_inventory, activity_data: activity_data, emission_factors: emission_factors, assurance_scope: assurance_scope }, outputs_to: inventory_draft }
  - { step: 2, skill: emissions-inventory-audit,  inputs_from: inventory_draft, outputs_to: emissions_inventory }

escalates_to:
  - { persona: cuo/chief-esg-officer, when: "Scope 3 increase > 10% triggers materiality review" }
  - { persona: cuo/chief-legal-officer,      when: "discrepancy triggers regulator restatement obligation" }

consults:
  - { persona: cuo/chief-procurement-officer, when: "Scope 3 Cat 1 (purchased goods) needs supplier data" }
  - { persona: cuo/chief-accounting-officer, when: "financial controls intersect (CSRD limited assurance)" }

audit_hooks:
  - workflow_complete row on PASS with emissions_inventory hash + total CO2e + Scope split
  - HITL pause at step 2 on QA-FACTOR-001 (factor source unclear) or QA-SCOPE3-001 (Cat 1-15 incomplete)
---

# Annual emissions inventory — `chief-sustainability-officer/annual-emissions-inventory`

Chief Sustainability Officer's annual GHG inventory per GHG Protocol Corporate Standard + ISO 14064-1 + IPCC AR6 factors. Feeds CSRD/ESRS, CDP disclosure, and SBTi target tracking.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-esg-officer/workflows/annual-esg-report.md` — downstream consumer
- `../../../skill/emissions-inventory-{author,audit}/SKILL.md`
