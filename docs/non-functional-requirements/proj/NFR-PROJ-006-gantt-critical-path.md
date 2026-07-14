---
id: NFR-PROJ-006
title: "PROJ Gantt critical path correctness — CPM algorithm MUST match reference output"
module: PROJ
category: reliability
priority: SHOULD
verification: T
phase: P1
slo: "100% of Gantt critical-path computations match a reference CPM implementation"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-PROJ-016]
---

## §1 — Statement (BCP-14 normative)

1. The Gantt view's critical-path-method (CPM) computation **MUST** match a reference implementation for any dependency graph the UI accepts (acyclic, single-source, single-sink, or general DAG).
2. Cycles in the dependency graph **MUST** be rejected at edge-creation time; the UI cannot create a graph with cycles.
3. CPM output **MUST** include `{critical_path_issue_ids, total_duration_days, slack_days_per_non_critical_issue}`.
4. Re-computation **MUST** trigger automatically on any dependency or duration mutation; UI shows the recomputed critical path within 1s.
5. The reference implementation is `modules/proj/cpm-reference/` — a small unit-tested CPM library used in CI fixtures.

## §2 — Why this constraint

Critical-path identification drives "what should we focus on?" decisions. A buggy CPM that hides the real critical path silently misroutes attention. The acyclic enforcement + closed reference + auto-recompute combination keeps the Gantt view trustworthy. The 1s response keeps UI feeling alive.

## §3 — Measurement

- Counter `proj_gantt_cpm_mismatch_total` — must be 0.
- Histogram `proj_gantt_cpm_compute_latency_ms`.
- Counter `proj_gantt_cycle_attempt_total` — surfaces attempted invalid edges.

## §4 — Verification

- Unit test (T) — fixture graphs vs reference; 50 cases.
- Integration test (T) — UI mutation triggers recompute < 1s.
- Property test (T) — random DAGs; assert CPM correctness.

## §5 — Failure handling

- Mismatch with reference → sev-2; halt Gantt; debug.
- Compute > 1s p95 → sev-3; graph size or algorithm performance issue.
- Cycle attempt > 10/min → sev-3; UI is allowing what it shouldn't.

---

*End of NFR-PROJ-006.*
