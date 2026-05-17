---
workflow_id: chief-information-security-officer/annual-pen-test-cycle
workflow_version: 1.0.0
purpose: Run the annual pen-test cycle — scope, engagement, report ingestion, finding-triage, remediation tracking, retest.
persona: cuo/chief-information-security-officer
cadence: annual
status: shipped

inputs:
  - { name: scope_definition,    source: ciso + cto (in-scope systems + rules of engagement), format: markdown }
  - { name: vendor_engagement,   source: third-party pen-test vendor or internal red-team,    format: markdown brief }
  - { name: prior_reports,       source: last year's pen-test-report@1 set,                   format: pen-test-report@1 }
  - { name: threat_model,        source: cuo/chief-technology-officer/threat-model-refresh,                        format: threat-model@1 }

outputs:
  - { name: pen_test_report,     format: pen-test-report@1, recipient: cuo/ciso + cuo/cto + cuo/clo-legal (for material-finding disclosure) }

skill_chain:
  - { step: 1, skill: pen-test-report-author, inputs_from: { scope_definition: scope_definition, vendor_engagement: vendor_engagement, prior_reports: prior_reports, threat_model: threat_model }, outputs_to: report_draft }
  - { step: 2, skill: pen-test-report-audit,  inputs_from: report_draft, outputs_to: pen_test_report }

escalates_to:
  - { persona: cuo/chief-technology-officer,         when: "Critical CVSS finding requires architecture change" }
  - { persona: cuo/chief-legal-officer,   when: "finding triggers SEC 10-K Item 1C cybersecurity disclosure" }
  - { persona: cuo/chief-executive-officer,         when: "finding implies material-event 8-K filing" }

consults:
  - { persona: cuo/chief-compliance-officer, when: "finding affects SOC 2 / ISO 27001 / FedRAMP certification" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with pen_test_report hash + finding-count by severity + retest plan
  - HITL pause at step 2 on QA-POC-001 (PoC not reproducible) or QA-REMEDIATION-001 (remediation lacks specificity)
---

# Annual pen-test cycle — `chief-information-security-officer/annual-pen-test-cycle`

CISO's annual penetration-test workflow. Scope + ROE definition → vendor engagement → report ingestion + finding-triage → remediation roadmap → retest. Per OWASP WSTG v4.2 + PTES + NIST SP 800-115 + OWASP ASVS v5.0. Critical findings escalate same-day; Material findings escalate to CEO for 8-K assessment.

## When to invoke

- "Run the 2026 pen-test cycle"
- "Annual security testing"
- "Schedule the next pen test"

## How to invoke

```bash
cyberos-cuo run cuo/chief-information-security-officer/annual-pen-test-cycle \
  --input scope_definition=./security/2026/pentest-scope.md \
  --input vendor_engagement=./security/2026/pentest-vendor.md \
  --input prior_reports=./security/2025/pentest/ \
  --input threat_model=./security/2026-Q1/threat-model.md \
  --output-dir ./security/2026/pentest/
```

## Expected duration

- **Happy path:** 4-8 weeks engagement + 1 week report finalization + 1-3 months remediation + retest
- **Worst case:** Critical finding pre-disclosure triggers emergency-patch window + same-day CISO+CTO+CEO+CLO call

## Skill chain

- **Step 1 `pen-test-report-author`** — drafts per OWASP WSTG + PTES + NIST SP 800-115 + ASVS structure.
- **Step 2 `pen-test-report-audit`** — validates per `pen_test_report_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-POC-001 | PoC not reproducible | Vendor re-test |
| 2 | QA-REMEDIATION-001 | Remediation lacks specificity | Operator extends |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CISO role profile
- `../../chief-technology-officer/workflows/threat-model-refresh.md` — quarterly upstream peer
- `../../../skill/pen-test-report-{author,audit}/SKILL.md`
