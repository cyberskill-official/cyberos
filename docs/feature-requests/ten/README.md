# TEN module — feature request index

_Generated 2026-05-17 — 14 FRs, 124 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-TEN-001](FR-TEN-001-provisioning-cli.md) | MUST | 1 | 5 | TEN tenant provisioning CLI — `cyberos-ten provision` ops-driven flow with schema namespace + NATS s |
| [FR-TEN-002](FR-TEN-002-plan-tiers.md) | MUST | 1 | 4 | 3 plan tiers (Starter / Team / Enterprise) hardcoded with per-tier caps |
| [FR-TEN-003](FR-TEN-003-stripe-billing.md) | MUST | 2 | 8 | Stripe billing integration — USD/EUR/SGD/GBP customer + subscription + per-period invoice + overage  |
| [FR-TEN-004](FR-TEN-004-four-axis-metering.md) | MUST | 1 | 8 | 4-axis metering — seats · API · AI tokens · storage (memory audit per metric event) |
| [FR-TEN-005](FR-TEN-005-vertical-pack-pricing.md) | MUST | 2 | 5 | TEN vertical-pack pricing add-on — per-pack monthly fee (not per-seat) on top of base plan tier; mul |
| [FR-TEN-101](FR-TEN-101-self-serve-signup.md) | MUST | 1 | 10 | Self-serve signup form ≤ 30 s end-to-end — email OTP + slug + plan + currency + payment + provisioni |
| [FR-TEN-102](FR-TEN-102-vnd-domestic-rail.md) | MUST | 2 | 12 | VND domestic billing rail — VnPay + Momo + ZaloPay subscription, recurring-charge, refund, dunning + |
| [FR-TEN-103](FR-TEN-103-four-residency-provisioning.md) | MUST | 2 | 10 | 4-residency provisioning — sg-1 / eu-1 / us-1 / vn-1 region pinning across Postgres + S3 + NATS + St |
| [FR-TEN-104](FR-TEN-104-offboarding-contract.md) | MUST | 1 | 12 | TEN 90-day offboarding contract — closed 4-state FSM (Active → Terminating-A → Terminating-B → Termi |
| [FR-TEN-105](FR-TEN-105-signed-bundle-export.md) | MUST | 2 | 8 | TEN signed-bundle export — deterministic zip + Ed25519 signature + memory audit anchor + chain-of-cus |
| [FR-TEN-106](FR-TEN-106-permanent-delete-attestation.md) | MUST | 2 | 5 | TEN permanent-delete attestation — CSO + CLO dual-sign + chain-anchored evidence + cascade hard-purg |
| [FR-TEN-107](FR-TEN-107-tenant-admin-spa.md) | SHOULD | 3 | 16 | TEN tenant-admin SPA — seats + billing + audit + residency + retention dashboard for ROOT-CFO tenant |
| [FR-TEN-201](FR-TEN-201-sg-holdco-flip.md) | MUST | 1 | 16 | TEN Singapore HoldCo flip CLI — `cyberos-ten holdco-flip` orchestrates ACRA filings + shareholder mi |
| [FR-TEN-202](FR-TEN-202-hostile-termination-override.md) | SHOULD | 1 | 5 | TEN hostile-termination override — legal-trigger fast-track with CEO+CLO+CSO triple-sign for hostile |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-TEN-004→FR-AI-001, FR-TEN-103→FR-AI-016
- **AUTH**: FR-TEN-001→FR-AUTH-001, FR-TEN-004→FR-AUTH-003, FR-TEN-101→FR-AUTH-104
- **memory**: FR-TEN-004→FR-MEMORY-111
- **ESOP**: FR-TEN-201→FR-ESOP-001
- **INV**: FR-TEN-003→FR-INV-003, FR-TEN-102→FR-INV-005
- **SKILL**: FR-TEN-005→FR-SKILL-107

**This module is depended on by:**

- **PORTAL**: FR-PORTAL-001→FR-TEN-101, FR-PORTAL-002→FR-TEN-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._