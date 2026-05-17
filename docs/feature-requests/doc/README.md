# DOC module — feature request index

_Generated 2026-05-17 — 11 FRs, 103 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-DOC-001](FR-DOC-001-document-repository.md) | MUST | 1 | 8 | DOC Document repository — S3 Object-Lock Compliance bucket + per-tenant residency pinning + versione |
| [FR-DOC-002](FR-DOC-002-eidas-qtsp.md) | MUST | 3 | 16 | DOC eIDAS QTSP integration — GlobalSign or Cryptomathic partner for EU residency qualified signature |
| [FR-DOC-003](FR-DOC-003-aatl-ca.md) | MUST | 3 | 12 | DOC AATL CA integration — Adobe Approved Trust List CA partner (DigiCert / Entrust / IdenTrust) for  |
| [FR-DOC-004](FR-DOC-004-vn-ca-chain.md) | MUST | 3 | 16 | DOC VN CA chain — VNeID + VnPay/MK Group/Viettel-CA partners for VN-residency qualified digital sign |
| [FR-DOC-005](FR-DOC-005-multi-party-signing.md) | MUST | 2 | 10 | DOC multi-party signing workflow — ordered + parallel + counter-sign with reminder cadence and full  |
| [FR-DOC-006](FR-DOC-006-identity-verification.md) | MUST | 2 | 8 | DOC identity verification — 4 methods (WebAuthn / VNeID / SMS-OTP / email-link) with per-document me |
| [FR-DOC-007](FR-DOC-007-lifecycle-metadata.md) | MUST | 1 | 5 | DOC lifecycle metadata — parties + effective_date + expiry_date + renewal_terms + parent_contract_id |
| [FR-DOC-008](FR-DOC-008-expiry-alert-cascade.md) | MUST | 1 | 4 | DOC expiry alert cascade — 90/30/7-day notifications to parties + CLO with deduplication and snooze  |
| [FR-DOC-009](FR-DOC-009-renewal-proposal-cuo.md) | SHOULD | 1 | 6 | DOC renewal proposal CUO draft — auto-generate renewal terms + price adjustment + send-to-customer f |
| [FR-DOC-010](FR-DOC-010-third-party-import.md) | SHOULD | 3 | 10 | DOC third-party import — DocuSign / Adobe Sign / HelloSign migration with LTV (long-term-validation) |
| [FR-DOC-011](FR-DOC-011-padesblt-restamping.md) | MUST | 3 | 8 | DOC PAdES-B-LT format + year-9 LTV re-stamping — extend B-T signatures with validation data + re-tim |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-DOC-001→FR-AUTH-101, FR-DOC-006→FR-AUTH-105
- **CUO**: FR-DOC-009→FR-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._