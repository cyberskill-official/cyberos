---
id: NFR-TEN-006
title: "TEN hostile-override approval window — emergency termination requires CEO + CLO co-sign"
module: TEN
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of hostile-override terminations carry CEO + CLO signatures within 24h window"
owner: CTO
created: 2026-05-18
related_frs: [FR-TEN-202]
---

## §1 — Statement (BCP-14 normative)

1. Hostile termination (FR-TEN-202) — used when a tenant is committing abuse and the normal 90-day FSM is too slow — **MUST** require both CEO and CLO-Legal signatures within a 24-hour window.
2. The override **MUST** be timestamped + sealed; immediately persists to BRAIN with `kind=tenant.hostile_override`.
3. Single-signer override is forbidden — no exceptions, including for the founder.
4. The override action **MUST** include a written rationale (≥ 100 words) explaining the abuse and the legal basis for accelerated termination.
5. Quarterly review by external counsel of all hostile overrides in the period — proportionality and process audit.

## §2 — Why this constraint

Hostile override is the platform's nuclear button against abusive tenants. Single-signer authority would be too much concentrated power — both for ethical reasons and abuse-resistance. The 24-hour window forces the two signers to coordinate fresh — stale approvals can't be used. The written rationale provides the legal-defensibility record. Quarterly external review ensures the platform doesn't normalise overuse.

## §3 — Measurement

- Counter `ten_hostile_override_total{tenant, signers_count}` — single-signer = 0.
- Audit row per override + rationale.
- Quarterly external review report.

## §4 — Verification

- Integration test (T) — single CEO sign → reject.
- Integration test (T) — both sign within 24h → admitted.
- Property test (T) — stale signature; assert rejected.

## §5 — Failure handling

- Single-signer attempt → block + sev-2 (governance risk).
- 24h window exceeded → require fresh signatures.
- Quarterly review finds disproportionate use → ops + governance review.

---

*End of NFR-TEN-006.*
