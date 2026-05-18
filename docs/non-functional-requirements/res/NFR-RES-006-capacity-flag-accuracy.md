---
id: NFR-RES-006
title: "RES capacity-flag accuracy — flag MUST match human-confirmed over/under state ≥ 95% of time"
module: RES
category: reliability
priority: SHOULD
verification: T
phase: P1
slo: "Flag agreement with COO confirmation ≥ 95% over a 4-week sample"
owner: COO
created: 2026-05-18
related_frs: [FR-RES-003]
---

## §1 — Statement (BCP-14 normative)

1. The capacity flags (over-alloc, under-alloc) **MUST** match COO's human assessment of the same member ≥ 95% of the time over a 4-week rolling sample.
2. Disagreements are reviewed by the COO + flagged for algorithm retuning.
3. False positives ("flagged over-allocated but actually fine") **MUST NOT** exceed 5% — too many false alarms cause flag fatigue.
4. False negatives ("not flagged but actually over-allocated") **MUST NOT** exceed 5%.
5. The flag algorithm + thresholds **MUST** be documented + versioned.

## §2 — Why this constraint

Flags are only useful if they're trustworthy. The 95% accuracy floor preserves trust. The dual-direction false-positive/negative budget catches both flag-fatigue and silent issues. The doc + version requirement makes retuning auditable.

## §3 — Measurement

- Weekly: COO samples 10 flags + 10 unflagged; computes agreement rate.
- Counter `res_flag_false_positive_total`, `res_flag_false_negative_total`.

## §4 — Verification

- Quarterly sample audit by COO.
- Algorithm-version test on fixture data.

## §5 — Failure handling

- Agreement < 95% → sev-3; retune thresholds.
- FP > 5% → fatigue risk; retune.
- FN > 5% → coverage hole; investigate.

---

*End of NFR-RES-006.*
