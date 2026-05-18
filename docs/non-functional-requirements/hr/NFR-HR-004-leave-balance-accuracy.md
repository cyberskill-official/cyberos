---
id: NFR-HR-004
title: "HR leave-balance accuracy — accrued - used = balance within ±0.01 days"
module: HR
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of member leave balances match accrual - usage within ±0.01 days"
owner: CHRO
created: 2026-05-18
related_frs: [FR-HR-004, FR-HR-006]
---

## §1 — Statement (BCP-14 normative)

1. Per-member per-leave-type balance **MUST** satisfy: `accrued - used - corrections = current_balance`, within ±0.01 days.
2. The reconciliation runs daily; drift > 0.01 days surfaces a sev-3.
3. Accrual + usage rows are append-only; corrections require explicit compensating rows with `correction_reason`.
4. Leave-balance display in UI **MUST** match the reconciliation source-of-truth at all times.
5. Cross-leave-type pool moves (e.g., banking unused annual leave) **MUST** be reflected in both source + destination type with matched values.

## §2 — Why this constraint

Leave balances are member-facing and legally significant (employees take leave based on what they see). Inaccurate balances produce disputes + scheduling issues. The append-only + compensation pattern is the audit-friendly leave-ledger. The daily reconciliation surfaces silent bugs before they accumulate.

## §3 — Measurement

- Daily reconciliation: `hr_leave_balance_drift_days{member, leave_type}` — must be < 0.01.
- Counter `hr_leave_correction_row_total{correction_reason}`.
- Counter `hr_leave_pool_move_total`.

## §4 — Verification

- Integration test (T) — accrue + use + correct; assert balance.
- Property test (T) — random sequences; assert invariant.

## §5 — Failure handling

- Drift > 0.01 → sev-3; investigate per-member.
- Append-only violation → sev-1.
- Pool-move mismatch → sev-2.

---

*End of NFR-HR-004.*
