---
workflow_id: chief-esg-officer/quarterly-esg-compliance
workflow_version: 1.0.0
purpose: Review ESG compliance posture — CSRD/ESRS data collection, SEC climate disclosure readiness, voluntary reporting frameworks.
persona: cuo/chief-esg-officer
cadence: quarterly
status: shipped

inputs:
  - { name: compliance_program,    source: cuo/chief-compliance-officer/annual-compliance-program (ESG chapter), format: compliance-program@1 }
  - { name: prior_review,          source: last quarter's review, format: compliance-program@1 (ESG chapter) }
  - { name: regulator_signals,     source: SEC / EFRAG / IFRS activity, format: markdown }

outputs:
  - { name: esg_compliance,        format: compliance-program@1 (ESG quarterly chapter), recipient: cuo/chief-esg-officer + cuo/cco-compliance + cuo/clo-legal + Board (ESG-compliance update) }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { compliance_program: compliance_program, prior_review: prior_review, regulator_signals: regulator_signals }, outputs_to: review_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: review_draft, outputs_to: esg_compliance }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "regulator triggers proxy / 10-K disclosure obligation" }

consults:
  - { persona: cuo/chief-compliance-officer, when: "control intersection" }

audit_hooks:
  - workflow_complete row on PASS with esg_compliance hash
  - HITL pause at step 2 on QA-REGULATOR-001
---

# Quarterly ESG compliance — `chief-esg-officer/quarterly-esg-compliance`

Chief ESG Officer's quarterly compliance posture per CSRD + SEC climate rule + ISSB + voluntary frameworks (GRI / SASB / TCFD).

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `../../chief-compliance-officer/workflows/annual-compliance-program.md` — upstream feeder
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
