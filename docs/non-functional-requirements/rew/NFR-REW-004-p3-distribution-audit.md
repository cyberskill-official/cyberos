---
id: NFR-REW-004
title: "REW P3 distribution audit — quarterly distribution MUST have CFO+CHRO co-sign before execution"
module: REW
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of P3 distributions carry CFO + CHRO signatures; no auto-execute without both"
owner: CFO
created: 2026-05-18
related_frs: [FR-REW-008]
---

## §1 — Statement (BCP-14 normative)

1. Every P3 quarterly distribution **MUST** be approved by both CFO and CHRO via signed approval rows before execution; single signature is insufficient.
2. Approval window **MUST** be ≤ 72 hours between CFO and CHRO signatures; > 72h requires re-submission (stale approval risk).
3. The approval row **MUST** reference the exact distribution plan (a deterministic hash of `{period, member_set, amount_set}`).
4. Post-approval mutation of the distribution plan **MUST** invalidate prior approvals and trigger re-submission.
5. Execution **MUST** be auto-paused if either signer is unavailable beyond 72h; manual escalation path (CEO override) is documented.

## §2 — Why this constraint

Quarterly bonus distributions are six-to-seven-figure events. Single-signer approval is insufficient — concentrated authority + ill intent or honest error would silently misroute funds. The two-signer requirement implements separation of duties: CFO attests to the numbers, CHRO attests to the people. The 72h staleness rule prevents an old "yes" being used against a quietly-changed plan. The deterministic hash binds approval to exact-plan.

## §3 — Measurement

- Counter `rew_p3_single_signer_attempt_total` — must be 0.
- Histogram `rew_p3_approval_gap_hours{stage}` — surfaces approval delays.
- Audit row for every approval / re-submission / override.

## §4 — Verification

- Integration test (T) — distribution with only CFO sign → rejected; both signs within 72h → admitted.
- Property test (T) — mutate plan post-approval; assert approval invalidated.
- CI gate — approval-flow code path always requires both signers.

## §5 — Failure handling

- Single-signer attempt → reject + audit.
- 72h gap exceeded → re-submit; possibly route to CEO override.
- Mutation invalidation → re-submit; CFO + CHRO re-attest.

---

*End of NFR-REW-004.*
