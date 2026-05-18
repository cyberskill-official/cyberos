---
workflow_id: chief-security-officer/quarterly-vuln-management
workflow_version: 1.0.0
purpose: Quarterly converged vulnerability management — physical-facility vulns + info-sec vulns + supply-chain vulns + insider-threat indicators.
persona: cuo/chief-security-officer
cadence: quarterly
status: shipped

inputs:
  - { name: ciso_vuln_report,      source: cuo/chief-information-security-officer/monthly-vuln-management (quarterly roll-up), format: vulnerability-management-report@1 (3 months) }
  - { name: physical_audit,        source: facility audits + access-log review, format: csv }
  - { name: supply_chain_intel,    source: supply-chain vendor security signals, format: markdown }

outputs:
  - { name: converged_vuln_report, format: vulnerability-management-report@1 (converged), recipient: cuo/cso-security + cuo/ciso + cuo/cao-admin + Board (security chapter) }

skill_chain:
  - { step: 1, skill: vulnerability-management-report-author, inputs_from: { ciso_vuln_report: ciso_vuln_report, physical_audit: physical_audit, supply_chain_intel: supply_chain_intel }, outputs_to: report_draft }
  - { step: 2, skill: vulnerability-management-report-audit,  inputs_from: report_draft, outputs_to: converged_vuln_report }

audit_hooks:
  - workflow_complete row on PASS with converged_vuln_report hash
  - HITL pause at step 2 on QA-SLA-001
---

# Quarterly converged vulnerability management — `chief-security-officer/quarterly-vuln-management`

CSO-Security's quarterly converged VM report per ASIS ESRM + CIS Controls v8 (physical extension) + NIST SP 800-82 (ICS/OT security).

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `../../chief-information-security-officer/workflows/monthly-vuln-management.md` — info-sec peer (monthly cadence)
- `../../../skill/vulnerability-mgmt-report-{author,audit}/SKILL.md`
