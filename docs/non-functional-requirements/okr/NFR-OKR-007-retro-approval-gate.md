---
id: NFR-OKR-007
title: "OKR retro-approval gate — quarterly retro MUST be CEO-signed before closing"
module: OKR
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of closed cycles carry a CEO-signed retro before close"
owner: CEO
created: 2026-05-18
related_frs: [FR-OKR-007]
---

## §1 — Statement (BCP-14 normative)

1. A cycle **MUST NOT** advance to `closed` without a CEO-signed retro.
2. The retro **MUST** include: outcomes per objective, lessons learned, action items for next cycle, links to evidence.
3. Auto-generated retro draft (`FR-OKR-007`) **MUST** be editable; CEO can revise before sign.
4. Once signed, the retro is immutable; corrections require an addendum.
5. Retro signatures **MUST** be persisted with `{signer_id, signed_at, retro_hash}`.

## §2 — Why this constraint

The retro is the cycle's closing artifact — without it, learnings dissipate. CEO signature signals leadership attention + accountability. The hash-on-sign + immutable rules prevent "we agreed it was a success" rewriting. Addendum-for-corrections preserves the original signed record.

## §3 — Measurement

- Counter `okr_cycle_close_no_retro_attempt_total` — must be 0.
- Audit row per retro signature.
- Gauge `okr_unclosed_cycles_past_retro_due`.

## §4 — Verification

- Integration test (T) — close without retro → reject.
- Snapshot test (T) — retro content + sign workflow.
- Mutation test (T) — post-sign edit → blocked.

## §5 — Failure handling

- Close-without-retro attempt → block.
- Mutation post-sign → sev-1; immutability broken.
- Unclosed cycles past due → sev-3 alert.

---

*End of NFR-OKR-007.*
