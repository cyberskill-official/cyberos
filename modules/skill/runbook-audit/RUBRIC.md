# `runbook_rubric@1.0` — machine-checkable Runbook rubric

> Sourced from `../../../modules/cuo/docs/module.md` §2(j) Operations; Google SRE Book (SLOs, error budgets); OpenTelemetry observability conventions. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `runbook@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `service_or_system` | required, string | error | false |
| `FM-103` | `runbook_version` | required, SemVer | error | true |
| `FM-104` | `owner_team` | required, string | error | false |
| `FM-105` | `oncall_rotation_url` | required, URL (PagerDuty / Opsgenie / Better Stack / etc.) | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-107` | `last_drilled_at` | required, ISO 8601 (date of last live runbook drill) | error | false |
| `FM-108` | `next_review_date` | required, ISO 8601 (max +90 days from `last_drilled_at`) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Service Overview` (what it does, who depends on it, criticality tier) | error |
| `SEC-002` | `## 2. SLOs and SLAs` (per-indicator targets with measurement method) | error |
| `SEC-003` | `## 3. Error-Budget Policy` (what happens when budget is exhausted) | error |
| `SEC-004` | `## 4. On-Call Rota` (link + escalation tree) | error |
| `SEC-005` | `## 5. Architecture Quick-Ref` (key components, data flows, external dependencies) | error |
| `SEC-006` | `## 6. Common Alerts` (one H3 per alert — symptom, dashboard link, runbook steps, escalation path) | error |
| `SEC-007` | `## 7. Common Operations` (restart, scale, drain, deploy, rollback, cache flush, key rotation) | error |
| `SEC-008` | `## 8. Observability` (Grafana / Datadog / Sentry / OTel — dashboard links, log queries, trace examples) | error |
| `SEC-009` | `## 9. Disaster Recovery` (backup location, RTO, RPO, last drill date + outcome) | error |
| `SEC-010` | `## 10. Vendor and Dependency Contacts` (support tier, escalation channel, contract reference) | warning |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Service handles personal data | `## 11. Data Breach Response` (per GDPR Art. 33 / Vietnam Decree 13/2023 PDPD timelines) | error |
| `COND-002` | Service is multi-region | `## 12. Region Failover Procedures` | error |
| `COND-003` | Service exposes a public API | `## 13. Rate-Limit and Abuse Response` | error |
| `COND-004` | Service is payment-related | `## 14. PCI-DSS Incident Procedures` | error → needs_human (`legal_compliance`) |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-SLO-001` | SLO without measurement method | A SLO in §2 lacks `measurement:` (query / formula) | error |
| `QA-SLO-002` | SLO without budget | A SLO in §2 lacks an explicit `budget:` | error |
| `QA-ALERT-001` | Alert without runbook steps | A row in §6 has only "symptom" but no concrete remediation steps | error |
| `QA-ALERT-002` | Alert with hard-coded host/IP | warning (use service / DNS abstraction) |
| `QA-OBS-001` | Observability dashboard link doesn't resolve | warning |
| `QA-DR-001` | Disaster recovery section without last-drill outcome | error |
| `QA-OPS-001` | Common operation lacks rollback path | A row in §7 destructive op lacks `rollback:` | error |
| `QA-VENDOR-001` | Vendor escalation channel vague | A row in §10 lacks specific contact (URL / email / phone) | warning |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | SLOs in §2 match SLA commitments in any linked SOW for this service | warning |
| `XCHAIN-004` | Alerts in §6 are wired in the monitoring stack referenced in §8 (verified via tool link if available) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | `next_review_date` is in the past | Trigger review cycle | warning → needs_human |
| `STALE-002` | `last_drilled_at` >180 days old | Suggest tabletop or live drill | error |
| `STALE-003` | A source SLA in linked SOW changed since runbook `provenance.source_hash` | Reset SLO section to needs_human | error → needs_human |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/docs/module.md` §2(j) — Operations stage source
- Google SRE Book — SLOs, error budgets
- OpenTelemetry — observability conventions
