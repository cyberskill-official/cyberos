---
workflow_id: chief-architect/per-threat-model-review
workflow_version: 1.0.0
purpose: Conduct per-system threat-model review — STRIDE analysis, ASVS verification level, mitigations.
persona: cuo/chief-architect
cadence: per-event
status: shipped

inputs:
  - { name: system_design,         source: SDD or architecture brief, format: markdown }
  - { name: prior_threat_model,    source: prior threat-model@1 if exists, format: threat-model@1 }
  - { name: adr_set,               source: relevant ADRs, format: architecture-decision-record@1 (set) }

outputs:
  - { name: threat_model,          format: threat-model@1, recipient: cuo/chief-architect + cuo/ciso + cuo/cto + engineering team }

skill_chain:
  - { step: 1, skill: threat-model-author, inputs_from: { system_design: system_design, prior_threat_model: prior_threat_model, adr_set: adr_set }, outputs_to: tm_draft }
  - { step: 2, skill: threat-model-audit,  inputs_from: tm_draft, outputs_to: threat_model }

escalates_to:
  - { persona: cuo/chief-information-security-officer,           when: "STRIDE-S/E/T findings cross security trust boundary" }

audit_hooks:
  - workflow_complete row on PASS with threat_model hash
  - HITL pause at step 2 on QA-ASVS-001
---

# Per threat model review — `chief-architect/per-threat-model-review`

Chief-Architect's per-system threat-model review per STRIDE + OWASP Top 10:2025 + ASVS v5.0 + MITRE ATT&CK.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3
- `../../chief-technology-officer/workflows/threat-model-refresh.md` — CTO quarterly peer
- `../../../skill/threat-model-{author,audit}/SKILL.md`
