---
workflow_id: chief-growth-officer/quarterly-experimentation-portfolio
workflow_version: 1.0.0
purpose: Review growth-experimentation portfolio — experiment hypotheses, ICE/RICE scoring, results, learnings, next-quarter queue.
persona: cuo/chief-growth-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_experiments,     source: last quarter's experiment log, format: program-charter@1 (set) }
  - { name: results,               source: experiment-platform results (Optimizely / VWO / Statsig), format: csv }
  - { name: growth_strategy,       source: cuo/chief-growth-officer/annual-growth-strategy, format: gtm-plan@1 }

outputs:
  - { name: experimentation_portfolio, format: program-charter@1 (portfolio summary), recipient: cuo/cgo + cuo/cpo-product + cuo/cmo }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { prior_experiments: prior_experiments, results: results, growth_strategy: growth_strategy }, outputs_to: portfolio_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: portfolio_draft, outputs_to: experimentation_portfolio }

escalates_to:
  - { persona: cuo/chief-product-officer,    when: "winning experiments require product-roadmap commitment" }

audit_hooks:
  - workflow_complete row on PASS with experimentation_portfolio hash + experiments count
  - HITL pause at step 2 on QA-STATSIG-001 (results without statistical significance)
---

# Quarterly experimentation portfolio — `chief-growth-officer/quarterly-experimentation-portfolio`

CGO's quarterly growth-experimentation review per Reforge experimentation + Sean Ellis ICE / RICE prioritization + Lean Analytics.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.1
- `./annual-growth-strategy.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
