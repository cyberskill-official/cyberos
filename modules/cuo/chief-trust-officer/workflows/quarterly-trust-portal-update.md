---
workflow_id: chief-trust-officer/quarterly-trust-portal-update
workflow_version: 1.0.0
purpose: Refresh the public trust portal — certifications, security posture, privacy controls, subprocessor list, audit evidence.
persona: cuo/chief-trust-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_portal,          source: last quarter's trust-portal-update@1, format: trust-portal-update@1 }
  - { name: cert_status,           source: cuo/ciso (SOC 2 / ISO 27001 / HIPAA / FedRAMP attestations), format: markdown }
  - { name: privacy_state,         source: cuo/chief-privacy-officer/annual-privacy-program, format: compliance-program@1 }
  - { name: subprocessor_list,     source: vendor + sub-processor register, format: csv }

outputs:
  - { name: trust_portal,          format: trust-portal-update@1, recipient: cuo/chief-trust-officer + public trust portal (customers + prospects) }

skill_chain:
  - { step: 1, skill: trust-portal-update-author, inputs_from: { prior_portal: prior_portal, cert_status: cert_status, privacy_state: privacy_state, subprocessor_list: subprocessor_list }, outputs_to: portal_draft }
  - { step: 2, skill: trust-portal-update-audit,  inputs_from: portal_draft, outputs_to: trust_portal }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "trust portal updates trigger customer contractual notification (DPA, BAA, MSA)" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "security-posture changes need CISO sign-off" }
  - { persona: cuo/chief-privacy-officer,    when: "privacy-control changes need CPO-Privacy sign-off" }
  - { persona: cuo/chief-communications-officer, when: "public-portal changes need external positioning" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with trust_portal hash + cert count + subprocessor count
  - HITL pause at step 2 on QA-CERT-EXPIRY-001 (cert approaching expiry) or QA-SUBPROCESSOR-001 (subprocessor change without customer notice)
---

# Quarterly trust portal update — `chief-trust-officer/quarterly-trust-portal-update`

Chief Trust Officer's quarterly public-trust-portal refresh per Vanta / Drata / Tugboat trust-portal patterns + ACSC trust-relations framework. Combines cert status + privacy state + subprocessor list into the customer-facing trust portal.

## When to invoke

- "Run the Q<n> trust portal refresh"
- "Update the trust center"
- "Quarterly trust posture update"

## How to invoke

```bash
cyberos-cuo run cuo/chief-trust-officer/quarterly-trust-portal-update \
  --input prior_portal=./trust/2026-Q1/portal.md \
  --input cert_status=./security/2026-Q1/certs.md \
  --input privacy_state=./privacy/2026-Q1/program.md \
  --input subprocessor_list=./vendors/subprocessors.csv \
  --output-dir ./trust/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for cross-function sign-off
- **Worst case:** subprocessor change triggers DPA-amendment cycle (1 quarter)

## Skill chain

- **Step 1 `trust-portal-update-author`** — drafts per Vanta + Drata + ACSC standards.
- **Step 2 `trust-portal-update-audit`** — validates per `trust_portal_update_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CERT-EXPIRY-001 | Cert approaching expiry | Escalate to CISO |
| 2 | QA-SUBPROCESSOR-001 | Change without notice | Escalate to CLO-Legal |

## Cross-references
- `../../../../modules/cuo/README.md` §5.6 — Chief Trust Officer role profile
- `../../chief-information-security-officer/workflows/soc2-audit-readiness.md` — cert peer
- `../../chief-privacy-officer/workflows/annual-privacy-program.md` — privacy peer
- `../../../skill/trust-portal-update-{author,audit}/SKILL.md`
