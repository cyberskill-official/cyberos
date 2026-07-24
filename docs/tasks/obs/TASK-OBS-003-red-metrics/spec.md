---
id: TASK-OBS-003
title: "RED metrics via cyberos-obs-sdk (axum middleware + cardinality guard)"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: obs
priority: p0
status: done
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OBS-001, TASK-OBS-002, TASK-OBS-007, TASK-AI-022]
depends_on: [TASK-OBS-001]
blocks: [TASK-OBS-007, TASK-OBS-005]

source_pages:
  - website/docs/modules/obs.html#red-metrics

source_decisions:
  - DEC-150 (RED metrics across all services; SLO measurement primitive)
  - DEC-151 (shared cyberos-obs-sdk crate; consistency over per-service flexibility)
  - DEC-152 (status_class labels 2xx/3xx/4xx/5xx; raw status would explode cardinality)
  - DEC-153 (standardised histogram buckets across services; cross-service aggregation depends on it)

language: rust 1.81
service: cyberos/services/shared/cyberos-obs-sdk/
new_files:
  - services/shared/cyberos-obs-sdk/Cargo.toml
  - services/shared/cyberos-obs-sdk/src/lib.rs
  - services/shared/cyberos-obs-sdk/src/red.rs
  - services/shared/cyberos-obs-sdk/src/layer.rs
  - services/shared/cyberos-obs-sdk/src/cardinality_guard.rs

modified_files:
  - services/Cargo.toml
  - services/auth/Cargo.toml
  - services/auth/src/main.rs
  - services/auth/src/handlers.rs
  - services/auth/src/middleware.rs
  - services/memory/Cargo.toml
  - services/memory/src/main.rs
  - services/ai-gateway/Cargo.toml
  - services/ai-gateway/src/bin/cyberos_gateway.rs
  - services/ai-gateway/src/server/mod.rs

