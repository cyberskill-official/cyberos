---
id: NFR-BRAIN-001
title: "BRAIN Layer-2 ingest lag — p95 < 1s from Layer-1 commit to l2_memory row visible"
module: BRAIN
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 1s; p99 < 3s end-to-end Layer-1 commit to Layer-2 visibility"
owner: CTO
created: 2026-05-18
related_frs: [FR-BRAIN-101, FR-BRAIN-108]
---

## §1 — Statement (BCP-14 normative)

1. From the moment a row is committed to a Layer-1 BRAIN store (per-actor SQLite append-log), the row **MUST** be visible in the global `l2_memory` Postgres table at **p95 < 1s** and **p99 < 3s**.
2. The ingest pipeline (`services/brain/src/layer2/ingest.rs`) **MUST** process Layer-1 binlog tails in cursor order; out-of-order ingest **MUST NOT** be possible.
3. Each ingest cycle **MUST** be transactional — the cursor advance and the `l2_memory` write commit together; failures rewind the cursor.
4. Pipeline back-pressure **MUST** be observable via the gauge `brain_layer2_ingest_lag_seconds` per (tenant_id, actor_id); sustained > 10s lag triggers a sev-2 alert.
5. The pipeline **MUST NOT** block Layer-1 writes — Layer-1 commits succeed locally even if Layer-2 ingest is paused.

## §2 — Why this constraint

Layer-1 → Layer-2 lag is the gap between "user/actor wrote something" and "the platform's memory has it." A 1s p95 budget means UX-perceived "I just told the assistant something and it remembers" feels instant. A 3s p99 budget tolerates occasional load spikes. The transactional ingest is the correctness invariant — without it, the cursor could advance past an unwritten row, silently losing data. The back-pressure observability tells operations whether ingest can keep up with write volume; the "don't block writes" rule ensures the Layer-2 backlog never propagates to user-facing latency.

## §3 — Measurement

- Histogram `brain_layer2_ingest_lag_seconds{tenant_id, actor_id}` measured per ingested row as `(l2_committed_at - l1_committed_at)`.
- Gauge `brain_layer2_pipeline_lag_seconds{tenant_id, actor_id}` — cursor distance behind tail in wall-clock seconds.
- BRAIN doctor invariant: `cursor.advance_at < l2_memory.max(committed_at) + 1s` always holds.

## §4 — Verification

- Integration test `services/brain/tests/ingest_lag_test.rs` (T) — writes 1000 Layer-1 rows; asserts p95 lag < 1s, p99 < 3s.
- Property test (T) — drives random write rates; asserts back-pressure surfaces in gauge before lag becomes critical.

## §5 — Failure handling

- p95 > 1s sustained → sev-3; investigate ingest worker CPU or DB contention.
- p99 > 10s → sev-2; pipeline is meaningfully behind; scale workers or pause non-critical writers.
- Cursor stuck (no advance for > 5 min) → sev-1; emergency intervention.

---

*End of NFR-BRAIN-001.*
