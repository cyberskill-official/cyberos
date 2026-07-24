---
id: TASK-MCP-008
title: "MCP elicitation store with confirmation round-trip"
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
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-MCP-006, TASK-MCP-007]
depends_on: [TASK-MCP-001, TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#elicitation
  - https://modelcontextprotocol.io/specification/2025-11-25/server/utilities/elicitation

source_decisions:
  - DEC-1140 2026-05-17 — Elicitation per MCP 2025-11-25: server raises structured prompts; caller responds; tool/gate resumes
  - DEC-1141 2026-05-17 — Closed enum elicitation_type = {string_input, single_choice, multi_choice, confirmation, file_upload}; cardinality 5
  - DEC-1142 2026-05-17 — Response validated against the fixed per-type schema before the elicitation is recorded as responded
  - DEC-1144 2026-05-17 — Pending elicitation stored with TTL columns (timeout sweeper deferred; rows + expires_at ship in migration 0016)
  - DEC-1145 2026-05-17 — HTTP polling REQUIRED for pending elicitations; NATS push OPTIONAL and deferred in this adopt
  - DEC-1148 2026-05-17 — Caller can cancel a pending elicitation via POST cancel
  - DEC-1149 2026-05-17 — Memory audit kinds: mcp.elicitation_requested, responded, timeout, cancelled, validation_failed
  - DEC-1150 2026-05-17 — Invalid response retries capped at 3 then terminal validation_failed
  - DEC-1151 2026-05-17 — TASK-MCP-006 destructive gating satisfied by confirmation elicitation
  - "DEC-1152 2026-05-17 — Confirmation type fixed schema { confirmed: boolean, reason?: string }"
  - DEC-1156 2026-05-17 — Idempotent resubmit of the same response_payload returns AlreadyRecorded
  - DEC-1159 2026-05-17 — Cross-caller elicitation access denied (handler-scoped; GUC RLS deferred)

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/elicitation.rs
  - services/mcp-gateway/src/elicitation_pg.rs
  - services/mcp-gateway/migrations/0016_mcp_elicitations.sql
modified_files:
  - services/mcp-gateway/src/router.rs
  - services/mcp-gateway/src/lib.rs
  - services/mcp-gateway/src/gating.rs
  - services/mcp-gateway/src/oauth/audit.rs
  - services/mcp-gateway/src/db_slice_test.rs
  - services/mcp-gateway/src/kms.rs

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_write: services/mcp-gateway/{src,migrations}/**
  - bash: cd services && cargo test -p cyberos-mcp-gateway elicitation

disallowed_tools:
  - claim NATS push, LISTEN/NOTIFY, S3 file_upload infra, rate limit, timeout sweeper, or 30-day prune as shipped
  - skip schema validation on respond
  - allow one caller to respond to or poll another caller's elicitation on the DB path

effort_hours: 5
subtasks:
  - "1.0h: elicitation.rs types, schemas, validate_response, in-memory ElicitationStore"
  - "1.0h: elicitation_pg.rs create/respond/cancel/pending/confirmation_state + KMS seal"
  - "0.5h: migration 0016_mcp_elicitations.sql"
  - "1.0h: router REST poll/respond/cancel + TASK-MCP-006 confirmation wire-up"
  - "1.0h: unit + router + #[ignore] db_slice persistence tests"
  - "0.5h: oauth/audit.rs elicitation_* emitters"

risk_if_skipped: "Without elicitation, destructive gating (TASK-MCP-006) has no confirmation channel and mid-call clarifications cannot round-trip. The as-built in-memory + PG store lands the confirmation primitive the gateway already depends on."
---

# TASK-MCP-008: MCP elicitation store with confirmation round-trip

## Summary

Ship the MCP elicitation primitive in `services/mcp-gateway/`: closed 5-type / 5-status enums, fixed per-type response schemas, in-memory `ElicitationStore` for the no-DB path, Postgres store-of-record via `elicitation_pg.rs` + migration `0016_mcp_elicitations.sql`, and REST poll/respond/cancel. The load-bearing use is confirmation round-trip for TASK-MCP-006 destructive gating; enums/schemas for other types (including `file_upload`) exist, but full infra for NATS, LISTEN/NOTIFY, S3 upload, rate limit, timeout sweeper, and prune is deferred.

## Problem

Server-initiated mid-call prompts are part of MCP 2025-11-25. Without them, missing args force fail-and-retry, and destructive tools cannot present a structured confirm/decline. The original engineering-spec targeted `services/mcp/src/elicitation/` with NATS push, LISTEN/NOTIFY wakeups, S3 presign, timeout jobs, and twelve `elicitation_*` integration test files — that tree was never built. As-built lives under `mcp-gateway` as two modules + one migration, with real in-crate and router tests.

## Proposed Solution

**In-memory (`elicitation.rs`):** `ElicitationType` / `ElicitationStatus` (cardinality 5 each), `response_schema`, `validate_response`, `ElicitationStore` with create / create_confirmation / respond / cancel / pending / confirmation_state / is_confirmed, retry cap `MAX_RETRIES=3`, idempotent resubmit.

**Postgres (`elicitation_pg.rs` + `0016_mcp_elicitations.sql`):** write-through store-of-record when a pool + authenticated caller exist — create_confirmation, confirmation_state, pending, respond (KMS-sealed payload + denormalized `confirmed`), cancel. Caller-scoped (DEC-1159) in the handler; append-only GRANT model (GUC RLS deferred).

**HTTP (`router.rs`):** `GET /v1/mcp/elicitations`, `POST /v1/mcp/elicitations/:id/respond`, `POST /v1/mcp/elicitations/:id/cancel`. Destructive `tools/call` (TASK-MCP-006) creates a confirmation elicitation and consults `confirmation_state` on re-invoke.

**Audit (`oauth/audit.rs`):** emitters for requested / responded / cancelled / validation_failed; `elicitation_timeout` defined for the deferred sweeper.

## Alternatives Considered

- **Full NATS push + LISTEN/NOTIFY blocking wait inside tool workers.** Deferred: sync gateway uses poll + re-invoke; no long-running worker wait loop in this adopt.
- **S3-presigned file_upload end-to-end.** Deferred: schema/enum exist; upload URL generation and object verify do not ship here.
- **Multi-file `elicitation/{request,response,poll,cancel,timeout_job,validate,file_upload,nats_push}.rs` tree under `services/mcp/`.** Rejected: as-built is `elicitation.rs` + `elicitation_pg.rs` in `mcp-gateway`.
- **Enable Postgres RLS policies with `auth.tenant_id` GUC now.** Deferred: gateway does not set that GUC yet; isolation is handler `caller_subject_id` checks (documented in migration 0016).

## Success Metrics

- Primary: confirmation create → respond `{confirmed:true|false}` → gate proceeds or aborts; invalid payloads 422 then terminal at retry cap; cancel clears pending; DB path seals payloads and is caller-scoped.
- Guardrail: unit + router tests green without claiming NATS/S3/sweeper/prune; DB persistence covered by `#[ignore]` integration test when Postgres is available.

## Scope

In scope (as-built under `services/mcp-gateway/`):

- `src/elicitation.rs` — types, schemas, validation, in-memory store + unit tests
- `src/elicitation_pg.rs` — PG store-of-record for confirmation + respond/cancel/pending
- `migrations/0016_mcp_elicitations.sql` — enums, table, indexes, writer grants
- Router REST poll/respond/cancel + TASK-MCP-006 confirmation integration
- Audit emitters in `oauth/audit.rs` for the lifecycle kinds used on the hot path
- `db_slice_test.rs::elicitation_persists_seals_and_is_caller_scoped` (Postgres-gated)

### Out of scope / Non-Goals

- NATS push transport
- Postgres LISTEN/NOTIFY wakeups for blocking tool-side `elicit()`
- S3 / TASK-DOC-001 presigned `file_upload` realization (enums/schemas may exist; infra deferred)
- Elicitation creation rate limit
- Timeout sweeper job (columns exist; sweeper deferred; `elicitation_timeout` audit reserved)
- 30-day response-payload prune job
- GUC-based RLS policies (handler isolation only in this adopt)
- Claimed `elicitation_*` integration test filenames that do not exist (cite real in-crate / router / db_slice tests)
- Sync-tool ban / TaskCtx::elicit long-running API (TASK-MCP-007 worker story)

## Dependencies

`depends_on: [TASK-MCP-001, TASK-MCP-004]` — gateway router + authenticated callers. Soft: **TASK-MCP-006** consumes confirmation elicitation for destructive gating (reciprocal integration). Related: TASK-MCP-007 (task-bound elicitations / worker wait deferred).

## 1. Description (normative)

- 1.1 `ElicitationType::ALL` and `ElicitationStatus::ALL` MUST each contain exactly five variants with snake_case wire labels matching migration `0016` enums.
- 1.2 `response_schema` + `validate_response` MUST implement the fixed per-type shapes (string_input, single_choice, multi_choice, confirmation, file_upload schema) without requiring a general JSON Schema engine.
- 1.3 In-memory `ElicitationStore` MUST support create_confirmation, respond (Recorded / Invalid / ValidationFailed / AlreadyRecorded), cancel, pending, and confirmation_state.
- 1.4 Invalid responses MUST increment retries up to `MAX_RETRIES=3` then transition to `validation_failed`.
- 1.5 Identical resubmit of an already-recorded payload MUST be idempotent (`AlreadyRecorded`).
- 1.6 Migration `0016_mcp_elicitations.sql` MUST create `mcp_elicitations` with the closed enums, TTL columns, `confirmed` denormalized column, and append-only writer grants.
- 1.7 When a DB pool + authenticated caller are present, router MUST use `elicitation_pg` for create/respond/cancel/pending and MUST scope rows to `caller_subject_id`.
- 1.8 Router MUST expose `GET /v1/mcp/elicitations`, `POST .../respond`, `POST .../cancel`, and TASK-MCP-006 MUST create/consult confirmation elicitations on destructive `tools/call`.
- 1.9 This adopt MUST NOT claim NATS, LISTEN/NOTIFY, S3 file_upload infra, rate limit, timeout sweeper, or prune as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - type and status enums have exactly five variants with correct wire labels - test: `services/mcp-gateway/src/elicitation.rs::type_and_status_have_exactly_five_variants`
- [ ] AC 2 (traces_to: #1.2) - confirmation schema requires confirmed boolean - test: `services/mcp-gateway/src/elicitation.rs::confirmation_schema_requires_confirmed_boolean`
- [ ] AC 3 (traces_to: #1.2) - validate_response accepts bool and rejects missing/wrong types - test: `services/mcp-gateway/src/elicitation.rs::validate_confirmation_accepts_bool_rejects_missing`
- [ ] AC 4 (traces_to: #1.2) - string and choice types validate correctly - test: `services/mcp-gateway/src/elicitation.rs::validate_string_and_choice_types`
- [ ] AC 5 (traces_to: #1.3,#1.8) - confirmation round-trip accept and decline - test: `services/mcp-gateway/src/elicitation.rs::confirmation_round_trip_accept_and_decline`
- [ ] AC 6 (traces_to: #1.4) - invalid responses retry then go terminal - test: `services/mcp-gateway/src/elicitation.rs::invalid_responses_retry_then_go_terminal`
- [ ] AC 7 (traces_to: #1.3) - confirmation_state reflects approval, decline, and absence - test: `services/mcp-gateway/src/elicitation.rs::confirmation_state_reflects_approval_decline_and_absence`
- [ ] AC 8 (traces_to: #1.5) - idempotent resubmit and cancel - test: `services/mcp-gateway/src/elicitation.rs::idempotent_resubmit_and_cancel`
- [ ] AC 9 (traces_to: #1.8) - HTTP poll/respond/validate/404 - test: `services/mcp-gateway/src/router.rs::elicitation_poll_respond_validate_and_404`
- [ ] AC 10 (traces_to: #1.8) - destructive gating hold/confirm/decline via elicitation - test: `services/mcp-gateway/src/router.rs::destructive_tool_without_confirmation_is_held`
- [ ] AC 11 (traces_to: #1.6,#1.7) - PG path seals payload and is caller-scoped (Postgres required) - test: `services/mcp-gateway/src/db_slice_test.rs::elicitation_persists_seals_and_is_caller_scoped`
- [ ] AC 12 (traces_to: #1.9) - Out of scope lists NATS/LISTEN/NOTIFY/S3/rate-limit/sweeper/prune; no AC claims them shipped - verify: `docs/tasks/mcp/TASK-MCP-008-elicitation/spec.md` Scope → Out of scope

## Verification

```bash
cd services && cargo test -p cyberos-mcp-gateway elicitation
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::elicitation_poll_respond_validate_and_404
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::destructive_
# Postgres-gated (ignored without DATABASE_URL / local pool):
cd services && cargo test -p cyberos-mcp-gateway --lib db_slice_test::elicitation_persists_seals_and_is_caller_scoped -- --ignored
```

Real tests to cite (do **not** cite non-existent `elicitation_*` integration filenames):

| Path | Covers |
|------|--------|
| `services/mcp-gateway/src/elicitation.rs` unit tests listed in ACs 1–8 | Enums, schemas, store semantics |
| `services/mcp-gateway/src/router.rs::elicitation_poll_respond_validate_and_404` | REST surface |
| `services/mcp-gateway/src/router.rs` destructive_* tests | TASK-MCP-006 confirmation integration |
| `services/mcp-gateway/src/db_slice_test.rs::elicitation_persists_seals_and_is_caller_scoped` | PG seal + caller scope + idempotency (`#[ignore]` without Postgres) |
| `services/mcp-gateway/migrations/0016_mcp_elicitations.sql` | Schema / enums (applied by migrate path) |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9a-mcp`, rewriting the engineering-spec body into task@1 grammar against as-built `services/mcp-gateway/` sources.
- **Scope:** Spec re-scoped to in-memory + PG confirmation/REST surface; NATS, LISTEN/NOTIFY, S3 file_upload infra, rate limit, sweeper, and prune ledgered under Out of scope; paths corrected to `mcp-gateway`; ACs cite real tests only.
- **Human review:** Required at the two HITL gates; this file is the batch/9a-mcp adopt rework (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9a-mcp adopt — TASK-MCP-008 re-spec against as-built mcp-gateway elicitation.*