allowed_tools:
  - file_read: services/shared/cyberos-obs-sdk/**, services/{auth,memory,ai-gateway}/**
  - file_write: services/shared/cyberos-obs-sdk/**
  - file_write: services/{auth,memory,ai-gateway}/**
  - bash: cd services && cargo test -p cyberos-obs-sdk

disallowed_tools:
  - emit RED without tenant_id label (per DEC-152 / TASK-OBS-002 tenant filtering)
  - use raw HTTP status as a metric label (per DEC-152 — cardinality explosion)
  - hand-roll per-service Prometheus counters (per DEC-151 — use the shared crate)
  - "claim crates/cyberos-obs-sdk or proc-macro red_instrument as shipped"

effort_hours: 8
subtasks:
  - "1.0h: red.rs record_request + status_class + 13 standard buckets"
  - "1.0h: cardinality_guard.rs 1000-combo cap + inline tests"
  - "1.0h: layer.rs axum red_mw middleware + TenantCtx"
  - "1.0h: wire auth + memory + ai-gateway (one .layer per service)"
  - "0.5h: init() OTLP meter provider when OBS_OTLP_ENDPOINT set"
  - "0.5h: batch/9b-obs re-spec + audit"

risk_if_skipped: "TASK-OBS-007 (auto-runbook router) has no signal to trigger on. SLO compliance cannot be measured. Without standardised buckets, cross-service p95 aggregation is impossible. Without the cardinality guard, label explosion takes down Prometheus."
---

# TASK-OBS-003: RED metrics via cyberos-obs-sdk

## Summary

Per-service RED (rate / errors / duration) metrics ship from `services/shared/cyberos-obs-sdk/`: `red::record_request` with `status_class` labels (DEC-152), thirteen standard histogram buckets (DEC-153), a 1000-series cardinality guard, and axum middleware (`red_mw` / `RedState` / `TenantCtx`) in `layer.rs` — **not** a `#[red_instrument]` proc-macro. Wired into **auth**, **memory**, and **ai-gateway** HTTP stacks; chat is not instrumented here (pinned image, no src). OTLP export is configured at service boot via `init()` when `OBS_OTLP_ENDPOINT` is set. Live end-to-end validation against a running TASK-OBS-001 collector stack remains a follow-on wiring step.

## Problem

The original engineering-spec claimed `crates/cyberos-obs-sdk/`, a `macros.rs` proc-macro, an AST `instrument_completeness_test`, chat-service wiring, and standalone integration test files — none match the as-built tree. The body used `## §N` grammar (FM-004). The live crate lives under `services/shared/cyberos-obs-sdk/` with axum middleware (ADR-OBS-003-001) and inline module tests instead of separate `tests/*.rs` files.

## Proposed Solution

Adopt the as-built layout:

- `red.rs` — `record_request`, `status_class`, `error_class`, `HISTOGRAM_BUCKETS_MS` (13 boundaries), `init()` installing OTel instruments
- `cardinality_guard.rs` — per-service/per-metric 1000-combo budget with idempotent re-seen combos
- `layer.rs` — `red_mw` axum middleware; handlers supply `TenantCtx` via response extensions
- Service wiring: `auth` (TenantCtx from JWT claims), `memory` (x-tenant-id header), `ai-gateway` (`server/mod.rs` layer + `cyberos_gateway.rs` init)
- AWH: full `cargo test -p cyberos-obs-sdk` plus held-out `acceptance-obs-sdk-cardinality`

## Alternatives Considered

- **Resume the old engineering-spec as-is.** Rejected: FM-004 blocks re-entry; `crates/` path and macro story lie.
- **Per-handler `#[red_instrument]` proc-macro.** Rejected for slice-1: axum middleware gives structural route coverage (ADR-OBS-003-001).
- **Instrument chat in this slice.** Rejected: chat is a pinned image without Rust sources in this repo.

## Success Metrics

- Primary: inline tests prove status_class derivation, standard buckets, cardinality guard block at 1001st combo, and middleware transparency; AWH `acceptance-obs-sdk-cardinality` stays green.
- Guardrail: auth, memory, and ai-gateway each install `RedState` + `red_mw` at router build time.

## Scope

In scope (as-built):

- `services/shared/cyberos-obs-sdk/src/{red,layer,cardinality_guard}.rs` + `lib.rs`
- `init()` + OTLP meter provider wiring at service boot (env-gated)
- Axum `red_mw` middleware wired in auth, memory, ai-gateway
- Inline `#[test]` / `#[tokio::test]` modules in red.rs, cardinality_guard.rs, layer.rs
- AWH tasks `obs-sdk-rust` and `acceptance-obs-sdk-cardinality`

### Out of scope / Non-Goals

- `crates/cyberos-obs-sdk/` path or `src/macros.rs` proc-macro `#[red_instrument]`
- `tests/red_test.rs`, `tests/macro_test.rs`, `tests/instrument_completeness_test.rs` from the old spec
- Chat-service instrumentation (no Rust sources in repo)
- Live end-to-end OTLP validation against deploy/obs collector (requires TASK-OBS-001 collector process slice + env wiring)

## Dependencies

`depends_on: [TASK-OBS-001]`. Soft: TASK-OBS-002 tenant proxy expects `tenant_id` on every metric series; TASK-OBS-007 alert rules query RED metrics.

## 1. Description (normative)

- 1.1 `red::record_request` MUST emit `cyberos_requests_total`, `cyberos_errors_total` (4xx/5xx only), and `cyberos_duration_ms` labelled by `service`, `route`, `tenant_id`, and `status_class` (not raw status).
- 1.2 `HISTOGRAM_BUCKETS_MS` MUST be exactly thirteen boundaries `[1, 2.5, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000]` ms (DEC-153).
- 1.3 `cardinality_guard::check` MUST refuse the 1001st distinct label combo per service/metric and remain idempotent for already-seen combos.
- 1.4 `layer::red_mw` MUST record RED metrics per HTTP request without changing the handler response; `TenantCtx` supplies tenant_id when present.
- 1.5 `init(service, version)` MUST install OTel counters/histograms on the global meter; OTLP export is configured at service boot when `OBS_OTLP_ENDPOINT` is set.
- 1.6 auth, memory, and ai-gateway HTTP routers MUST install `RedState::new(<service>)` + `red_mw` middleware.
- 1.7 This adopt MUST NOT claim `crates/cyberos-obs-sdk`, proc-macro instrumentation, chat wiring, or live E2E OTLP stack validation as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - status_class covers every band - test: `services/shared/cyberos-obs-sdk/src/red.rs::status_class_covers_every_band`
- [ ] AC 2 (traces_to: #1.1) - client vs server error_class split - test: `services/shared/cyberos-obs-sdk/src/red.rs::error_class_splits_client_and_server`
- [ ] AC 3 (traces_to: #1.2) - thirteen standard histogram buckets - test: `services/shared/cyberos-obs-sdk/src/red.rs::buckets_are_the_thirteen_standard_boundaries`
- [ ] AC 4 (traces_to: #1.3) - cardinality guard blocks 1001st combo - test: `services/shared/cyberos-obs-sdk/src/cardinality_guard.rs::blocks_past_the_budget_and_is_idempotent_under_it`
- [ ] AC 5 (traces_to: #1.3) - AWH held-out cardinality invariant - verify: `cd services && cargo test -p cyberos-obs-sdk blocks_past_the_budget_and_is_idempotent_under_it`
- [ ] AC 6 (traces_to: #1.4) - middleware leaves handler status/body unchanged - test: `services/shared/cyberos-obs-sdk/src/layer.rs::middleware_is_transparent_to_the_response`
- [ ] AC 7 (traces_to: #1.5,#1.6) - auth, memory, ai-gateway call init and install red_mw - verify: `services/auth/src/main.rs`, `services/memory/src/main.rs`, `services/ai-gateway/src/bin/cyberos_gateway.rs`, `services/ai-gateway/src/server/mod.rs`
- [ ] AC 8 (traces_to: #1.7) - Out of scope lists crates/ path, macros, chat, live E2E; new_files cite services/shared only - verify: this spec Scope / new_files

## Verification

```bash
cd services && cargo test -p cyberos-obs-sdk
cd services && cargo test -p cyberos-obs-sdk blocks_past_the_budget_and_is_idempotent_under_it
```

| Path | Covers |
|------|--------|
| `src/red.rs` inline tests | status_class, error_class, buckets, safe no-op before init |
| `src/cardinality_guard.rs` inline tests | 1000-combo cap, per-service isolation, label order |
| `src/layer.rs` inline tests | TenantCtx defaults, middleware transparency |
| `services/auth/src/handlers.rs` | auth RedState + red_mw wiring |
| `services/memory/src/main.rs` | memory RedState + red_mw wiring |
| `services/ai-gateway/src/server/mod.rs` | ai-gateway RedState + red_mw wiring |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built `services/shared/cyberos-obs-sdk/` + auth/memory/ai-gateway middleware wiring; deferred proc-macro, chat, and live OTLP E2E ledgered Out of scope.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-003 re-spec against as-built cyberos-obs-sdk middleware path.*
