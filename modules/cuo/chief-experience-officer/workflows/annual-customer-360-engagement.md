---
workflow_id: chief-experience-officer/annual-customer-360-engagement
workflow_version: 1.0.0
purpose: Engage with CDO-Data on customer-360 architecture from a CX lens — identity-resolution gaps that block journey coherence, activation surfaces, consent.
persona: cuo/chief-experience-officer
cadence: annual
status: shipped

inputs:
  - { name: cdo_customer_360,      source: cuo/chief-data-officer/annual-customer-360-architecture, format: customer-360@1 }
  - { name: cx_priorities,         source: cuo/chief-experience-officer/annual-cx-strategy, format: strategy-document@1 }

outputs:
  - { name: cx_360_engagement,     format: customer-360@1 (CX-lens chapter), recipient: cuo/cxo + cuo/cdo-data + cuo/cpo-product }

skill_chain:
  - { step: 1, skill: customer-360-author, inputs_from: { cdo_customer_360: cdo_customer_360, cx_priorities: cx_priorities }, outputs_to: engagement_draft }
  - { step: 2, skill: customer-360-audit,  inputs_from: engagement_draft, outputs_to: cx_360_engagement }

audit_hooks:
  - workflow_complete row on PASS with cx_360_engagement hash
  - HITL pause at step 2 on QA-MATCH-001
---

# Annual customer-360 engagement (CX lens) — `chief-experience-officer/annual-customer-360-engagement`

CXO's CX-lens augmentation of CDO-Data's customer-360 architecture. Sister workflow pattern (same as risk-lens-vs-engineering-lens, content-vs-distribution).

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4
- `../../chief-data-officer/workflows/annual-customer-360-architecture.md` — upstream peer
- `../../../skill/customer-360-{author,audit}/SKILL.md`
