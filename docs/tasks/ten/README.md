# TEN module — task index

_Generated 2026-05-17 — 14 FRs, 124 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-TEN-001](TASK-TEN-001-provisioning-cli/spec.md) | MUST | 1 | 5 | TEN tenant provisioning CLI — `cyberos-ten provision` ops-driven flow with schema namespace + NATS s |
| [TASK-TEN-002](TASK-TEN-002-plan-tiers/spec.md) | MUST | 1 | 4 | 3 plan tiers (Starter / Team / Enterprise) hardcoded with per-tier caps |
| [TASK-TEN-003](TASK-TEN-003-stripe-billing/spec.md) | MUST | 2 | 8 | Stripe billing integration — USD/EUR/SGD/GBP customer + subscription + per-period invoice + overage  |
| [TASK-TEN-004](TASK-TEN-004-four-axis-metering/spec.md) | MUST | 1 | 8 | 4-axis metering — seats · API · AI tokens · storage (memory audit per metric event) |
| [TASK-TEN-005](TASK-TEN-005-vertical-pack-pricing/spec.md) | MUST | 2 | 5 | TEN vertical-pack pricing add-on — per-pack monthly fee (not per-seat) on top of base plan tier; mul |
| [TASK-TEN-101](TASK-TEN-101-self-serve-signup/spec.md) | MUST | 1 | 10 | Self-serve signup form ≤ 30 s end-to-end — email OTP + slug + plan + currency + payment + provisioni |
| [TASK-TEN-102](TASK-TEN-102-vnd-domestic-rail/spec.md) | MUST | 2 | 12 | VND domestic billing rail — VnPay + Momo + ZaloPay subscription, recurring-charge, refund, dunning + |
| [TASK-TEN-103](TASK-TEN-103-four-residency-provisioning/spec.md) | MUST | 2 | 10 | 4-residency provisioning — sg-1 / eu-1 / us-1 / vn-1 region pinning across Postgres + S3 + NATS + St |
| [TASK-TEN-104](TASK-TEN-104-offboarding-contract/spec.md) | MUST | 1 | 12 | TEN 90-day offboarding contract — closed 4-state FSM (Active → Terminating-A → Terminating-B → Termi |
| [TASK-TEN-105](TASK-TEN-105-signed-bundle-export/spec.md) | MUST | 2 | 8 | TEN signed-bundle export — deterministic zip + Ed25519 signature + memory audit anchor + chain-of-cus |
| [TASK-TEN-106](TASK-TEN-106-permanent-delete-attestation/spec.md) | MUST | 2 | 5 | TEN permanent-delete attestation — CSO + CLO dual-sign + chain-anchored evidence + cascade hard-purg |
| [TASK-TEN-107](TASK-TEN-107-tenant-admin-spa/spec.md) | SHOULD | 3 | 16 | TEN tenant-admin SPA — seats + billing + audit + residency + retention dashboard for ROOT-CFO tenant |
| [TASK-TEN-201](TASK-TEN-201-sg-holdco-flip/spec.md) | MUST | 1 | 16 | TEN Singapore HoldCo flip CLI — `cyberos-ten holdco-flip` orchestrates ACRA filings + shareholder mi |
| [TASK-TEN-202](TASK-TEN-202-hostile-termination-override/spec.md) | SHOULD | 1 | 5 | TEN hostile-termination override — legal-trigger fast-track with CEO+CLO+CSO triple-sign for hostile |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-TEN-004→TASK-AI-001, TASK-TEN-103→TASK-AI-016
- **AUTH**: TASK-TEN-001→TASK-AUTH-001, TASK-TEN-004→TASK-AUTH-003, TASK-TEN-101→TASK-AUTH-104
- **memory**: TASK-TEN-004→TASK-MEMORY-111
- **ESOP**: TASK-TEN-201→TASK-ESOP-001
- **INV**: TASK-TEN-003→TASK-INV-003, TASK-TEN-102→TASK-INV-005
- **SKILL**: TASK-TEN-005→TASK-SKILL-107

**This module is depended on by:**

- **PORTAL**: TASK-PORTAL-001→TASK-TEN-101, TASK-PORTAL-002→TASK-TEN-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._