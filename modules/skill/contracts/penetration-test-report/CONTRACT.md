# penetration-test-report@1 — Penetration test report

A `penetration-test-report@1` artefact authors a penetration-test engagement report (internal red-team OR external third-party). Executive summary, scope, methodology, findings ranked by severity (CVSS), proof-of-concept, remediation guidance, retest plan. Per OWASP Web Security Testing Guide (WSTG) + PTES (Penetration Testing Execution Standard) + NIST SP 800-115.

## Required sections (template.md H2 order)
1. Executive summary
2. Engagement scope & rules of engagement
3. Methodology
4. Findings (per-finding: title / CVSS / impact / PoC / remediation)
5. Remediation roadmap
6. Retest plan
7. Appendices (full tool output, attestations)

## Citations
- OWASP Web Security Testing Guide (WSTG) v4.2
- PTES (Penetration Testing Execution Standard)
- NIST SP 800-115 (Technical Guide to Information Security Testing)
- OWASP ASVS v5.0
- MITRE ATT&CK framework

## KPI
- Number of findings by severity
- Mean CVSS score
- Remediation acceptance rate
