---
workflow_id: chief-esg-officer/annual-esg-report
workflow_version: 1.0.0
purpose: Author the annual ESG (Environmental + Social + Governance) report — strategy, performance against targets, stakeholder priorities, regulatory disclosures.
persona: cuo/chief-esg-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_report,          source: last year's sustainability-report@1, format: sustainability-report@1 }
  - { name: emissions_data,        source: cuo/chief-sustainability-officer/annual-emissions-inventory, format: emissions-inventory@1 }
  - { name: governance_state,      source: cuo/cco-compliance + Board governance, format: markdown }
  - { name: stakeholder_priorities, source: stakeholder engagement (investors, customers, employees, communities), format: markdown }

outputs:
  - { name: esg_report,            format: sustainability-report@1, recipient: cuo/chief-esg-officer + cuo/ceo + Board + investors + regulators (CSRD/ESRS jurisdictions) }

skill_chain:
  - { step: 1, skill: sustainability-report-author, inputs_from: { prior_report: prior_report, emissions_data: emissions_data, governance_state: governance_state, stakeholder_priorities: stakeholder_priorities }, outputs_to: report_draft }
  - { step: 2, skill: sustainability-report-audit,  inputs_from: report_draft, outputs_to: esg_report }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "report includes material restatement OR target revision" }
  - { persona: cuo/chief-legal-officer,      when: "CSRD/ESRS or SEC climate-disclosure obligations" }

consults:
  - { persona: cuo/chief-communications-officer, when: "external launch coordination" }
  - { persona: cuo/chief-financial-officer,            when: "ESG-linked financial disclosures" }

audit_hooks:
  - workflow_complete row on PASS with esg_report hash + CSRD/ESRS coverage
  - HITL pause at step 2 on QA-MATERIALITY-001 or QA-ASSURANCE-001
---

# Annual ESG report — `chief-esg-officer/annual-esg-report`

Chief ESG Officer's annual report per CSRD/ESRS + ISSB IFRS S1/S2 + GRI + SASB + TCFD frameworks. Critical for EU-market companies (CSRD mandatory) + companies seeking ESG investor capital.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../chief-sustainability-officer/workflows/annual-emissions-inventory.md` — upstream emissions feeder
- `../../../skill/sustainability-report-{author,audit}/SKILL.md`
