---
template: runbook@1
title: <service> — operational runbook
service_or_system: <service name>
runbook_version: 1.0.0
owner_team: <team name>
oncall_rotation_url: https://pagerduty.com/schedules/<id>
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
last_drilled_at: 2026-MM-DD
next_review_date: 2026-MM-DD    # ≤90 days from last_drilled_at
---

# <service> — operational runbook

## 1. Service Overview
What it does; who depends on it; criticality tier.

## 2. SLOs and SLAs
| SLI | target | measurement | budget |
|---|---|---|---|

## 3. Error-Budget Policy
What happens when budget is exhausted.

## 4. On-Call Rota
See [`oncall_rotation_url`]. Escalation tree: <primary> → <secondary> → <manager>.

## 5. Architecture Quick-Ref
Key components, data flows, external dependencies.

## 6. Common Alerts

### 6.1 <alert_name>
- **Symptom:** <what the alert says>
- **Dashboard:** <link>
- **Runbook steps:** 1. <step> 2. <step>
- **Escalation:** when to page <secondary>

## 7. Common Operations
| operation | command / link | rollback |
|---|---|---|
| restart   | ... | n/a |
| scale up  | ... | scale down |
| drain     | ... | undrain |
| deploy    | ... | rollback (see DEP-003 in deploy-checklist) |
| key rotate| ... | use prior key cert; rotate again |

## 8. Observability
- Grafana: <links>
- Datadog: <links>
- Sentry: <links>
- OTel: <trace examples>

## 9. Disaster Recovery
- Backup location: ...
- RTO: ...
- RPO: ...
- Last drill: <date + outcome>

## 10. Vendor and Dependency Contacts
| vendor | support tier | escalation channel | contract ref |
|---|---|---|---|

<!-- ## 11. Data Breach Response       — when personal data -->
<!-- ## 12. Region Failover Procedures — when multi-region -->
<!-- ## 13. Rate-Limit + Abuse Response — when public API -->
<!-- ## 14. PCI-DSS Incident Procedures — when payment-related -->
