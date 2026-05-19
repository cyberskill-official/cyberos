---
id: NFR-CUO-010
title: "CUO LangGraph Postgres checkpointer durability — checkpoint writes MUST be fsync'd"
module: CUO
category: reliability
priority: MUST
verification: T
phase: P0
slo: "0 lost checkpoints after process crash; recovery resumes from last fsync'd checkpoint"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-102]
---

## §1 — Statement (BCP-14 normative)

1. LangGraph state checkpoints **MUST** be written to Postgres with `synchronous_commit = on`; checkpoint writes that return success are durable.
2. On supervisor crash, the next start **MUST** resume from the last successfully-committed checkpoint — no chain is re-executed from scratch if it was mid-flight.
3. Checkpoint table schema **MUST** include `thread_id, checkpoint_id, parent_id, state, committed_at` plus `chain_id` to link to memory audit rows.
4. Checkpoints **MUST** be retained for a minimum of 30 days post-chain-end; older checkpoints can be archived/dropped.
5. Checkpoint write latency **MUST NOT** exceed 100ms p95 — beyond this, throughput collapses.

## §2 — Why this constraint

The Postgres checkpointer is what makes multi-step CUO chains crash-resilient. Without durable checkpoints, a supervisor crash mid-chain means either (a) re-running the entire chain (wasted work + duplicated side effects) or (b) silently dropping the chain (correctness violation). The `synchronous_commit` requirement is what gives the durability promise teeth — async commit can lose recent writes on crash. The 30-day retention matches operator review windows; the 100ms latency ceiling keeps checkpoint writes off the critical path.

## §3 — Measurement

- Histogram `cuo_checkpoint_write_latency_seconds`.
- Counter `cuo_checkpoint_resume_total{result=success|stale|missing}`.
- Gauge `cuo_checkpoint_table_row_count` — capacity planning.

## §4 — Verification

- Integration test `modules/cuo/tests/test_checkpoint_durability.py` (T) — write checkpoint, kill -9 the process, restart, assert resume.
- Chaos test (T) — power-loss simulation during checkpoint write; assert no torn writes.
- Schema test (T) — assert column set + constraints match spec.

## §5 — Failure handling

- Checkpoint write latency > 100ms p95 → sev-3; Postgres slow or row contention.
- Resume returns stale state (re-execution would dup side effects) → sev-2; investigate checkpoint integrity.
- Lost checkpoint after crash → sev-1; durability promise broken; halt CUO until RCA.

---

*End of NFR-CUO-010.*
