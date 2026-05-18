---
id: NFR-EMAIL-003
title: "EMAIL BIMI freshness — VMC + BIMI record MUST stay valid; auto-alert on expiry"
module: EMAIL
category: maintainability
priority: SHOULD
verification: A
phase: P1
slo: "BIMI VMC + DNS record validated weekly; alert ≥ 30 days before expiry"
owner: CTO
created: 2026-05-18
related_frs: [FR-EMAIL-004]
---

## §1 — Statement (BCP-14 normative)

1. The tenant's BIMI Verified Mark Certificate (VMC) + DNS BIMI record **MUST** be validated weekly via the platform's BIMI prober.
2. Expiry alerts **MUST** fire 30, 14, 7, and 1 day(s) before VMC expiry — multiple cadences ensure the operator sees it.
3. DNS BIMI record drift (record removed or modified incorrectly) **MUST** be detected within 24h and alerted.
4. Tenants without BIMI configured **MUST NOT** trigger alerts — BIMI is opt-in.
5. Validation failure does NOT block email send; it only triggers operator alerts (brand-display degrades gracefully).

## §2 — Why this constraint

BIMI is the brand-display authentication on supported mail clients (Gmail, Apple Mail). Expired or broken BIMI silently removes the brand logo from inbound mail at recipients — a brand-trust degradation that's invisible from inside the system. The escalating-cadence alerts ensure the operator can't miss it. The "doesn't block send" rule is deliberate: send is more important than logo display.

## §3 — Measurement

- Gauge `email_bimi_vmc_days_until_expiry{tenant}`.
- Counter `email_bimi_validation_failure_total{tenant, reason}`.
- Counter `email_bimi_drift_alert_total`.

## §4 — Verification

- Weekly prober (A) — fetches VMC + DNS record; asserts validity.
- Synthetic recipient (A) — confirms BIMI logo renders in test inbox.
- Quarterly review of alert cadence + operator follow-through.

## §5 — Failure handling

- VMC expiring < 30d → SHOULD alert; operator renews.
- VMC expired → sev-3; brand display broken; renew ASAP.
- DNS drift → sev-3; investigate config change.

---

*End of NFR-EMAIL-003.*
