# INV module — feature request index

_Generated 2026-05-17 — 11 FRs, 67 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-INV-001](FR-INV-001-invoice-draft-from-time-rollup/spec.md) | MUST | 1 | 8 | INV invoice substrate — draft invoices from TIME per-cycle rollup with rate-card snapshot preservati |
| [FR-INV-002](FR-INV-002-multi-currency/spec.md) | MUST | 1 | 6 | INV multi-currency support — VND/USD/SGD/EUR/GBP with daily SBV FX snapshot + per-invoice currency l |
| [FR-INV-003](FR-INV-003-stripe-webhook/spec.md) | MUST | 2 | 8 | INV Stripe webhook handler — Stripe-Signature verify + closed event-type allowlist + idempotent rece |
| [FR-INV-004](FR-INV-004-wise-webhook/spec.md) | SHOULD | 1 | 6 | Wise webhook handler for multi-currency receipts (USD / EUR / GBP / SGD / JPY) |
| [FR-INV-005](FR-INV-005-vietqr-webhook/spec.md) | MUST | 2 | 6 | INV VietQR / Napas247 webhook handler — HMAC-SHA256 signature + idempotent receipt insert + referenc |
| [FR-INV-006](FR-INV-006-cash-application/spec.md) | MUST | 2 | 8 | INV cash application — closed 4-step matching cascade (exact-ref → amount+date → fuzzy-fraction → ma |
| [FR-INV-007](FR-INV-007-vn-hoadon-autoemit/spec.md) | MUST | 2 | 6 | INV VN hóa đơn auto-emit on AM-send — Decree 123/2020 GDT XML signing + idempotent transmission + ve |
| [FR-INV-008](FR-INV-008-vn-hoadon-cancellation/spec.md) | MUST | 2 | 5 | INV VN hóa đơn cancellation flow — Decree 123 Art. 19 replacement-or-cancellation protocol with GDT  |
| [FR-INV-009](FR-INV-009-ar-aging-report/spec.md) | MUST | 2 | 4 | INV AR aging report — current/30/60/90/120+ bucket rollup per customer + per engagement with as-of d |
| [FR-INV-010](FR-INV-010-cuo-dunning-draft/spec.md) | MUST | 2 | 5 | INV CUO dunning draft — auto-generate polite/firm/legal-warning email drafts per aging bucket + CFO  |
| [FR-INV-011](FR-INV-011-revenue-recognition/spec.md) | MUST | 2 | 5 | INV revenue recognition — ASC 606 / IFRS 15 compliant deferred-revenue rollforward with monthly jour |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-INV-003→FR-AUTH-101, FR-INV-004→FR-AUTH-101, FR-INV-005→FR-AUTH-101
- **CUO**: FR-INV-010→FR-CUO-101
- **EMAIL**: FR-INV-010→FR-EMAIL-009
- **TIME**: FR-INV-001→FR-TIME-009

**This module is depended on by:**

- **CRM**: FR-CRM-010→FR-INV-007
- **ESOP**: FR-ESOP-004→FR-INV-005
- **REW**: FR-REW-009→FR-INV-005
- **TEN**: FR-TEN-003→FR-INV-003, FR-TEN-102→FR-INV-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._