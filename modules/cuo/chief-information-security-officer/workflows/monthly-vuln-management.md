---
workflow_id: chief-information-security-officer/monthly-vuln-management
workflow_version: 1.0.0
purpose: Author the monthly vulnerability-management report — open vulns by CVSS severity, patch SLA compliance, exception backlog, trend, remediation roadmap.
persona: cuo/chief-information-security-officer
cadence: monthly
status: shipped

inputs:
  - { name: scanner_export,     source: Tenable / Qualys / Wiz / Snyk / Dependabot, format: csv / json }
  - { name: ticket_export,      source: Jira / Linear (security tickets),            format: csv }
  - { name: prior_report,       source: last month's vulnerability-mgmt-report@1,    format: vulnerability-mgmt-report@1 }
  - { name: exception_register, source: ciso exception-tracking register,             format: markdown }

outputs:
  - { name: vuln_report,        format: vulnerability-mgmt-report@1, recipient: cuo/ciso + cuo/cto + cuo/cco-compliance }

skill_chain:
  - { step: 1, skill: vulnerability-mgmt-report-author, inputs_from: { scanner_export: scanner_export, ticket_export: ticket_export, prior_report: prior_report, exception_register: exception_register }, outputs_to: report_draft }
  - { step: 2, skill: vulnerability-mgmt-report-audit,  inputs_from: report_draft, outputs_to: vuln_report }

escalates_to:
  - { persona: cuo/chief-technology-officer,         when: "Critical CVSS unremediated past SLA (7 days for internet-facing, 30 days for internal)" }
  - { persona: cuo/chief-legal-officer,   when: "exception accepts risk on customer-data system" }

consults:
  - { persona: cuo/chief-operating-officer,         when: "vuln remediation requires production deployment slot in delivery schedule" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with vuln_report hash + open-vuln-count + SLA-compliance%
  - HITL pause at step 2 on QA-SLA-001 (Critical past SLA without escalation flag)
---

# Monthly vulnerability management — `chief-information-security-officer/monthly-vuln-management`

CISO's monthly vulnerability-management workflow. Combines scanner exports + ticket data + prior report + exception register into the standard MTTR + SLA-compliance + remediation-roadmap report per CIS Controls v8 Control 7. SLA targets: Critical CVSS 7 days (internet-facing) / 30 days (internal); High 14/60; Medium 30/90.

## When to invoke

- "Run the monthly vuln report"
- "VM status for May"
- "Vulnerability management update"

## How to invoke

```bash
cyberos-cuo run cuo/chief-information-security-officer/monthly-vuln-management \
  --input scanner_export=./security/2026-05/tenable.csv \
  --input ticket_export=./security/2026-05/jira.csv \
  --input prior_report=./security/2026-04/vuln-report.md \
  --input exception_register=./security/exceptions.md \
  --output-dir ./security/2026-05/
```

## Expected duration

- **Happy path:** 30-60 min runtime + 1-2 days for owner review
- **Worst case:** Critical past-SLA triggers same-day CTO escalation + emergency patch window

## Skill chain

- **Step 1 `vulnerability-mgmt-report-author`** — drafts per CIS Controls v8 + NIST SP 800-40 Rev 4 + CVSS v3.1/v4.0 structure.
- **Step 2 `vulnerability-mgmt-report-audit`** — validates per `vulnerability_mgmt_report_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SLA-001 | Critical past SLA unflagged | Escalate to CTO |
| 2 | QA-EXCEPTION-001 | Exception lacks expiry / re-review date | Operator adds |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CISO role profile
- `../../chief-technology-officer/workflows/deploy-readiness-review.md` — patch-window peer
- `../../../skill/vulnerability-mgmt-report-{author,audit}/SKILL.md`
