---
workflow_id: chief-privacy-officer/data-subject-request-cycle
workflow_version: 1.0.0
purpose: Handle a single data-subject request (DSR) — verify identity, scope, fulfill within statutory window (GDPR 30 days / PDPD 30 days / CCPA 45 days).
persona: cuo/chief-privacy-officer
cadence: on-demand
status: shipped

inputs:
  - { name: dsr_intake,         source: privacy intake portal or email,         format: markdown brief }
  - { name: data_inventory,     source: ROPA / data-inventory register,         format: csv export }
  - { name: identity_evidence,  source: requester (verification documents),     format: markdown }

outputs:
  - { name: dsr_response,       format: dsr-runbook@1, recipient: data subject + cuo/cpo-privacy (audit log) }

skill_chain:
  - { step: 1, skill: dsr-runbook-author, inputs_from: { dsr_intake: dsr_intake, data_inventory: data_inventory, identity_evidence: identity_evidence }, outputs_to: response_draft }
  - { step: 2, skill: dsr-runbook-audit,  inputs_from: response_draft, outputs_to: dsr_response }

escalates_to:
  - { persona: cuo/chief-legal-officer,   when: "request is unusual: deletion of customer of record / class request / litigation-related" }
  - { persona: cuo/chief-information-security-officer,        when: "fulfillment requires access to system not in standard DSR runbook" }

consults:
  - { persona: cuo/chief-accounting-officer, when: "request touches financial records (retention obligation conflicts)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with dsr_response hash + jurisdiction + statutory-window-days + fulfillment-days
  - HITL pause at step 2 on QA-WINDOW-001 (response past statutory window) or QA-IDENTITY-001 (identity verification weak)
---

# Data subject request cycle — `chief-privacy-officer/data-subject-request-cycle`

CPO-Privacy's standard per-request DSR workflow. Per GDPR Art. 12-22 / Vietnam Decree 13/2023 PDPD / CCPA-CPRA / LGPD. Statutory windows: GDPR 30 days (extendable to 90 with notice), PDPD 30 days, CCPA 45 days. Critical: NEVER respond past window without documented extension reason — that's itself a violation.

## When to invoke

- "Handle DSR from [requester]"
- "Process this access request"
- "Run the DSR runbook for [request type]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-privacy-officer/data-subject-request-cycle \
  --input dsr_intake=./privacy/dsr/2026-05-acme/intake.md \
  --input data_inventory=./privacy/ropa.csv \
  --input identity_evidence=./privacy/dsr/2026-05-acme/identity.md \
  --output-dir ./privacy/dsr/2026-05-acme/
```

## Expected duration

- **Happy path:** 1-3 hours runtime + 5-15 business days for fulfillment
- **Worst case:** statutory-window extension requires legal review + documented justification

## Skill chain

- **Step 1 `dsr-runbook-author`** — drafts per GDPR Art. 12-22 / PDPD / CCPA structure.
- **Step 2 `dsr-runbook-audit`** — validates per `dsr_runbook_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-WINDOW-001 | Past statutory window | Escalate to CLO-Legal |
| 2 | QA-IDENTITY-001 | Identity verification weak | Operator requests stronger evidence |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CPO-Privacy role profile
- `../../chief-legal-officer/workflows/quarterly-regulatory-cycle.md` — peer for aggregate reporting
- `../../../skill/dsr-runbook-{author,audit}/SKILL.md`
