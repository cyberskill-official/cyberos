---
id: NFR-REW-003
title: "REW bonus-pool conservation — pool inflow MUST equal sum of distributions + carry-over"
module: REW
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% conservation: |pool_in - (pool_distributed + carry_over)| < 1 VND per quarter"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-007, FR-REW-008]
---

## §1 — Statement (BCP-14 normative)

1. The bonus-pool ledger (`FR-REW-007`) **MUST** satisfy: every inflow row sums to the same total as the corresponding outflow rows (distribution) plus the carry-over balance, per quarter.
2. The conservation check **MUST** run automatically at every P3 quarterly distribution (`FR-REW-008`) and refuse to commit if conservation drifts > 1 VND.
3. Every ledger row **MUST** carry `{quarter, kind=inflow|distribution|carry_over, member_id?, amount_vnd, source, committed_at}`.
4. Ledger rows **MUST** be append-only; corrections take the form of compensating rows referencing the original.
5. The quarterly conservation result **MUST** be exposed in the P3 distribution attestation memo.

## §2 — Why this constraint

The bonus pool is real money flowing through the platform. Without conservation, the ledger drifts from reality — money appears or disappears silently, an accounting fraud signal. The append-only + compensation pattern is the standard double-entry-ledger discipline applied to bonus accounting. The auto-block at distribution time prevents bad data from propagating to payroll.

## §3 — Measurement

- Per-quarter `rew_bp_conservation_delta_vnd` — must be ≤ 1.
- Counter `rew_bp_correction_row_total` — surfaces correction frequency.

## §4 — Verification

- Integration test (T) — fixture ledger with known inflow + distribution; assert conservation holds.
- Property test (T) — random inflow/distribution sequences + corrections; assert invariant.
- Per-quarter manual review by CFO.

## §5 — Failure handling

- Conservation drift > 1 VND at distribution → distribution refused; CFO investigates.
- Two quarters in drift → sev-1; halt all P3 operations.
- Append-only violation (mutation detected) → sev-1; ledger trust broken; investigate immediately.

---

*End of NFR-REW-003.*
