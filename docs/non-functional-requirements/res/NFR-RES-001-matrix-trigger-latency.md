---
id: NFR-RES-001
title: "RES matrix-trigger latency — over/under flags MUST surface within 1h of capacity change"
module: RES
category: observability
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 1h from allocation change to over/under flag visibility"
owner: COO
created: 2026-05-18
related_frs: [FR-RES-001, FR-RES-003]
---

## §1 — Statement (BCP-14 normative)

1. The capacity-demand matrix (`FR-RES-001`) **MUST** refresh on allocation, hiring, or scope mutations within 1h.
2. Over-allocated (> 100% capacity) and under-allocated (< 50% capacity) flags **MUST** surface in the resource dashboard within the refresh cadence.
3. The matrix **MUST** include forward-looking horizon (4 weeks default) — flags warn before the period starts, not after.
4. Manual recompute **MUST** be available via UI action for COO + team leads.
5. Matrix data **MUST** be exportable for offline planning (CSV).

## §2 — Why this constraint

Capacity issues that surface after the period began are too late. The 1h freshness + 4-week horizon together let leaders adjust before pain hits. The flag thresholds (100% / 50%) are operational defaults; tenant-configurable for different industries.

## §3 — Measurement

- Histogram `res_matrix_refresh_latency_seconds`.
- Counter `res_over_alloc_flag_active{member}`.
- Counter `res_under_alloc_flag_active{member}`.

## §4 — Verification

- Integration test (T) — mutate allocation; assert refresh within 1h.
- Snapshot test (T) — over/under flags rendered.
- Manual recompute (T) — UI action triggers.

## §5 — Failure handling

- Refresh > 1h → sev-3; investigate cron.
- Active flags > 25% of staff → sev-3; product retrospective.
- Manual recompute fail → sev-3 UI bug.

---

*End of NFR-RES-001.*
