---
id: NFR-REW-007
title: "REW VietQR payroll batch integrity — batch hash MUST match sum of payslip nets"
module: REW
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of VietQR batches verifiable: |sum(batch.amounts) - sum(payslip.nets)| = 0"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-REW-009]
---

## §1 — Statement (BCP-14 normative)

1. Each VietQR payroll batch **MUST** be cryptographically tied to the payslip set it disburses: the batch row carries a SHA-256 hash of the canonical-sorted payslip-net list.
2. Pre-execution, the batch hash **MUST** be recomputed from the payslip table and compared against the batch row's stored hash; mismatch refuses execution.
3. Once submitted to NAPAS, the batch **MUST NOT** be mutable; corrections take the form of a separate compensating batch.
4. Batch settlement confirmation from NAPAS **MUST** be persisted with the batch row; unsettled batches > 24h trigger sev-2 alert.
5. Per-batch success/fail rate **MUST** be > 99% (banking rail availability) — sustained below triggers escalation to NAPAS support.

## §2 — Why this constraint

The VietQR batch is the actual money-movement event. A batch decoupled from its source payslips can silently disburse wrong amounts. The cryptographic binding + pre-execution recompute means tampering or corruption is detected before money moves. The immutable + compensating-batch pattern prevents post-submission rewrite (a fraud vector). NAPAS confirmation closes the loop — we know money was actually sent, not just submitted.

## §3 — Measurement

- Counter `rew_vietqr_hash_mismatch_total` — must be 0.
- Counter `rew_vietqr_unsettled_24h_total` — should be 0; > 0 triggers alert.
- Per-batch success rate aggregated monthly.

## §4 — Verification

- Integration test (T) — generate batch, tamper with payslip table, re-execute; assert hash mismatch refuses.
- Sandbox test (T) — full NAPAS sandbox submission + confirmation roundtrip.

## §5 — Failure handling

- Hash mismatch → block execution; CFO investigates.
- Unsettled > 24h → sev-2; check NAPAS gateway + bank-side.
- Per-batch success < 99% → escalate to NAPAS support; potentially fall back to alternate rail.

---

*End of NFR-REW-007.*
