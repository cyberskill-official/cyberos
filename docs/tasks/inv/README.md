# INV module — task index

_Generated 2026-05-17 — 11 FRs, 67 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-INV-001](TASK-INV-001-invoice-draft-from-time-rollup/spec.md) | MUST | 1 | 8 | INV invoice substrate — draft invoices from TIME per-cycle rollup with rate-card snapshot preservati |
| [TASK-INV-002](TASK-INV-002-multi-currency/spec.md) | MUST | 1 | 6 | INV multi-currency support — VND/USD/SGD/EUR/GBP with daily SBV FX snapshot + per-invoice currency l |
| [TASK-INV-003](TASK-INV-003-stripe-webhook/spec.md) | MUST | 2 | 8 | INV Stripe webhook handler — Stripe-Signature verify + closed event-type allowlist + idempotent rece |
| [TASK-INV-004](TASK-INV-004-wise-webhook/spec.md) | SHOULD | 1 | 6 | Wise webhook handler for multi-currency receipts (USD / EUR / GBP / SGD / JPY) |
| [TASK-INV-005](TASK-INV-005-vietqr-webhook/spec.md) | MUST | 2 | 6 | INV VietQR / Napas247 webhook handler — HMAC-SHA256 signature + idempotent receipt insert + referenc |
| [TASK-INV-006](TASK-INV-006-cash-application/spec.md) | MUST | 2 | 8 | INV cash application — closed 4-step matching cascade (exact-ref → amount+date → fuzzy-fraction → ma |
| [TASK-INV-007](TASK-INV-007-vn-hoadon-autoemit/spec.md) | MUST | 2 | 6 | INV VN hóa đơn auto-emit on AM-send — Decree 123/2020 GDT XML signing + idempotent transmission + ve |
| [TASK-INV-008](TASK-INV-008-vn-hoadon-cancellation/spec.md) | MUST | 2 | 5 | INV VN hóa đơn cancellation flow — Decree 123 Art. 19 replacement-or-cancellation protocol with GDT  |
| [TASK-INV-009](TASK-INV-009-ar-aging-report/spec.md) | MUST | 2 | 4 | INV AR aging report — current/30/60/90/120+ bucket rollup per customer + per engagement with as-of d |
| [TASK-INV-010](TASK-INV-010-cuo-dunning-draft/spec.md) | MUST | 2 | 5 | INV CUO dunning draft — auto-generate polite/firm/legal-warning email drafts per aging bucket + CFO  |
| [TASK-INV-011](TASK-INV-011-revenue-recognition/spec.md) | MUST | 2 | 5 | INV revenue recognition — ASC 606 / IFRS 15 compliant deferred-revenue rollforward with monthly jour |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-INV-003→TASK-AUTH-101, TASK-INV-004→TASK-AUTH-101, TASK-INV-005→TASK-AUTH-101
- **CUO**: TASK-INV-010→TASK-CUO-101
- **EMAIL**: TASK-INV-010→TASK-EMAIL-009
- **TIME**: TASK-INV-001→TASK-TIME-009

**This module is depended on by:**

- **CRM**: TASK-CRM-010→TASK-INV-007
- **ESOP**: TASK-ESOP-004→TASK-INV-005
- **REW**: TASK-REW-009→TASK-INV-005
- **TEN**: TASK-TEN-003→TASK-INV-003, TASK-TEN-102→TASK-INV-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._