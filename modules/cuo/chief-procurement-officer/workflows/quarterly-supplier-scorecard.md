---
workflow_id: chief-procurement-officer/quarterly-supplier-scorecard
workflow_version: 1.0.0
purpose: Score strategic suppliers — performance, cost, risk, sustainability, innovation contribution.
persona: cuo/chief-procurement-officer
cadence: quarterly
status: shipped

inputs:
  - { name: supplier_register,     source: ERP / procurement system, format: csv }
  - { name: sla_attainment,        source: per-supplier SLA reports, format: csv }
  - { name: spend_data,            source: AP, format: csv }
  - { name: risk_signals,          source: cuo/cro-risk supplier-risk signals, format: markdown }

outputs:
  - { name: supplier_scorecard,    format: vendor-scorecard@1, recipient: cuo/cpo-procurement + cuo/cfo + cuo/coo + cuo/cro-risk }

skill_chain:
  - { step: 1, skill: vendor-scorecard-author, inputs_from: { supplier_register: supplier_register, sla_attainment: sla_attainment, spend_data: spend_data, risk_signals: risk_signals }, outputs_to: scorecard_draft }
  - { step: 2, skill: vendor-scorecard-audit,  inputs_from: scorecard_draft, outputs_to: supplier_scorecard }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "scorecard recommends off-board > $250K/yr supplier" }
  - { persona: cuo/chief-legal-officer,      when: "off-board has cancellation-penalty exposure" }

audit_hooks:
  - workflow_complete row on PASS with supplier_scorecard hash + off-board count + at-risk count
  - HITL pause at step 2 on QA-RISK-001
---

# Quarterly supplier scorecard — `chief-procurement-officer/quarterly-supplier-scorecard`

CPO-Procurement's quarterly supplier scoring per CIPS supplier-performance-management + Kraljic-matrix + Sustainable Procurement Pledge.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../chief-operating-officer/workflows/quarterly-vendor-scorecard.md` — vendor-broader peer
- `../../../skill/vendor-scorecard-{author,audit}/SKILL.md`
