---
id: NFR-PROJ-002
title: "PROJ billable cascade correctness — rate-card change MUST propagate to all unbilled time"
module: PROJ
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of unbilled time entries reflect the current rate-card within 60s of rate change"
owner: CFO
created: 2026-05-18
related_frs: [FR-PROJ-005, FR-PROJ-006]
---

## §1 — Statement (BCP-14 normative)

1. When a rate-card row is created/updated/deactivated, all **unbilled** time entries falling under that rate-card's scope **MUST** recompute their `billable_amount` within 60s.
2. **Billed** time entries (already invoiced) **MUST NOT** be recomputed — their amount is frozen at invoice time.
3. The cascade **MUST** be transactional per time entry: either the new amount is committed or the entry stays at the old amount; no torn state.
4. The cascade **MUST** emit an audit row per recomputed entry: `{entry_id, old_amount, new_amount, rate_card_id, rate_card_version, cascaded_at}`.
5. Cascade failures (DB write fail) **MUST** be retried with exponential backoff; sustained failures (> 5 min) trigger sev-2.

## §2 — Why this constraint

Rate-card changes are routine — promotions, role changes, project-specific rates. Without cascade, time entries silently retain stale amounts and invoices ship with wrong numbers. The billed-frozen rule preserves invoice integrity (regulators expect immutable invoices). The transactional per-entry approach + audit row makes the cascade observable and reversible.

## §3 — Measurement

- Histogram `proj_billable_cascade_latency_seconds` — time from rate change to last entry recomputed.
- Counter `proj_billable_cascade_entry_total{result=success|skipped_billed|failed}`.
- Audit row per recomputation.

## §4 — Verification

- Integration test (T) — change rate card; assert all unbilled entries recompute within 60s.
- Property test (T) — rate-card change while invoices are mid-creation; assert no torn state.
- Reconciliation: sum of audit-row deltas equals net change in `sum(billable_amount)` per scope.

## §5 — Failure handling

- Cascade > 60s p95 → sev-3; large unbilled set; investigate worker scale.
- Failed entry retry exhausted → sev-2; manual intervention.
- Billed entry mistakenly recomputed → sev-1; invoice integrity broken; halt + remediate.

---

*End of NFR-PROJ-002.*
