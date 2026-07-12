---
workflow_id: chief-risk-officer/annual-erm-framework
workflow_version: 1.0.0
purpose: Refresh the annual Enterprise Risk Management framework — risk taxonomy, appetite statements, risk-and-control matrix, governance.
persona: cuo/chief-risk-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_framework,       source: last year's enterprise-risk-framework@1, format: enterprise-risk-framework@1 }
  - { name: incident_lookback,     source: 12 months postmortems + breach-notifications, format: markdown }
  - { name: regulator_signal,      source: cuo/clo-legal regulatory activity, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: erm_framework,         format: enterprise-risk-framework@1, recipient: cuo/cro-risk + cuo/ceo + cuo/clo-legal + Board (annual ERM chapter) }

skill_chain:
  - { step: 1, skill: enterprise-risk-framework-author, inputs_from: { prior_framework: prior_framework, incident_lookback: incident_lookback, regulator_signal: regulator_signal, ceo_priorities: ceo_priorities }, outputs_to: framework_draft }
  - { step: 2, skill: enterprise-risk-framework-audit,  inputs_from: framework_draft, outputs_to: erm_framework }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "framework proposes appetite change or new risk class" }
  - { persona: cuo/chief-legal-officer,      when: "regulator alignment requires legal review" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "cyber-risk class needs CISO sign-off" }
  - { persona: cuo/chief-privacy-officer,    when: "privacy-risk class needs CPO-Privacy sign-off" }
  - { persona: cuo/chief-compliance-officer, when: "compliance-risk class aligns with compliance-program" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with erm_framework hash + risk-class count + appetite statement count
  - HITL pause at step 2 on QA-APPETITE-001 (appetite ambiguous) or QA-COVERAGE-001 (incident class not represented)
---

# Annual ERM framework — `chief-risk-officer/annual-erm-framework`

CRO-Risk's annual ERM framework refresh. Per COSO ERM (2017) + ISO 31000:2018 + RIMS Risk Maturity Model. Defines risk taxonomy + appetite + risk-and-control matrix + governance for the year. Foundation document for all CRO-Risk activity.

## When to invoke

- "Refresh the 2026 ERM framework"
- "Annual ERM review"
- "Update risk taxonomy + appetite"

## How to invoke

```bash
cyberos-cuo run cuo/chief-risk-officer/annual-erm-framework \
  --input prior_framework=./risk/2025/erm.md \
  --input incident_lookback=./incidents/2025/ \
  --input regulator_signal=./legal/2026/regulatory-activity.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./risk/2026/erm/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** new risk class triggers control-design cycle (1-2 quarter)

## Skill chain

- **Step 1 `enterprise-risk-framework-author`** — drafts per COSO ERM + ISO 31000 + RIMS RMM.
- **Step 2 `enterprise-risk-framework-audit`** — validates per `enterprise_risk_framework_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-APPETITE-001 | Appetite ambiguous | Operator quantifies |
| 2 | QA-COVERAGE-001 | Incident class missing | Operator adds class |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — CRO-Risk role profile
- `../../cco-compliance/README.md` — compliance peer
- `../../../skill/enterprise-risk-framework-{author,audit}/SKILL.md`
