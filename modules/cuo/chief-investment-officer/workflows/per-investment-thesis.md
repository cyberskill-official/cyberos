---
workflow_id: chief-investment-officer/per-investment-thesis
workflow_version: 1.0.0
purpose: Author an investment thesis for a position — investment hypothesis, valuation, catalysts, risks, position sizing.
persona: cuo/chief-investment-officer
cadence: per-event
status: shipped

inputs:
  - { name: target_brief,          source: research analyst, format: markdown }
  - { name: investment_strategy,   source: fund mandate + IPS, format: markdown }
  - { name: comparable_theses,     source: prior investment-thesis@1, format: investment-thesis@1 (set) }

outputs:
  - { name: investment_thesis,     format: investment-thesis@1, recipient: cuo/cio-investment + investment committee + portfolio team }

skill_chain:
  - { step: 1, skill: investment-thesis-author, inputs_from: { target_brief: target_brief, investment_strategy: investment_strategy, comparable_theses: comparable_theses }, outputs_to: thesis_draft }
  - { step: 2, skill: investment-thesis-audit,  inputs_from: thesis_draft, outputs_to: investment_thesis }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "thesis size > 5% of fund OR sector concentration > IPS limit" }

audit_hooks:
  - workflow_complete row on PASS with investment_thesis hash + position size + IRR / multiple targets
  - HITL pause at step 2 on QA-CATALYST-001 (catalyst date vague) or QA-RISK-001
---

# Per investment thesis — `chief-investment-officer/per-investment-thesis`

CIO-Investment's per-position thesis workflow per Soros reflexivity + Druckenmiller catalyst-driven + Howard Marks risk-first + Damodaran valuation framework.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7 — CIO-Investment role profile
- `../../../skill/investment-thesis-{author,audit}/SKILL.md`
