---
id: NFR-REW-006
title: "REW gross/net reconciliation — gross - deductions MUST equal net for every payslip"
module: REW
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of payslips satisfy gross - (PIT + SI + other deductions) = net within ±1 VND"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-REW-005, TASK-REW-006]
---

## §1 — Statement (BCP-14 normative)

1. Every payslip row **MUST** satisfy the identity: `gross_vnd - (pit_vnd + si_employee_vnd + advance_recovery_vnd + other_deductions_vnd) = net_vnd`, within ±1 VND tolerance for rounding.
2. The reconciliation check **MUST** run inline during payslip generation; failures block payslip emit + VietQR batch.
3. Per-cycle reconciliation summary **MUST** be exposed: `sum(gross) - sum(deductions) = sum(net)` — fleet-level check.
4. Reconciliation failures **MUST** surface a per-member breakdown of the contributing components so CFO can debug.
5. The check **MUST** include zero-net edge case (gross fully consumed by deductions); zero-net is legal but the payslip is still emitted.

## §2 — Why this constraint

Gross-net reconciliation is the most basic payroll arithmetic invariant. Failure usually means a deduction row was missed or double-applied. Inline checking + block-on-fail catches these bugs before payslips reach employees. The fleet-level check catches issues that per-row checks might miss (e.g., a deduction applied to wrong member). The zero-net carve-out documents the legal case (someone with very low gross + statutory deductions).

## §3 — Measurement

- Per-cycle counter `rew_payslip_reconciliation_fail_total{period}` — must be 0.
- Per-cycle gauge `rew_payslip_total_drift_vnd` — must be ≤ |members| (each contributes ≤ 1 VND).

## §4 — Verification

- Integration test (T) — fixture cycle with known deductions; assert per-payslip + fleet reconciliation.
- Property test (T) — random gross/deduction sets; assert identity holds.

## §5 — Failure handling

- Per-payslip fail → block emit; CFO investigates member.
- Fleet drift > tolerance → sev-1; halt cycle; root-cause.
- Zero-net edge case mishandled → sev-2; payslip emit logic has a bug.

---

*End of NFR-REW-006.*
