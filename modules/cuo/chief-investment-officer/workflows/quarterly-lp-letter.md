---
workflow_id: chief-investment-officer/quarterly-lp-letter
workflow_version: 1.0.0
purpose: Author the quarterly LP letter — performance, positioning, market commentary, capital activity.
persona: cuo/chief-investment-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_letter,          source: last quarter's lp-letter@1, format: limited-partner-letter@1 }
  - { name: performance_data,      source: fund-administrator NAV + attribution, format: csv }
  - { name: market_commentary,     source: market intel, format: markdown }

outputs:
  - { name: lp_letter,             format: lp-letter@1, recipient: cuo/cio-investment + LPs + GP team }

skill_chain:
  - { step: 1, skill: limited-partner-letter-author, inputs_from: { prior_letter: prior_letter, performance_data: performance_data, market_commentary: market_commentary }, outputs_to: letter_draft }
  - { step: 2, skill: limited-partner-letter-audit,  inputs_from: letter_draft, outputs_to: lp_letter }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "letter requires forward-looking-statement disclaimer (SEC IA Marketing Rule)" }

audit_hooks:
  - workflow_complete row on PASS with lp_letter hash + period return + AUM
  - HITL pause at step 2 on QA-ILPA-001 (ILPA template line-items missing) or QA-MARKETING-001 (Marketing Rule violation)
---

# Quarterly LP letter — `chief-investment-officer/quarterly-lp-letter`

CIO-Investment's quarterly LP letter per ILPA Reporting Template + SEC IA Marketing Rule + Mark Yusko / Jim Simons letter conventions.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../../skill/lp-letter-{author,audit}/SKILL.md`
