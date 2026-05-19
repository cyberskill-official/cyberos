---
id: NFR-REW-005
title: "REW memory exclusion — comp data MUST NOT land in memory Layer-1 or Layer-2"
module: REW
category: privacy
priority: MUST
verification: T
phase: P0
slo: "0 memory rows referencing REW comp tables; CI gate enforces per DEC-036"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-010]
---

## §1 — Statement (BCP-14 normative)

1. Per DEC-036, comp/payroll data (salary, bonus, 3P income, distributions) **MUST NOT** be written to memory Layer-1 or Layer-2 — REW maintains its own segregated audit log.
2. The CI gate (`FR-REW-010`) **MUST** scan every memory-write call-site in `services/rew/` and assert none reference comp tables (`rew_*`, `payroll_*`, `bp_ledger`, `p3_distribution`).
3. The REW-specific audit log **MUST** be retained separately at `services/rew/audit/` with its own retention policy (7 years per VN tax law).
4. References to comp data from other modules' memory rows (e.g., HR linking to payroll period) **MUST** use opaque identifiers, never amounts or formulas.
5. Any CI failure on the memory-exclusion gate **MUST** block merge unconditionally.

## §2 — Why this constraint

Comp data has elevated sensitivity (highly-restricted access; legal disclosure requirements). Co-mingling with memory — which has broad operator + LLM read access — would create a wide blast radius for any compromise. DEC-036 made the architectural call: REW has its own audit chain. The CI gate is what gives that decision teeth — without the gate, well-meaning developers would slowly drift comp rows into memory.

## §3 — Measurement

- CI metric `rew_memory_exclusion_gate_violations` — must be 0.
- Audit log size per quarter — surfaces volume.

## §4 — Verification

- CI gate `rew-memory-exclusion` (T) — static scan + dynamic test against fixture comp ops; assert no memory writes.
- Quarterly review (T) — CISO + CFO inspect `services/rew/` for memory-import drift.

## §5 — Failure handling

- CI gate violation → block merge, contributor fixes.
- Production memory row found referencing comp table → sev-1; halt; remove row; postmortem.
- REW audit log loss > 0 rows → sev-1; investigate (separate from memory durability budget).

---

*End of NFR-REW-005.*
