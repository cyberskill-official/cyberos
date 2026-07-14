---
id: NFR-RES-004
title: "RES hiring-memo signoff — CUO-generated memo MUST have hiring-manager + CHRO co-sign"
module: RES
category: compliance
priority: MUST
verification: T
phase: P1
slo: "100% of hiring memos carry both signatures before role goes to recruiting"
owner: CHRO
created: 2026-05-18
related_tasks: [TASK-RES-004]
---

## §1 — Statement (BCP-14 normative)

1. The hiring-memo CUO workflow (`TASK-RES-004`) **MUST** produce a memo signed by both the hiring manager + CHRO before the role is published to recruiting channels.
2. Memo content includes: role description, comp band, justification, target team, capacity-gap-source reference.
3. The capacity-gap reference **MUST** point to a specific over-alloc flag or strategic-plan line item — no "we just feel like hiring."
4. Single-signer attempts **MUST** be rejected.
5. Approved memos **MUST** be the source for the recruiting publication; ad-hoc job posts not tied to a memo are forbidden.

## §2 — Why this constraint

Headcount is the company's largest cost lever. Requiring a documented justification + dual signoff prevents informal hiring + budget drift. The capacity-gap link grounds the hire in measurable need. The "no posts without a memo" rule closes the back door of "we just put up a posting."

## §3 — Measurement

- Counter `res_hiring_memo_published_total{has_both_signatures}`.
- Counter `res_recruiting_post_without_memo_total` — must be 0.
- Audit row per memo.

## §4 — Verification

- Integration test (T) — single-signer → reject.
- Integration test (T) — post without memo → reject.
- CI gate (T) — recruiting publish path requires memo ref.

## §5 — Failure handling

- Single-signer attempt → block.
- Post without memo → block.
- Capacity-gap ref missing → block; workflow re-prompts.

---

*End of NFR-RES-004.*
