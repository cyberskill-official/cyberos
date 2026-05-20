---
workflow_id: chief-information-officer/quarterly-it-vendor-scorecard
workflow_version: 1.0.0
purpose: Score IT vendors — SLA attainment, security posture, sustainability, contract renewal triage.
persona: cuo/chief-information-officer
cadence: quarterly
status: shipped

inputs:
  - { name: vendor_register,       source: AP + IT-vendor register, format: csv }
  - { name: sla_attainment,        source: per-vendor SLA reports, format: csv }
  - { name: security_signals,      source: cuo/ciso vendor risk reports, format: markdown }

outputs:
  - { name: it_vendor_scorecard,   format: vendor-scorecard@1, recipient: cuo/cio-information + cuo/cao-admin + cuo/ciso }

skill_chain:
  - { step: 1, skill: vendor-scorecard-author, inputs_from: { vendor_register: vendor_register, sla_attainment: sla_attainment, security_signals: security_signals }, outputs_to: scorecard_draft }
  - { step: 2, skill: vendor-scorecard-audit,  inputs_from: scorecard_draft, outputs_to: it_vendor_scorecard }

audit_hooks:
  - workflow_complete row on PASS with it_vendor_scorecard hash
  - HITL pause at step 2 on QA-SLA-001
---

# Quarterly IT vendor scorecard — `chief-information-officer/quarterly-it-vendor-scorecard`

CIO-Information's quarterly IT vendor scoring per Gartner Magic Quadrant + ITIL supplier-management.

## Cross-references
- `../../../../modules/cuo/README.md` §5.3
- `../../chief-operating-officer/workflows/quarterly-vendor-scorecard.md` — broader peer
- `../../../skill/vendor-scorecard-{author,audit}/SKILL.md`
