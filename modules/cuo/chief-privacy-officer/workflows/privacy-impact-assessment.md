---
workflow_id: chief-privacy-officer/privacy-impact-assessment
workflow_version: 1.0.0
purpose: Conduct a privacy impact assessment for a new product feature or system change touching personal data.
persona: cuo/chief-privacy-officer
cadence: per-event
status: shipped

inputs:
  - { name: feature_brief,      source: cuo/cpo-product (PRD or feature spec), format: markdown }
  - { name: data_flow_diagram,  source: cuo/cto / engineering team,            format: markdown / diagram }
  - { name: prior_pia,          source: similar features' prior privacy-impact-assessment@1 (if any), format: privacy-impact-assessment@1 }

outputs:
  - { name: pia,                format: pia@1, recipient: cuo/cpo-privacy + cuo/clo-legal + cuo/cto + cuo/cpo-product }

skill_chain:
  - { step: 1, skill: privacy-impact-assessment-author, inputs_from: { feature_brief: feature_brief, data_flow_diagram: data_flow_diagram, prior_pia: prior_pia }, outputs_to: pia_draft }
  - { step: 2, skill: privacy-impact-assessment-audit,  inputs_from: pia_draft, outputs_to: pia }

escalates_to:
  - { persona: cuo/chief-legal-officer,   when: "PIA surfaces high-risk processing requiring DPIA (GDPR Art. 35) or regulator consultation (Art. 36)" }
  - { persona: cuo/chief-technology-officer,         when: "PIA surfaces technical control gap (encryption / pseudonymization / access)" }

consults:
  - { persona: cuo/chief-information-security-officer,        when: "PIA touches security boundaries (encryption / KMS / access controls)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with pia hash + risk-rating + DPIA-required flag
  - HITL pause at step 2 on QA-HIGH-RISK-001 (high-risk processing without DPIA escalation)
---

# Privacy impact assessment — `chief-privacy-officer/privacy-impact-assessment`

CPO-Privacy's per-feature PIA workflow. Triggered before any product feature or system change that touches personal data goes live. Per GDPR Art. 35 (DPIA) + UK ICO PIA framework + NIST Privacy Framework. PIA upgrades to DPIA when processing meets Art. 35(3) high-risk criteria.

## When to invoke

- "PIA for [feature]"
- "Privacy review for [system change]"
- "DPIA required for [proposal]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-privacy-officer/privacy-impact-assessment \
  --input feature_brief=./product/prd/2026-acme-feature.md \
  --input data_flow_diagram=./engineering/dfd/2026-acme.md \
  --input prior_pia=./privacy/pia/2025-similar-feature.md \
  --output-dir ./privacy/pia/2026-acme/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 1-2 weeks for cross-function input
- **Worst case:** DPIA escalation + regulator consultation may add 1-2 months

## Skill chain

- **Step 1 `privacy-impact-assessment-author`** — drafts per GDPR Art. 35 + ICO PIA framework + NIST Privacy Framework.
- **Step 2 `privacy-impact-assessment-audit`** — validates per `pia_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-HIGH-RISK-001 | High-risk no DPIA escalation | Escalate to CLO-Legal |
| 2 | QA-MITIGATION-001 | Identified risk no mitigation | Operator drafts |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CPO-Privacy role profile
- `../../chief-legal-officer/workflows/quarterly-regulatory-cycle.md` — peer for DPIA filing
- `../../../skill/pia-{author,audit}/SKILL.md`
