# DOC module — task index

_Generated 2026-05-17 — 11 FRs, 103 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-DOC-001](TASK-DOC-001-document-repository/spec.md) | MUST | 1 | 8 | DOC Document repository — S3 Object-Lock Compliance bucket + per-tenant residency pinning + versione |
| [TASK-DOC-002](TASK-DOC-002-eidas-qtsp/spec.md) | MUST | 3 | 16 | DOC eIDAS QTSP integration — GlobalSign or Cryptomathic partner for EU residency qualified signature |
| [TASK-DOC-003](TASK-DOC-003-aatl-ca/spec.md) | MUST | 3 | 12 | DOC AATL CA integration — Adobe Approved Trust List CA partner (DigiCert / Entrust / IdenTrust) for  |
| [TASK-DOC-004](TASK-DOC-004-vn-ca-chain/spec.md) | MUST | 3 | 16 | DOC VN CA chain — VNeID + VnPay/MK Group/Viettel-CA partners for VN-residency qualified digital sign |
| [TASK-DOC-005](TASK-DOC-005-multi-party-signing/spec.md) | MUST | 2 | 10 | DOC multi-party signing workflow — ordered + parallel + counter-sign with reminder cadence and full  |
| [TASK-DOC-006](TASK-DOC-006-identity-verification/spec.md) | MUST | 2 | 8 | DOC identity verification — 4 methods (WebAuthn / VNeID / SMS-OTP / email-link) with per-document me |
| [TASK-DOC-007](TASK-DOC-007-lifecycle-metadata/spec.md) | MUST | 1 | 5 | DOC lifecycle metadata — parties + effective_date + expiry_date + renewal_terms + parent_contract_id |
| [TASK-DOC-008](TASK-DOC-008-expiry-alert-cascade/spec.md) | MUST | 1 | 4 | DOC expiry alert cascade — 90/30/7-day notifications to parties + CLO with deduplication and snooze  |
| [TASK-DOC-009](TASK-DOC-009-renewal-proposal-cuo/spec.md) | SHOULD | 1 | 6 | DOC renewal proposal CUO draft — auto-generate renewal terms + price adjustment + send-to-customer f |
| [TASK-DOC-010](TASK-DOC-010-third-party-import/spec.md) | SHOULD | 3 | 10 | DOC third-party import — DocuSign / Adobe Sign / HelloSign migration with LTV (long-term-validation) |
| [TASK-DOC-011](TASK-DOC-011-padesblt-restamping/spec.md) | MUST | 3 | 8 | DOC PAdES-B-LT format + year-9 LTV re-stamping — extend B-T signatures with validation data + re-tim |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-DOC-001→TASK-AUTH-101, TASK-DOC-006→TASK-AUTH-105
- **CUO**: TASK-DOC-009→TASK-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._