---
workflow_id: chief-information-officer/annual-it-security-strategy
workflow_version: 1.0.0
purpose: Author IT-side input to the annual security strategy — endpoint security, identity-access, network-security, BCP/DR.
persona: cuo/chief-information-officer
cadence: annual
status: shipped

inputs:
  - { name: ciso_strategy,         source: cuo/chief-information-security-officer/annual-security-strategy, format: security-strategy@1 }
  - { name: it_strategy,           source: cuo/chief-information-officer/annual-it-strategy, format: strategy-document@1 }
  - { name: incident_lookback,     source: prior-year IT-security incidents, format: markdown }

outputs:
  - { name: it_security_input,     format: security-strategy@1 (IT chapter), recipient: cuo/cio-information + cuo/ciso + cuo/ceo }

skill_chain:
  - { step: 1, skill: security-strategy-author, inputs_from: { ciso_strategy: ciso_strategy, it_strategy: it_strategy, incident_lookback: incident_lookback }, outputs_to: input_draft }
  - { step: 2, skill: security-strategy-audit,  inputs_from: input_draft, outputs_to: it_security_input }

audit_hooks:
  - workflow_complete row on PASS with it_security_input hash
  - HITL pause at step 2 on QA-POSTURE-001
---

# Annual IT security strategy — `chief-information-officer/annual-it-security-strategy`

CIO-Information's IT-lens input to security strategy per NIST CSF 2.0 + CIS Controls v8 IG2 (IT operations focus).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3
- `../../chief-information-security-officer/workflows/annual-security-strategy.md` — upstream peer
- `../../../skill/security-strategy-{author,audit}/SKILL.md`
