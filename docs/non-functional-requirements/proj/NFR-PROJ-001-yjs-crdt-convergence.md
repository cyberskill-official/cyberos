---
id: NFR-PROJ-001
title: "PROJ Yjs CRDT convergence — concurrent edits MUST converge to identical state across all clients"
module: PROJ
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of concurrent edit sets converge within 2s of last network exchange"
owner: CTO
created: 2026-05-18
related_frs: [FR-PROJ-003]
---

## §1 — Statement (BCP-14 normative)

1. Concurrent edits to the same project artifact (issue, brief, cycle plan) via Yjs CRDT **MUST** converge to byte-identical state across all connected clients within 2s after the last network exchange.
2. Convergence applies regardless of edit interleaving — operations are commutative by Yjs construction; the host MUST NOT introduce non-commutative side effects.
3. Conflict resolution **MUST** be automatic; there is no "merge dialog" — Yjs's deterministic conflict resolution is the contract.
4. Y.Doc state **MUST** be persisted to Postgres on every server-side flush (default 5s window) so a server restart does not lose recent edits.
5. Clients reconnecting after offline editing **MUST** sync within 10s of reconnect for a Y.Doc of up to 10MB; larger docs scale linearly.

## §2 — Why this constraint

CRDT convergence is the foundation of multi-user collab on PROJ. Without it, users would see divergent state ("on my screen the issue says X, on yours Y"), erasing trust in the tool. The 2s post-exchange budget is the perception threshold above which collab feels "broken." The persisted state requirement ensures no edit loss on server crash. Offline-to-sync handles real-world disconnects gracefully.

## §3 — Measurement

- Histogram `proj_yjs_convergence_latency_seconds` — measured by synthetic multi-client probes.
- Counter `proj_yjs_divergence_detected_total` — must be 0.
- Histogram `proj_yjs_offline_sync_latency_seconds{doc_size_kb}`.

## §4 — Verification

- Integration test `modules/proj/tests/test_yjs_convergence.py` (T) — N clients drive concurrent edits; assert convergence.
- Chaos test (T) — partition network, edit on both sides, heal; assert convergence.
- Property test (T) — random edit sequences; assert state matches.

## §5 — Failure handling

- Divergence detected → sev-1; halt PROJ writes; investigate.
- Convergence > 2s p95 → sev-3; investigate network or server flush latency.
- Offline-sync > 10s for normal-size docs → sev-3; sync algorithm may be naive.

---

*End of NFR-PROJ-001.*
