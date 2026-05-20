---
workflow_id: chief-data-officer/annual-customer-360-architecture
workflow_version: 1.0.0
purpose: Audit the customer-360 / CDP architecture and propose the next-year refinement — identity resolution, master entity, activation surfaces.
persona: cuo/chief-data-officer
cadence: annual
status: shipped
pattern: persona_pair
peer_persona: chief-customer-officer
peer_workflow: quarterly-customer-health-review
shared_artefact: customer-profile
handoff_step: 4

inputs:
  - { name: prior_audit,           source: last year's customer-360@1, format: customer-360@1 }
  - { name: data_sources,          source: integration registry,        format: csv }
  - { name: consent_state,         source: cuo/cpo-privacy (consent register), format: markdown }
  - { name: activation_surfaces,   source: marketing / sales / in-product owners, format: markdown brief }

outputs:
  - { name: customer_360,          format: customer-360@1, recipient: cuo/cdo-data + cuo/cmo + cuo/cco-customer + cuo/cpo-privacy }

skill_chain:
  - { step: 1, skill: customer-360-author, inputs_from: { prior_audit: prior_audit, data_sources: data_sources, consent_state: consent_state, activation_surfaces: activation_surfaces }, outputs_to: audit_draft }
  - { step: 2, skill: customer-360-audit,  inputs_from: audit_draft, outputs_to: customer_360 }

escalates_to:
  - { persona: cuo/chief-privacy-officer,    when: "consent posture has cross-jurisdiction gap" }
  - { persona: cuo/chief-marketing-officer,            when: "activation surfaces miss critical marketing channel" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "CDP architecture decisions affect platform investment" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with customer_360 hash + match-rate + entity-count + activation-surface count
  - HITL pause at step 2 on QA-MATCH-001 (match-rate below benchmark) or QA-CONSENT-001 (consent gap)
---

# Annual customer-360 architecture — `chief-data-officer/annual-customer-360-architecture`

CDO-Data's annual customer-360 / CDP architecture audit. Per CDP Institute reference architecture + Segment / RudderStack / mParticle patterns + DAMA-DMBOK MDM. Drives identity resolution / master entity / activation roadmap for the year.

## When to invoke

- "Audit the customer-360 / CDP"
- "Annual CDP architecture review"
- "Customer-360 health check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-data-officer/annual-customer-360-architecture \
  --input prior_audit=./data/2025/customer-360.md \
  --input data_sources=./data/2026/integrations.csv \
  --input consent_state=./privacy/2026/consent.md \
  --input activation_surfaces=./data/2026/activation.md \
  --output-dir ./data/2026/customer-360/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for cross-function inputs
- **Worst case:** consent gap triggers regulator review

## Skill chain

- **Step 1 `customer-360-author`** — drafts per CDP Institute + DAMA-DMBOK MDM.
- **Step 2 `customer-360-audit`** — validates per `customer_360_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-MATCH-001 | Match-rate below benchmark | Operator extends entity resolution |
| 2 | QA-CONSENT-001 | Consent gap | Escalate to CPO-Privacy |

## Cross-references
- `../../../../modules/cuo/README.md` §5.3 — CDO-Data role profile
- `../../cpo-privacy/README.md` — consent peer
- `../../../skill/customer-360-{author,audit}/SKILL.md`
