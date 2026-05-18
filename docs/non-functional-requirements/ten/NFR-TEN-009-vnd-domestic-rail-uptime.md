---
id: NFR-TEN-009
title: "TEN VND domestic rail uptime — VND-billing path MUST maintain 99.5% monthly availability"
module: TEN
category: reliability
priority: MUST
verification: A
phase: P0
slo: "99.5% monthly availability for VND-billing initiation + settlement"
owner: CFO
created: 2026-05-18
related_frs: [FR-TEN-102]
---

## §1 — Statement (BCP-14 normative)

1. The VND domestic billing rail (NAPAS + bank integration) **MUST** maintain 99.5% monthly availability for initiation + settlement.
2. Failover to a secondary rail provider **MUST** activate within 5 minutes of primary outage detection.
3. Per-attempt latency budget: p95 < 3s for VND payment initiation.
4. Per-month settlement rate (% of attempts settled within 24h) **MUST** stay ≥ 99%.
5. Monthly SLO miss triggers a postmortem with bank/NAPAS — if it's their issue, the platform's vendor management protocol kicks in.

## §2 — Why this constraint

VND domestic rail is the platform's primary billing channel for VN-resident tenants. An outage means stalled invoices + delayed cash. The 99.5% floor is realistic for banking-rail availability (banks themselves rarely exceed 99.9% for online services). Failover within 5 minutes contains user impact. The settlement floor is the contractual minimum.

## §3 — Measurement

- Synthetic prober (60s cadence) against payment initiation endpoint.
- Histogram `ten_vnd_rail_settlement_hours`.
- Monthly availability per rail provider.

## §4 — Verification

- Synthetic monitoring (A).
- Chaos drill (A) — failover to secondary; assert ≤ 5 min.
- Monthly availability report.

## §5 — Failure handling

- Single-provider down → failover.
- Both providers down → sev-1; CFO + ops on operator-comms to tenants.
- Monthly SLO miss → vendor postmortem.

---

*End of NFR-TEN-009.*
