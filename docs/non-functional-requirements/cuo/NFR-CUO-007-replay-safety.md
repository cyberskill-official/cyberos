---
id: NFR-CUO-007
title: "CUO replay safety — replayed chain MUST produce same audit row count + order"
module: CUO
category: reliability
priority: MUST
verification: T
phase: P1
slo: "100% of replayed chains match original step count + skill chain + outcome"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-103, FR-CUO-105]
---

## §1 — Statement (BCP-14 normative)

1. Replaying a previously-executed chain (via `cyberos-cuo execute --replay <chain_id>`) **MUST** produce the same `step_count`, same `skill_chain`, same `outcome` as the original.
2. Replay **MUST** read from the original chain's persisted state (Phase-2 step output JSON files + Phase-3 memory rows) — it does NOT re-execute external side effects.
3. If the underlying workflow has been updated (newer `version:`) the replay **MUST** use the version recorded in the original audit row, NOT the latest.
4. If the recorded workflow version is no longer present in the catalog, replay **MUST** refuse with `E_REPLAY_VERSION_GONE` rather than substitute a current version.
5. Per-step rollback (FR-CUO-105) **MUST** preserve replay safety — rolled-back steps remain visible to replay as `outcome=rolled_back`.

## §2 — Why this constraint

Replay is the platform's audit-rehydration promise: "given a chain id, show me exactly what ran." If replay could produce different outputs depending on current catalog state, the promise breaks — audit history becomes fiction. The "no re-execute side effects" rule is the safety net: replay reads, never writes. The "refuse if version gone" rule errs on the side of explicit failure instead of silent substitution. Per-step rollback respect ensures the partial-state semantics survive into replay.

## §3 — Measurement

- Counter `cuo_replay_attempt_total{result=success|version_gone|mismatch}`.
- Quarterly: replay 100 random historical chains; assert 100% match.
- Counter `cuo_replay_mismatch_total` — must be 0.

## §4 — Verification

- Integration test `modules/cuo/tests/test_replay.py` (T) — execute chain, replay, assert matching.
- Quarterly production replay drill (T) — operator-driven; surfaces drift not caught by CI fixtures.
- Property test (T) — random chains executed + replayed; assert invariant holds.

## §5 — Failure handling

- Single replay mismatch → sev-2; replay safety has a bug; investigate via the specific chain.
- Replay refuses version-gone → expected; operator works with audit row directly.
- Replay quarterly drill > 1% mismatch → sev-1; replay reliability has regressed; halt new feature work until fixed.

---

*End of NFR-CUO-007.*
