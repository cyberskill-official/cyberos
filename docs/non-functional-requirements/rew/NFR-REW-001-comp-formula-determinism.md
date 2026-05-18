---
id: NFR-REW-001
title: "REW comp-formula determinism — same inputs MUST produce same payroll output across reruns"
module: REW
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of payroll runs are bit-identical on rerun with same inputs + parameter version"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-005, FR-REW-002]
---

## §1 — Statement (BCP-14 normative)

1. The monthly payroll compute (`FR-REW-005`) **MUST** produce byte-identical outputs when rerun with the same `{member_set, parameter_version, period, 3p_income_set}` tuple.
2. Floating-point arithmetic **MUST** use `Decimal` (Python) or `rust_decimal` (Rust) with explicit precision (10 digits past the point for VND); no IEEE-754 float anywhere in the compute path.
3. Iteration order over the member set **MUST** be deterministic (sorted by member_id ascending).
4. The parameter version (`FR-REW-002`) **MUST** be stamped onto every output row — the same period can have multiple recomputes under different versions and they are NOT merged.
5. Compute outputs **MUST** be hashed (SHA-256 of the canonical CSV/JSON) and the hash stored alongside the output — operators can verify integrity at any time.

## §2 — Why this constraint

Payroll is the platform's most legally-attestable computation. Non-determinism would mean "the same set of facts produces different paychecks" — an immediate audit-fail signal. Float arithmetic introduces ULP-level drift that compounds; Decimal arithmetic eliminates it. The parameter-version stamping is what makes recomputes safe — operators can preview a new statutory-deduction table without overwriting last month's run. The hash check is the catch-all integrity verify.

## §3 — Measurement

- Hash comparison on every recompute; counter `rew_payroll_recompute_hash_mismatch_total` — must be 0 if inputs unchanged.
- Counter `rew_float_arithmetic_violation_total` — static linter; must be 0.
- CI: rerun a fixture payroll 10×; assert bit-identical.

## §4 — Verification

- Unit test (T) — fixture members + params; rerun 10×; assert hashes match.
- Static lint (T) — grep for `float(` / `f32` / `f64` in `services/rew/`; assert 0 matches in compute path.
- Property test (T) — random member sets; assert determinism holds.

## §5 — Failure handling

- Hash mismatch on supposed-same-inputs rerun → sev-1; payroll determinism is broken; halt all REW writes; investigate.
- Float arithmetic detected → CI block.
- Parameter-version stamp missing → sev-1; output rows not traceable to source; halt.

---

*End of NFR-REW-001.*
