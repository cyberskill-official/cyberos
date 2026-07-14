---
id: NFR-TEN-007
title: "TEN four-axis metering precision — users/storage/AI/seats measured within ±0.1%"
module: TEN
category: reliability
priority: MUST
verification: T
phase: P0
slo: "Metered amounts accurate within ±0.1% vs ground-truth reconciliation"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-TEN-004]
---

## §1 — Statement (BCP-14 normative)

1. The four metering axes (active users, storage GB-month, AI tokens, seats) **MUST** report values accurate to within ±0.1% of ground-truth (independent recount from DB).
2. Metering pipelines **MUST** be at-least-once with idempotent dedup — no double-counting.
3. Reconciliation runs daily; drift > 0.1% triggers sev-3.
4. Per-tenant per-axis monthly invoice values **MUST** be the source of truth for billing — once invoiced, frozen.
5. Pre-invoice metering corrections are allowed (drift fix); post-invoice corrections require credit-memo workflow.

## §2 — Why this constraint

Metering accuracy is the foundation of usage-based billing. Customers will audit; meters that drift produce customer disputes + revenue loss. The ±0.1% tolerance accommodates clock-skew + race conditions in a distributed counter; tighter would be unrealistic. The "frozen after invoice" rule is standard accounting practice — invoices must be immutable.

## §3 — Measurement

- Daily reconciliation: `ten_metering_drift_pct{axis}` — must be < 0.1%.
- Counter `ten_metering_double_count_event_total` — must be 0.
- Per-axis monthly aggregate published.

## §4 — Verification

- Daily reconciliation (T) — meter vs DB recount.
- Property test (T) — burst events; assert dedup.
- Integration test (T) — pre vs post-invoice; assert immutability post.

## §5 — Failure handling

- Drift > 0.1% → sev-3; reconcile.
- Double-count detected → sev-2; idempotency broken.
- Post-invoice mutation → sev-1.

---

*End of NFR-TEN-007.*
