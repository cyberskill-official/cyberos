---
id: TASK-MCP-007
title: "MCP Tasks primitive — handles, status poll, cancel lifecycle"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: mcp
priority: p0
status: reviewing
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-MCP-006, TASK-MCP-008]
depends_on: [TASK-MCP-001, TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#tasks
  - https://modelcontextprotocol.io/specification/2025-11-25/server/tools#long-running

source_decisions:
  - DEC-1100 2026-05-17 — Tasks primitive: long-running tool calls return a handle + status poll + final result
  - DEC-1101 2026-05-17 — Closed enum task_status = {pending, running, completed, failed, cancelled, expired}; cardinality 6
  - DEC-1114 2026-05-17 — Closed enum task_progress_unit = {percent, items, bytes, none}; cardinality 4
  - "DEC-1106 (as-built slice) — Postgres mcp_tasks store-of-record exists (migration 0017 + tasks_pg); full reconnect/worker resume still deferred"

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/tasks.rs
  - services/mcp-gateway/src/tasks_pg.rs
  - services/mcp-gateway/migrations/0017_mcp_tasks.sql
modified_files:
  - services/mcp-gateway/src/lib.rs
  - services/mcp-gateway/src/router.rs
  - services/mcp-gateway/src/oauth/audit.rs

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_write: services/mcp-gateway/{src,migrations}/**
  - bash: cd services && cargo test -p cyberos-mcp-gateway tasks

disallowed_tools:
  - claim worker pool / NATS / checkpoints / tools/call async routing as shipped
  - claim phantom services/mcp/src/tasks/** tree as the live path

effort_hours: 6
subtasks:
  - "2.0h: tasks.rs lifecycle store + unit tests"
  - "2.0h: tasks_pg.rs + migration 0017"
  - "1.0h: audit kinds on oauth::audit"
  - "1.0h: batch/9a-mcp re-spec + audit"

risk_if_skipped: "Without a tasks handle API, any tool call that outlives one HTTP request has no honest status/result contract."
---

# TASK-MCP-007: MCP Tasks primitive (lifecycle adopt)

## Summary

Provide an opaque task handle with a closed status enum, start → running → terminal lifecycle, result fetch, and cancellation. As-built: in-memory `tasks.rs` (dev-real contract) plus `tasks_pg.rs` + migration `0017_mcp_tasks.sql` as the Postgres store-of-record. Unit tests cover start/complete/cancel/unknown-id and enum cardinalities.

## Problem

The engineering-spec claimed a ten-file `services/mcp/src/tasks/` tree, three migrations (0009–0011), worker pools, NATS progress, checkpoints, `long_running` tools/call routing, idempotency, rate limits, TTL sweeper, prune, and fifteen `task_*` integration tests. HEAD has consolidated `tasks.rs` / `tasks_pg.rs` and migration 0017; the request-path async router and worker infrastructure remain deferred (already named in source comments).

## Proposed Solution

Adopt the shipped lifecycle:

- `TaskStatus` (6) + `TaskProgressUnit` (4) with `as_str` / `is_terminal`
- In-memory `TaskStore`: start, complete, fail, cancel, status, result
- `tasks_pg`: sealed payloads, caller-scoped rows, ready for worker wiring
- Audit helpers for task lifecycle kinds on `oauth::audit`

Ledger everything else under Out of scope so HITL can accept an honest adopt.

## Alternatives Considered

- **Hold the task until worker pool + NATS land.** Rejected: recreates stuck-WIP; lifecycle contract is already valuable and tested.
- **Claim tools/call long_running routing as done.** Rejected: not wired; would be a false done-flip.
- **Keep the old 10-file tree claim.** Rejected: FM-004 + path lies.

## Success Metrics

- Primary: start returns a handle; status moves running → completed/failed/cancelled; result fetchable; unknown id errors cleanly; enums stay cardinality-pinned.
- Guardrail: no AC claims deferred worker/NATS/checkpoint/async-routing surfaces.

## Scope

In scope:

- `src/tasks.rs` lifecycle + unit tests
- `src/tasks_pg.rs` + `migrations/0017_mcp_tasks.sql`
- Related `oauth::audit` task_* helpers that exist today

### Out of scope / Non-Goals

- Per-module worker pool / bounded concurrency queue draining
- NATS progress push
- Checkpoints + crash resume
- `long_running` annotation + tools/call async routing (request-path task creator)
- Idempotency keys, per-tenant create rate limit
- TTL sweeper / 30-day prune
- Phantom `services/mcp/src/tasks/**` and claimed `task_*` integration filenames

## Dependencies

`depends_on: [TASK-MCP-001, TASK-MCP-004]`. Soft: TASK-MCP-006/008 for future destructive-at-start confirm on long-running tools (not required for this lifecycle adopt).

## 1. Description (normative)

- 1.1 `TaskStatus::ALL` MUST have cardinality 6; `TaskProgressUnit::ALL` cardinality 4.
- 1.2 Starting a task MUST return an opaque id and leave the task in a non-terminal running (or pending) state until completed/failed/cancelled.
- 1.3 Completed tasks MUST expose a fetchable result; failed tasks MUST record an error; cancel MUST be terminal and block further transitions.
- 1.4 Unknown task ids MUST error cleanly.
- 1.5 When Postgres is configured, `tasks_pg` MUST persist caller-scoped sealed rows per migration 0017.
- 1.6 This adopt MUST NOT claim Out-of-scope worker/NATS/checkpoint/async-routing surfaces as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - status and unit cardinalities pinned - test: `services/mcp-gateway/src/tasks.rs::status_and_unit_have_the_pinned_cardinalities`
- [ ] AC 2 (traces_to: #1.2,#1.3) - start runs then completes with fetchable result - test: `services/mcp-gateway/src/tasks.rs::start_runs_then_completes_with_a_fetchable_result`
- [ ] AC 3 (traces_to: #1.3) - cancel is terminal and blocks further transitions - test: `services/mcp-gateway/src/tasks.rs::cancel_is_terminal_and_blocks_further_transitions`
- [ ] AC 4 (traces_to: #1.3) - fail records the error - test: `services/mcp-gateway/src/tasks.rs::fail_records_the_error`
- [ ] AC 5 (traces_to: #1.4) - unknown task id errors cleanly - test: `services/mcp-gateway/src/tasks.rs::unknown_task_id_errors_cleanly`
- [ ] AC 6 (traces_to: #1.5) - migration 0017 and tasks_pg module exist - verify: `services/mcp-gateway/migrations/0017_mcp_tasks.sql` and `services/mcp-gateway/src/tasks_pg.rs`
- [ ] AC 7 (traces_to: #1.6) - Out of scope lists worker/NATS/checkpoints/async routing - verify: `docs/tasks/mcp/TASK-MCP-007-tasks-primitive/spec.md` Scope → Out of scope

## Verification

```bash
cd services && cargo test -p cyberos-mcp-gateway tasks::
```

| Path | Covers |
|------|--------|
| `src/tasks.rs` unit tests | Lifecycle contract |
| `src/tasks_pg.rs` + `migrations/0017_mcp_tasks.sql` | Store-of-record presence |
| `src/db_slice_test.rs` (Postgres-gated / ignored without pool) | DB path when available |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9a-mcp`.
- **Scope:** Re-spec/adopt lifecycle surface only; deferred worker/NATS/async routing ledgered.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9a-mcp adopt — TASK-MCP-007 re-spec against as-built tasks.rs / tasks_pg.*
