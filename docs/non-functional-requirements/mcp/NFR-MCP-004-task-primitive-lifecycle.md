---
id: NFR-MCP-004
title: "MCP task primitive lifecycle — task state transitions MUST be linear + auditable"
module: MCP
category: reliability
priority: MUST
verification: T
phase: P1
slo: "100% of task state transitions follow the spec FSM; 0 backward transitions"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-MCP-007]
---

## §1 — Statement (BCP-14 normative)

1. Tasks (per the spec `tasks` primitive) **MUST** transition through states `pending → running → (succeeded | failed | cancelled)` — no backward transitions; no state outside the closed set.
2. Each state transition **MUST** be timestamped and persisted; the task object surface exposes the full transition log via `task/get`.
3. A task in `running` for longer than its declared `maxDurationSeconds` **MUST** be transitioned to `cancelled` with reason `timeout`.
4. Task cancellation from the client (`task/cancel`) **MUST** propagate to the underlying skill invocation within 5s.
5. Completed tasks **MUST** retain their state + output for at least 24 hours; expiry **MUST** be observable via the `expiresAt` field.

## §2 — Why this constraint

MCP tasks are long-running operations. Without strict FSM enforcement, a task could be observed in inconsistent states across clients, or could leak resources by staying "running" forever. The linear-transition rule simplifies all reasoning: state at time T is the most recent transition. Time-boxing prevents runaway tasks. Cancellation propagation under 5s keeps the user-perceived "stop" responsive. 24h retention matches the typical operator review window.

## §3 — Measurement

- Histogram `mcp_task_lifetime_seconds{outcome}`.
- Counter `mcp_task_invalid_transition_total{from, to}` — must always be 0.
- Histogram `mcp_task_cancel_propagation_latency_seconds`.

## §4 — Verification

- Unit test (T) — drive every legal transition; assert allowed.
- Unit test (T) — attempt illegal backward transition; assert rejected with error.
- Integration test (T) — cancel a running task; assert skill is stopped within 5s.

## §5 — Failure handling

- Invalid transition attempt → rejected + counter; should be 0 in production.
- Cancel propagation > 5s p95 → sev-3; skill runtime doesn't honor cancel signal.
- Stuck task (running > 2× max duration) → sev-3; force-transition + investigate.

---

*End of NFR-MCP-004.*
