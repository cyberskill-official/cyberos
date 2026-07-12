---
workflow_id: chief-medical-officer/per-clinical-protocol
workflow_version: 1.0.0
purpose: Author a clinical trial protocol — study design, endpoints, eligibility, statistical plan, safety monitoring.
persona: cuo/chief-medical-officer
cadence: per-event
status: shipped

inputs:
  - { name: study_brief,           source: clinical research team, format: markdown }
  - { name: ind_application,       source: regulatory team (if IND-stage), format: markdown }
  - { name: prior_protocols,       source: similar prior clinical-protocol@1, format: clinical-protocol@1 (set) }

outputs:
  - { name: clinical_protocol,     format: clinical-protocol@1, recipient: cuo/chief-medical-officer + IRB + FDA/EMA + clinical sites + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: clinical-protocol-author, inputs_from: { study_brief: study_brief, ind_application: ind_application, prior_protocols: prior_protocols }, outputs_to: protocol_draft }
  - { step: 2, skill: clinical-protocol-audit,  inputs_from: protocol_draft, outputs_to: clinical_protocol }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "protocol triggers IND amendment OR multi-country regulatory coordination" }
  - { persona: cuo/chief-privacy-officer,    when: "protocol includes personal-health-data processing (HIPAA / GDPR Article 9 special category)" }

audit_hooks:
  - workflow_complete row on PASS with clinical_protocol hash + protocol version
  - HITL pause at step 2 on QA-ICH-GCP-001 (E6(R3) section ordering deviation) or QA-ENDPOINT-001
---

# Per clinical protocol — `chief-medical-officer/per-clinical-protocol`

Chief Medical Officer's per-study clinical-protocol workflow per ICH-GCP E6(R3) + ICH E8(R1) + CONSORT 2010 + SPIRIT 2013. Mandatory IRB + regulator review pre-execution.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Medical Officer role profile
- `../../../skill/clinical-protocol-{author,audit}/SKILL.md`
