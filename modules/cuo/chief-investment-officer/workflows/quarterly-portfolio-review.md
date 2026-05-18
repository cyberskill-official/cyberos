---
workflow_id: chief-investment-officer/quarterly-portfolio-review
workflow_version: 1.0.0
purpose: Review portfolio — position attribution, risk exposures, thesis re-validation, rebalance decisions.
persona: cuo/chief-investment-officer
cadence: quarterly
status: shipped

inputs:
  - { name: investment_strategy,   source: cuo/chief-investment-officer/annual-investment-strategy, format: strategy-document@1 }
  - { name: position_attribution,  source: per-position P&L attribution, format: csv }
  - { name: thesis_inventory,      source: all active investment-thesis@1, format: investment-thesis@1 (set) }

outputs:
  - { name: portfolio_review,      format: strategy-document@1 (quarterly chapter), recipient: cuo/cio-investment + investment committee + LP communications }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { investment_strategy: investment_strategy, position_attribution: position_attribution, thesis_inventory: thesis_inventory }, outputs_to: review_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: review_draft, outputs_to: portfolio_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "drawdown > IPS threshold OR thesis-validation failure on top-10 position" }

audit_hooks:
  - workflow_complete row on PASS with portfolio_review hash
  - HITL pause at step 2 on QA-EXPOSURE-001 (risk exposure outside IPS)
---

# Quarterly portfolio review — `chief-investment-officer/quarterly-portfolio-review`

CIO-Investment's quarterly portfolio review per Brinson-Hood-Beebower attribution + factor-based risk decomposition (Fama-French / Barra).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./per-investment-thesis.md` — upstream thesis feeders
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
