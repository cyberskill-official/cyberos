---
id: TASK-TEN-204
title: "Metering ai_tokens emit at cost_reconcile — WalQueue push from ai-gateway"
template: task@1
type: feature
module: ten
status: reviewing
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T05:49:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-004]
blocks: []
related_tasks: [TASK-TEN-004, TASK-TEN-002, TASK-TEN-203]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 4
service: services/ai-gateway
new_files:
  - services/ai-gateway/src/metering_emit.rs
  - services/ai-gateway/tests/metering_ai_tokens_emit_test.rs
modified_files:
  - services/ai-gateway/Cargo.toml
  - services/ai-gateway/src/lib.rs
  - services/ai-gateway/src/cost_reconcile.rs
source_pages:
  - docs/batches/batch-ten-inv-ready.md
  - docs/tasks/ten/TASK-TEN-004-four-axis-metering/spec.md
source_decisions:
  - DEC-700 four metering axes
  - DEC-715 idempotent (tenant, axis, idempotency_key)
  - "2026-07-26 operator: option A residual batch ten-inv-host-a; ai_tokens before api_calls"
---

# TASK-TEN-204: ai_tokens metering emit at cost_reconcile

## Summary

Add the first live metering emit path: when ai-gateway `cost_reconcile` finalises a hold with
token usage (Success or Cancelled with partial usage), push one `MeteringEvent` with
`axis = ai_tokens` onto a process-local `WalQueue` (cyberos-metering), idempotent on `hold_id`.

## Problem

TASK-TEN-004 shipped axes / recorder / WalQueue / SQL migration with no callers.
Spec §1 #7 names the cost-ledger postcall hook; as-built that hook is
`services/ai-gateway/src/cost_reconcile.rs` (not `cost_ledger.rs`). Without an emit path,
period aggregates stay empty.

## Proposed Solution

1. `cyberos-ai-gateway` depends on `cyberos-metering`.
2. Module `metering_emit.rs`: process-local `OnceLock<Mutex<WalQueue>>` (or test-injectable
   `Recorder`); `emit_ai_tokens(tenant_id, hold_id, provider, model, prompt, completion)`.
3. Call from `reconcile` on `CallOutcome::Success` and `Cancelled { partial_usage: Some(_) }`
   after usage is known; **do not** emit on ProviderError or Cancelled(None).
4. Event shape:
   - `axis: AiTokens`
   - `quantity: prompt_tokens + completion_tokens` (skip if quantity == 0)
   - `idempotency_key: hold_id.to_string()`
   - `source_service: "ai-gateway"`
   - `extra: { provider, model_alias, input_tokens, output_tokens }`
5. WAL overflow: log + metric/counter; reconcile still commits (metering must not fail the call).
6. Unit test: push Success path → queue depth 1; duplicate hold_id → still depth 1 if using
   InMemoryRecorder for the test surface, or WalQueue accepts duplicates until Pg drain
   (document: WAL may carry dupes; Pg UNIQUE is the idempotency floor — for this slice test
   InMemoryRecorder via a thin `emit_to` helper).

## Alternatives Considered

- **Auth middleware `api_calls` first.** Rejected for host-a: larger surface (AppState, every
  JWT route, overage). Operator chose ai_tokens as smallest emit.
- **Postgres insert inside reconcile TX.** Rejected: metering migration not in ai-gateway DB;
  WAL decouples; Pg drain is a later slice.
- **Block reconcile on WAL overflow.** Rejected: TEN-004 hot-path doctrine — metering outage
  must not fail AI calls.

## Success Metrics

- Primary: one Success reconcile with N tokens produces one queued/recorded AiTokens event.
- Guardrail: ProviderError reconcile does not enqueue; existing cost_reconcile tests pass.

## Scope

### In scope

- Dep + emit helper + two reconcile branches + unit tests.

### Out of scope / Non-Goals

- Auth `api_calls` middleware emit and overage 402.
- Pg `Recorder` / sqlx migrate CI for metering / background drain.
- Seats / storage snapshot jobs; period close; memory dual-write for metering.
- INV Wise host.

## Dependencies

- `TASK-TEN-004` done (library).
- ai-gateway `cost_reconcile` (TASK-AI-001 lineage).

## AI Authorship Disclosure

Generated then reviewed against as-built metering crate + cost_reconcile (2026-07-26).

## Acceptance Criteria

1. **Success emit** — reconcile Success with prompt=10 completion=5 → event quantity 15, axis ai_tokens.
2. **Cancelled partial** — Cancelled(Some(usage)) emits; Cancelled(None) does not.
3. **ProviderError** — no metering event.
4. **Idempotency key** — equals `hold_id` UUID string.
5. **extra fields** — provider, model_alias, input_tokens, output_tokens present.
6. **Non-blocking** — WalQueue overflow / emit error does not change ReconcileOutcome success.
7. **Zero tokens** — quantity 0 skipped (no invalid quantity push).
8. **Dep** — `Cargo.toml` lists `cyberos-metering`.

## Verification

```sh
cd services
cargo test -p cyberos-ai-gateway --test metering_ai_tokens_emit_test -- --test-threads=1
cargo test -p cyberos-metering -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

## Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| WAL overflow | WalError::Overflow | log; call OK | Drain / raise capacity later |
| qty out of range | validate_quantity | skip + log | Cap at provider |
| Missing hold fields | N/A (locked row) | — | — |
| Double reconcile | AlreadyFinalised | no second emit | Idempotent hold |
| Mutex poison | lock err | log; call OK | Restart process |
| Zero tokens | qty check | no emit | OK |
| Test isolation | reset_for_tests | clean queue | cfg(test) reset |
| Dep version skew | cargo | build fail | Workspace path |
| Extra JSON missing keys | test assert | fail CI | Fix emit |
| Emit after TX commit vs before | code review | prefer after successful apply, before commit OK if non-blocking | Document |

---

*End of TASK-TEN-204.*
