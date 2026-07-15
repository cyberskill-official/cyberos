---
id: NFR-DOC-006
title: "DOC expiry alert latency — alerts MUST fire at 90/30/7/1 day(s) before document expiry"
module: DOC
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of tracked-expiry documents fire alerts at the four scheduled cadences within ±1 day"
owner: CLO-Legal
created: 2026-05-18
related_tasks: [TASK-DOC-008]
---

## §1 — Statement (BCP-14 normative)

1. Documents with `expiry_at:` metadata **MUST** trigger alerts at 90, 30, 7, and 1 day(s) before expiry, each within ±1 day of the scheduled point.
2. Alerts **MUST** target the document owner + the CLO-Legal admin role + the renewal-proposal CUO workflow (`TASK-DOC-009`).
3. Each alert fires exactly once per cadence per document (idempotent on retry).
4. Missing/malformed `expiry_at` **MUST NOT** silently disable alerts — the document import gate refuses to accept docs with declared category requiring expiry but missing the field.
5. Already-expired documents **MUST** stay alerted at daily cadence until owner action.

## §2 — Why this constraint

Document expiry (contracts, NDAs, licenses) silently turning into legal liability is the most common DOC failure mode. The four-cadence alert ladder gives the owner ample warning. The CUO workflow auto-trigger provides drafting headstart. The "still alert daily after expiry" rule keeps overdue items visible.

## §3 — Measurement

- Counter `doc_expiry_alert_fired_total{cadence}`.
- Counter `doc_expiry_alert_missed_total` — must be 0.
- Gauge `doc_overdue_unresolved_count`.

## §4 — Verification

- Integration test (T) — set expiry; advance clock; assert all 4 alerts fire.
- CI gate (T) — declared-category docs require expiry field.
- Daily reconciliation against the alert schedule.

## §5 — Failure handling

- Missed cadence → sev-3; investigate scheduler.
- Overdue unresolved > 30 → sev-3; legal liability accumulation.
- Schedule scheduler dead → sev-2; alerts fail silently if scheduler down.

---

*End of NFR-DOC-006.*
