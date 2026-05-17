---
template: decomm@1
title: <system> — Decommissioning Package
system_being_retired: <system name>
retirement_date: 2026-MM-DD
decomm_version: 1.0.0
data_retention_policy_ref: <link to policy>
provenance: { source_path: ./runbook.md, source_hash: sha256:<hash> }
decision_owner: { handle: "@<em>",  role: "EM" }
compliance_owner: @<compliance-owner>
---

# <system> — Decommissioning Package

## 1. Retirement Decision and Rationale

## 2. Affected Stakeholders
Customers, internal teams, vendors, regulators.

## 3. Customer Communication Timeline
| milestone | date | channel | message |
|---|---|---|---|
| T-60 | ... | email + status page | ... |
| T-30 | ... | ... | ... |
| T-7  | ... | ... | ... |
| T-1  | ... | ... | ... |
| T-0  | ... | ... | sunset |

## 4. Data Retention Plan
| data class | retain? | where | for how long | who holds |
|---|---|---|---|---|

## 5. Data Export Plan
| target | format | channel | verification |
|---|---|---|---|

## 6. Data Destruction Certificate
| data class | destruction method | witness | completion log |
|---|---|---|---|
| ... | overwrite + verify / crypto-erase / physical | @<handle> | ./logs/destruction-<ts>.log |

## 7. DNS and Endpoint Retirement
| FQDN | sunset_status_code | redirect target | final removal date |
|---|---|---|---|

## 8. License and Vendor Cancellation
| third-party | cancellation date | residual obligations |
|---|---|---|

## 9. Source-Code Archive Manifest
| repo | branch | tag | archive_location | archive_sha256 |
|---|---|---|---|---|

## 10. Final Backup
Last full backup location + retention term.

## 11. On-Call and Runbook Decommissioning
- Runbook artefact archived: <path>
- On-call rota removed at: <PagerDuty/Opsgenie ref>

## 12. Sign-Off
| role | signer | signed_at |
|---|---|---|
| decision_owner | @<em> | <ts> |
| compliance_owner | @<co> | <ts> |
| ops_owner | @<op> | <ts> |

<!-- ## 13. GDPR Article 17 Compliance       — when system processed personal data -->
<!-- ## 14. Vietnam Decree 13/2023 PDPD      — when system processed personal data in VN -->
<!-- ## 15. PCI-DSS Decommissioning          — when financial/payment data -->
<!-- ## 16. HIPAA Disposal Compliance        — when health data -->
<!-- ## 17. Partner Notification Log         — when external-partner integrations -->
<!-- ## 18. Migration Path to Successor      — when being replaced -->
<!-- ## 19. Refund / Credit Policy           — when retired without replacement + paying customers -->
