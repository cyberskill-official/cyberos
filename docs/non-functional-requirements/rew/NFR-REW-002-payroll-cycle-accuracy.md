---
id: NFR-REW-002
title: "REW payroll-cycle accuracy — monthly run MUST close within +/- 1 VND vs hand-verified sample"
module: REW
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of monthly runs within ±1 VND of a manual sample of 5 members per period"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-005, FR-REW-009]
---

## §1 — Statement (BCP-14 normative)

1. Each monthly payroll run **MUST** be sample-verified: 5 random members' gross/SI/PIT/net amounts hand-computed by the CFO + an external accountant and compared against the system output.
2. Discrepancy **MUST** be ≤ 1 VND per member; > 1 VND blocks the VietQR payroll batch from initiating.
3. The hand-verified sample **MUST** include at least one member from each contract type (probation, full-time, part-time, contractor) — coverage of the parameter-application matrix.
4. The sample audit **MUST** be documented in a per-cycle attestation memo stored at `docs/payroll-attestations/<period>.md`.
5. Two consecutive cycles with > 1 VND discrepancy **MUST** trigger a sev-1 review pausing all payroll until root cause identified.

## §2 — Why this constraint

Payroll inaccuracy is catastrophic — for employees (wrong paycheck, regulatory fines), for the company (back-tax liability, employee relations). The hand-verified sample is the platform's continuous proof-of-correctness. ±1 VND is the rounding floor; anything beyond that signals a real bug. The per-contract-type coverage ensures we exercise the matrix where bugs lurk. The two-cycle escalation is the "fool me once" rule.

## §3 — Measurement

- Per-cycle attestation memo (mandatory artifact).
- Counter `rew_payroll_discrepancy_total{period, member, delta_vnd}`.
- Annual report: 12 cycles, sum of discrepancies — target = 0.

## §4 — Verification

- Manual sample audit (T, monthly) — CFO + accountant.
- CI smoke (T) — fixture members with known gross/SI/PIT/net; assert system computes match.
- Automated cross-check against a second implementation (Excel reference) for 10% of members.

## §5 — Failure handling

- > 1 VND discrepancy → VietQR batch blocked; CFO investigates.
- Two-cycle escalation → sev-1; payroll halted; root-cause review.
- Missing attestation memo → sev-2 governance gap; cycle re-attested before next run.

---

*End of NFR-REW-002.*
