---
id: TASK-MCP-006
title: "MCP destructive-tool gating via elicitation confirmation"
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
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-MCP-007, TASK-MCP-008]
depends_on: [TASK-MCP-001, TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#tool-annotations
  - https://modelcontextprotocol.io/specification/2025-11-25/server/tools#tool-annotations
  - https://modelcontextprotocol.io/specification/2025-11-25/server/utilities/elicitation

source_decisions:
  - "DEC-1040 2026-05-17 — MCP 2025-11-25 tool annotations include title, readOnlyHint, destructiveHint, idempotentHint, openWorldHint"
  - DEC-1041 2026-05-17 — Tools with destructiveHint=true MUST require explicit user confirmation (via TASK-MCP-008 elicitation) BEFORE the gateway forwards tools/call
  - DEC-1042 2026-05-17 — Tools with readOnlyHint=true AND destructiveHint=false are FAST-PATH allowed (no confirmation)
  - DEC-1052 2026-05-17 — Gating is enforced AT THE MCP GATEWAY ENTRY before tools/call dispatches to the per-module server
  - DEC-1053 2026-05-17 — Annotation precedence: destructiveHint=true requires confirmation; readOnly fast-path; idempotentHint informational only
  - DEC-1055 2026-05-17 — Elicit-mode confirmation delegates to TASK-MCP-008 (as-built: confirmation elicitation holds the call)

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/gating.rs
modified_files:
  - services/mcp-gateway/src/annotations.rs
  - services/mcp-gateway/src/router.rs
  - services/mcp-gateway/src/lib.rs
  - services/mcp-gateway/src/elicitation.rs
  - services/mcp-gateway/src/elicitation_pg.rs

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_write: services/mcp-gateway/src/**
  - bash: cd services && cargo test -p cyberos-mcp-gateway gating

disallowed_tools:
  - dispatch a destructive tools/call without a gating decision
  - claim the deferred 7-file gating/ tree or gating_* integration test filenames as shipped
  - silently forward when the DB store-of-record path cannot resolve caller ids for a destructive call

effort_hours: 4
subtasks:
  - "0.5h: annotations.rs ToolAnnotations + constructors (shared with TASK-MCP-001)"
  - "1.0h: gating.rs evaluate / held_result / user_rejected_result + unit tests"
  - "1.5h: router.rs tools/call wire-up (in-memory + elicitation_pg confirmation)"
  - "1.0h: router integration tests for hold / confirm / decline / read-only fast path"

risk_if_skipped: "Without gateway-side destructive gating, tools/call forwards irreversible actions on hallucinated AI calls with no user intent check. MCP clients MAY prompt; they are not required to. The as-built hold-via-elicitation path is the server-side safety net for destructiveHint tools."
---

# TASK-MCP-006: MCP destructive-tool gating via elicitation confirmation

## Summary

Ship gateway-entry gating so a `destructiveHint=true` `tools/call` is held for a TASK-MCP-008 confirmation elicitation before the gateway forwards to the module. As-built surface is a single `gating.rs` decision module plus `ToolAnnotations` in `annotations.rs`, wired from `router.rs` (in-memory store or `elicitation_pg` when a DB + authenticated caller are present). Non-destructive tools fast-path through; decline returns an in-band `user_rejected` tool error.

## Problem

MCP 2025-11-25 advertises tool annotations, but without a gateway gate a destructive tool still executes on the first `tools/call`. Client UX prompts are optional; the server must refuse to forward until the caller confirms. The original engineering-spec imagined a 7-file `gating/` tree, per-tenant YAML policy, bypass tokens, confirm-TTL tables, and ten `gating_*` integration test files under `services/mcp/` — none of that matches the shipped `services/mcp-gateway/` crate.

## Proposed Solution

Keep the decision pure in `services/mcp-gateway/src/gating.rs`:

- `evaluate(destructive, confirmed) -> ConfirmationOutcome::{Proceed, Declined, NeedsConfirmation}`
- `held_result` / `held_result_parts` — non-error `tools/call` result carrying `elicitation_required` + confirmation schema
- `user_rejected_result` — in-band error when the caller declines
- `confirmation_prompt(tool_name)` — fixed prompt shape for the elicitation

`router.rs` consults `entry.annotations.destructive_hint` on `tools/call`. On hold it creates a confirmation elicitation (`ElicitationStore` or `elicitation_pg::create_confirmation`), returns the held result, and on re-invoke with `_meta.confirmation_id` re-evaluates and either forwards or aborts. `annotations.rs` exposes `ToolAnnotations` (including `read_only_idempotent` / `destructive` constructors) for `tools/list` and the gate.

## Alternatives Considered

- **Full per-tenant YAML policy + confirm-TTL ack table + bypass scope.** Rejected for this adopt: not shipped; ledgered under Out of scope. Confirmation via TASK-MCP-008 elicitation is the as-built confirm path.
- **Gate on `openWorldHint` as well as `destructiveHint` (DEC-1041 original wording).** Deferred: router today checks only `destructive_hint`. Documented as Out of scope rather than claimed done.
- **Separate `services/mcp/src/gating/{decision,policy,confirm,elicit,bypass,drift_detector}.rs` tree.** Rejected: as-built is one `gating.rs` module inside `mcp-gateway`.
- **Fail-open when DB path lacks caller ids.** Rejected: as-built fails closed (never forwards a destructive call without a resolvable confirmation owner).

## Success Metrics

- Primary: every `destructiveHint` `tools/call` without a valid confirmation is held with `elicitation_required`; confirmed calls forward; declined calls return `user_rejected`; read-only tools never hold.
- Guardrail: `cargo test -p cyberos-mcp-gateway` stays green for gating + annotations + router destructive-gate tests; no claim of deferred policy/bypass/drift surfaces.

## Scope

In scope (as-built under `services/mcp-gateway/`):

- `src/gating.rs` — evaluate / held_result / user_rejected_result + unit tests
- `src/annotations.rs` — `ToolAnnotations` + constructors used by the gate
- `src/router.rs` — destructive hold / confirm / decline wire-up with TASK-MCP-008 (in-memory + `elicitation_pg` when DB present)
- Router tests covering read-only fast path and the destructive confirmation cycle

### Out of scope / Non-Goals

- Per-tenant `mcp_gating_policy` YAML / admin CRUD / NATS hot-reload
- Bypass token (`mcp_gating_bypass`) and audit-only shadow mode
- Separate `mcp_pending_confirmations` confirm-TTL table (as-built uses elicitation rows)
- `openWorldHint` gating (only `destructive_hint` is checked today)
- Annotation drift detector / nightly job
- Confirmation rate limit
- Memory audit-sampling story for `mcp.tool_gating_*` kinds
- Separate 7-file `gating/` module tree under `services/mcp/`
- Claimed `gating_*` integration test filenames that do not exist (cite real in-crate / router tests instead)

## Dependencies

`depends_on: [TASK-MCP-001, TASK-MCP-004]` — protocol `tools/call` path and OAuth-authenticated callers for the DB store-of-record path. Soft: **TASK-MCP-008** supplies confirmation elicitation (in-memory + `elicitation_pg` + migration `0016`). Related: TASK-MCP-007 (future long-running destructive confirm-at-start is not required for this adopt).

## 1. Description (normative)

- 1.1 `services/mcp-gateway/src/gating.rs` MUST expose `evaluate(destructive: bool, confirmed: Option<bool>) -> ConfirmationOutcome` where non-destructive always `Proceed`, destructive+`None` is `NeedsConfirmation`, destructive+`Some(true)` is `Proceed`, and destructive+`Some(false)` is `Declined`.
- 1.2 On `NeedsConfirmation`, the gateway MUST return a non-error `tools/call` result whose structured content sets `elicitation_required: true` and carries a confirmation-typed elicitation (via `held_result` / `held_result_parts`), and MUST NOT forward to the module.
- 1.3 On `Declined`, the gateway MUST return `user_rejected_result()` (`is_error: true`, `user_rejected: true`) and MUST NOT forward.
- 1.4 Tools with `destructive_hint=false` MUST fast-path through the gate without creating an elicitation.
- 1.5 When the DB store-of-record is active, confirmation create/lookup MUST go through `elicitation_pg` scoped to the authenticated caller; if caller ids cannot be resolved for a destructive call, the gateway MUST fail closed (never forward).
- 1.6 `ToolAnnotations` MUST serialize MCP hint names (`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`) and expose the `destructive` / `read_only_idempotent` constructors used by registration and tests.
- 1.7 This adopt MUST NOT claim deferred surfaces in Out of scope as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `evaluate` truth table matches Proceed / NeedsConfirmation / Declined - test: `services/mcp-gateway/src/gating.rs::non_destructive_always_proceeds`
- [ ] AC 2 (traces_to: #1.1) - destructive holds then proceeds or declines per confirmation - test: `services/mcp-gateway/src/gating.rs::destructive_holds_then_proceeds_or_declines`
- [ ] AC 3 (traces_to: #1.2) - held result carries elicitation_required and is not an error - test: `services/mcp-gateway/src/gating.rs::held_result_carries_the_elicitation_and_is_not_an_error`
- [ ] AC 4 (traces_to: #1.3) - user_rejected result is an in-band error - test: `services/mcp-gateway/src/gating.rs::user_rejected_result_is_an_in_band_error`
- [ ] AC 5 (traces_to: #1.6) - annotation constructors flip the correct hints - test: `services/mcp-gateway/src/annotations.rs::destructive_flips_correct_hints`
- [ ] AC 6 (traces_to: #1.4) - read-only tool forwards through the gate - test: `services/mcp-gateway/src/router.rs::read_only_tool_forwards_through_the_gate`
- [ ] AC 7 (traces_to: #1.2) - destructive tool without confirmation is held - test: `services/mcp-gateway/src/router.rs::destructive_tool_without_confirmation_is_held`
- [ ] AC 8 (traces_to: #1.1,#1.5) - destructive tool with confirmation forwards - test: `services/mcp-gateway/src/router.rs::destructive_tool_with_confirmation_forwards`
- [ ] AC 9 (traces_to: #1.3) - declined confirmation aborts cleanly - test: `services/mcp-gateway/src/router.rs::destructive_tool_declined_aborts_cleanly`
- [ ] AC 10 (traces_to: #1.7) - Out of scope lists deferred policy/bypass/drift/openWorld/phantom gating_* tests; no AC claims them shipped - verify: `docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating/spec.md` Scope → Out of scope

## Verification

Run from repo root / `services/`:

```bash
cd services && cargo test -p cyberos-mcp-gateway gating
cd services && cargo test -p cyberos-mcp-gateway annotations
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::destructive_
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::read_only_tool_forwards_through_the_gate
```

Real in-crate tests (do **not** cite non-existent `gating_*` integration files):

| Path | Covers |
|------|--------|
| `services/mcp-gateway/src/gating.rs` (`non_destructive_always_proceeds`, `destructive_holds_then_proceeds_or_declines`, `held_result_carries_the_elicitation_and_is_not_an_error`, `user_rejected_result_is_an_in_band_error`) | Pure gate decision + result shapes |
| `services/mcp-gateway/src/annotations.rs` (`read_only_idempotent_flips_correct_hints`, `destructive_flips_correct_hints`) | Hint constructors |
| `services/mcp-gateway/src/router.rs` (`read_only_tool_forwards_through_the_gate`, `destructive_tool_without_confirmation_is_held`, `destructive_tool_with_confirmation_forwards`, `destructive_tool_declined_aborts_cleanly`) | End-to-end tools/call gating with in-memory elicitation |

Persistence of confirmations is verified under TASK-MCP-008 (`elicitation_pg` + `db_slice_test.rs::elicitation_persists_seals_and_is_caller_scoped`, Postgres-gated / `#[ignore]` until a pool is available).

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9a-mcp`, rewriting the engineering-spec body into task@1 grammar against as-built `services/mcp-gateway/` sources.
- **Scope:** Spec re-scoped to shipped gating surface; deferred policy/bypass/drift/openWorld/audit-sampling ledgered under Out of scope; paths corrected from `services/mcp/` to `services/mcp-gateway/`; ACs cite real in-crate tests only.
- **Human review:** Required at the two HITL gates; this file is the batch/9a-mcp adopt rework (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9a-mcp adopt — TASK-MCP-006 re-spec against as-built mcp-gateway gating.*
