---
workflow_id: chief-growth-officer/weekly-growth-cadence
workflow_version: 1.0.0
purpose: Weekly growth cadence — PQL conversion, viral-loop metrics, activation funnel, expansion signals.
persona: cuo/chief-growth-officer
cadence: weekly
status: shipped

inputs:
  - { name: pipeline,              source: cuo/chief-sales-officer/weekly-pipeline-review, format: pipeline-report@1 }
  - { name: product_metrics,       source: cuo/chief-product-officer/quarterly-product-metrics-review (latest weekly slice), format: product-metrics-review@1 }
  - { name: prior_cadence,         source: last week's rhythm-of-business@1, format: rhythm-of-business@1 }

outputs:
  - { name: growth_cadence,        format: rhythm-of-business@1 (growth chapter), recipient: cuo/cgo + cuo/cmo + cuo/cpo-product + cuo/cso-sales }

skill_chain:
  - { step: 1, skill: rhythm-of-business-author, inputs_from: { pipeline: pipeline, product_metrics: product_metrics, prior_cadence: prior_cadence }, outputs_to: cadence_draft }
  - { step: 2, skill: rhythm-of-business-audit,  inputs_from: cadence_draft, outputs_to: growth_cadence }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "PQL→customer conversion drops > 20% WoW" }

audit_hooks:
  - workflow_complete row on PASS with growth_cadence hash
  - HITL pause at step 2 on QA-CONVERSION-001
---

# Weekly growth cadence — `chief-growth-officer/weekly-growth-cadence`

CGO's weekly growth-engine cadence per OpenView PLG playbook + Reforge growth-loop framework. Distinct from CSO-Sales pipeline (sales-led) — CGO owns product-led acquisition + viral loops + expansion.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.1 — CGO role profile
- `../../chief-sales-officer/workflows/weekly-pipeline-review.md` — sales-led peer
- `../../../skill/rhythm-of-business-{author,audit}/SKILL.md`
