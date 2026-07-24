---
id: TASK-OBS-005
title: "W3C TraceContext correlation (SDK + ai-gateway middleware + auth JWT)"
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
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OBS-001, TASK-OBS-003, TASK-OBS-004, TASK-AI-022]
depends_on: [TASK-OBS-001, TASK-OBS-003]
blocks: []

source_pages:
  - website/docs/modules/obs.html#correlation
  - W3C TraceContext spec (https://www.w3.org/TR/trace-context/)

source_decisions:
  - DEC-160 2026-05-15 — W3C TraceContext over B3/Jaeger; IETF-standard, broad support
  - DEC-161 2026-05-15 — trace_id embedded in every structured log + histogram exemplar; correlation primitive
  - DEC-162 2026-05-15 — malformed traceparent → forensic hash16, never log raw bytes

language: rust 1.81
service: cyberos/services/shared/cyberos-obs-sdk/
new_files:
  - services/shared/cyberos-obs-sdk/src/tracecontext.rs
  - services/shared/cyberos-obs-sdk/src/logging.rs
  - services/shared/cyberos-obs-sdk/src/exemplar.rs
  - services/ai-gateway/tests/otel_propagation_test.rs
modified_files:
  - services/shared/cyberos-obs-sdk/src/red.rs
  - services/ai-gateway/src/server/mod.rs
  - services/auth/src/jwt.rs

allowed_tools:
  - file_read: services/shared/cyberos-obs-sdk/**, services/ai-gateway/**, services/auth/**
  - file_write: services/shared/cyberos-obs-sdk/**, services/ai-gateway/**, services/auth/**
  - bash: cd services && cargo test -p cyberos-obs-sdk tracecontext
  - bash: cd services && cargo test -p cyberos-ai-gateway trace_ctx

disallowed_tools:
  - strip trace_id from any structured log line (per DEC-161)
  - reject requests when traceparent is missing or malformed (per DEC-160 operational semantics)
  - use B3 or Jaeger-native propagation (per DEC-160)
  - claim LangSmith AI-trace correlation or obs-correlation-gate CI as shipped (deferred to TASK-OBS-004)

effort_hours: 8
subtasks:
  - "0.5h: tracecontext.rs — parse / extract / inject / hash16 + inline tests"
  - "0.5h: logging.rs — request_span + init_json_subscriber + inline enrichment test"
  - "0.5h: exemplar.rs — record_with_exemplar + red.rs hook"
  - "0.5h: red.rs — record_tracecontext_extracted + obs_exemplar_emission_total"
  - "1.0h: ai-gateway trace_ctx middleware + server/mod.rs tests"
  - "0.5h: auth jwt.rs traceparent claim + jwt_roundtrip_test"
  - "0.5h: otel_propagation_test.rs (TASK-AI-022 propagation helpers)"
  - "1.0h: batch/9b-obs re-spec + audit"
risk_if_skipped: "Investigation requires joining Loki + Prometheus + Tempo by timestamp. Without trace_id in logs, the primary debug query `{trace_id=\"...\"}` is impossible. Without exemplars, jumping from a Grafana latency spike to the offending trace requires manual time-window narrowing."
---

# TASK-OBS-005: W3C TraceContext correlation

## Summary

Propagate W3C TraceContext across CyberOS services and embed `trace_id` into structured logs and RED histogram exemplars. As-built surface lives in `services/shared/cyberos-obs-sdk/src/{tracecontext,logging,exemplar}.rs` with `red.rs` counters (`record_tracecontext_extracted`, exemplar emission), the ai-gateway `trace_ctx` axum middleware in `services/ai-gateway/src/server/mod.rs`, and the auth JWT `traceparent` claim in `services/auth/src/jwt.rs`.

## Problem

The original engineering-spec claimed `crates/cyberos-obs-sdk/`, separate integration test crates (`tracecontext_test.rs`, `end_to_end_correlation_test.rs`), `.github/workflows/obs-correlation-gate.yml`, and trace middleware on chat/memory. The live SDK is `services/shared/cyberos-obs-sdk/`; ai-gateway ships `trace_ctx` middleware; auth carries `traceparent` in JWT claims. The process evidence failed FM-004 (task@1 frontmatter + `## §N` body), `depends_on` incorrectly included TASK-OBS-004, and phantom paths blocked re-entry.

## Proposed Solution

Adopt the as-built layout:

- `tracecontext.rs` — strict W3C version-00 parse, `extract_traceparent`, `inject_traceparent`, `hash16` for malformed values
- `logging.rs` — `request_span(trace_id, span_id, tenant_id)` + `init_json_subscriber` (span-scope JSON rendering)
- `exemplar.rs` — `record_with_exemplar` on `cyberos_duration_ms` via `red::record_request`
- `red.rs` — `record_tracecontext_extracted(outcome)` with `extracted | malformed | missing_generated_new`
- ai-gateway `trace_ctx` — extract/generate, metric emit, `request_span` instrumentation, response `traceparent` echo
- auth `jwt.rs` — optional `traceparent` claim round-trips through issue/verify

## Alternatives Considered

- **Resume the old engineering-spec as-is.** Rejected: FM-004 blocks re-entry; paths and CI gate lie.
- **Custom OTel-context tracing Layer (original §3 sketch).** Rejected: as-built uses `tracing::Span` + JSON subscriber — no OTel tracer provider required at the log boundary; verifiable with in-memory capture.
- **Block on TASK-OBS-004 LangSmith correlation before adopt.** Rejected: OBS-004 is soft-related; slice-2 correlation primitives stand alone.

## Success Metrics

- Primary: every ai-gateway request carries a valid W3C trace context; logs emitted inside the request span include `trace_id` + `tenant_id`; duration histogram records exemplars.
- Guardrail: `obs_tracecontext_extracted_total{outcome}` increments on the three middleware paths; JWT issue/verify preserves `traceparent`.

## Scope

In scope (as-built):

- `services/shared/cyberos-obs-sdk/src/{tracecontext,logging,exemplar,red}.rs`
- ai-gateway `trace_ctx` middleware + inline server tests + `tests/otel_propagation_test.rs`
- auth JWT `traceparent` claim (`jwt.rs`, `jwt_roundtrip_test.rs`)

### Out of scope / Non-Goals

- TASK-OBS-004 LangSmith / AI-trace correlation ledger (soft: TASK-AI-022 OTel propagation helpers only)
- Deleted `.github/workflows/obs-correlation-gate.yml` and `end_to_end_correlation_test.rs` (no live CI gate querying Loki + Tempo + LangSmith + Prometheus)
- chat/memory HTTP `trace_ctx` middleware (not wired in this slice; auth README notes outbound RPC propagation deferred)
- `InstrumentedClient` HTTP wrapper (outgoing inject lives in TASK-AI-022 `propagation` module for ai-gateway router backends)
- Subprocess `OTEL_TRACE_ID` env propagation and tokio-spawn `Instrument` enforcement beyond what ai-gateway already does

## Dependencies

`depends_on: [TASK-OBS-001, TASK-OBS-003]`. Soft: TASK-AI-022 (OTel propagation helpers in ai-gateway); TASK-OBS-004 (LangSmith trace_id alignment — not a hard gate for this adopt).

## 1. Description (normative)

- 1.1 `parse_w3c_traceparent` and `extract_traceparent` MUST enforce W3C version-00 strictly: `00-{32 lower-hex}-{16 lower-hex}-{2 lower-hex flags}`; reject wrong version, wrong lengths, uppercase hex, and all-zero trace/span ids (DEC-160, DEC-162).
- 1.2 At every HTTP boundary, a missing `traceparent` MUST generate a fresh trace context; a malformed header MUST generate a fresh context and MUST NOT honour attacker-supplied ids. Requests MUST NOT be rejected — TraceContext is operational, not security (DEC-160).
- 1.3 Every structured log line emitted while handling a request MUST carry `trace_id`, `span_id`, and `tenant_id` via the canonical `request_span` + a JSON subscriber that renders the span scope (`logging.rs`, DEC-161).
- 1.4 `cyberos_duration_ms` histogram samples MUST pass through `exemplar::record_with_exemplar` so the OTel→Prometheus exporter can attach trace exemplars when a trace is in context (DEC-161, TASK-OBS-003).
- 1.5 `record_tracecontext_extracted(outcome)` MUST increment `obs_tracecontext_extracted_total{outcome}` for `extracted`, `malformed`, and `missing_generated_new` at the request boundary.
- 1.6 The ai-gateway `trace_ctx` middleware MUST stamp `RequestTrace` on the request, instrument the handler future with `request_span`, and echo the resolved `traceparent` on the response.
- 1.7 Auth JWT issue/verify MUST round-trip an optional W3C `traceparent` claim so token issuance stitches to the inbound request trace (AUTHORING §3.7 rule 22).
- 1.8 This adopt MUST NOT claim OBS-004 LangSmith correlation, the deleted `obs-correlation-gate.yml`, or chat/memory HTTP middleware as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - strict W3C parse accepts valid / rejects bad shapes - test: `services/shared/cyberos-obs-sdk/src/tracecontext.rs::parses_a_valid_traceparent`
- [ ] AC 2 (traces_to: #1.1) - rejects non-00 version and wrong lengths - test: `services/shared/cyberos-obs-sdk/src/tracecontext.rs::rejects_non_zero_version`
- [ ] AC 3 (traces_to: #1.2) - extract returns Missing / Malformed(hash16) / Ok - test: `services/shared/cyberos-obs-sdk/src/tracecontext.rs::extract_missing_then_malformed_then_valid`
- [ ] AC 4 (traces_to: #1.2) - middleware generates valid traceparent when absent - test: `services/ai-gateway/src/server/mod.rs::response_carries_a_generated_traceparent_when_absent`
- [ ] AC 5 (traces_to: #1.6) - middleware echoes valid inbound traceparent - test: `services/ai-gateway/src/server/mod.rs::response_echoes_a_valid_inbound_traceparent`
- [ ] AC 6 (traces_to: #1.3) - log JSON carries trace_id and tenant_id inside request span - test: `services/shared/cyberos-obs-sdk/src/logging.rs::an_event_inside_the_request_span_carries_trace_id_and_tenant_id`
- [ ] AC 7 (traces_to: #1.4) - exemplar record is safe no-op before init - test: `services/shared/cyberos-obs-sdk/src/red.rs::record_with_exemplar_is_a_safe_noop_on_a_noop_meter`
- [ ] AC 8 (traces_to: #1.5) - tracecontext extracted counter safe before init - test: `services/shared/cyberos-obs-sdk/src/red.rs::record_tracecontext_extracted_is_a_safe_noop_before_init`
- [ ] AC 9 (traces_to: #1.7) - JWT issue then verify preserves traceparent - test: `services/auth/tests/jwt_roundtrip_test.rs`
- [ ] AC 10 (traces_to: #1.1) - hash16 is 16 hex chars, deterministic - test: `services/shared/cyberos-obs-sdk/src/tracecontext.rs::hash16_is_deterministic_16_hex`
- [ ] AC 11 (traces_to: #1.8) - Out of scope lists OBS-004 gate + chat/memory middleware - verify: this spec Scope / Out of scope

## Verification

```bash
cd services && cargo test -p cyberos-obs-sdk tracecontext
cd services && cargo test -p cyberos-obs-sdk logging
cd services && cargo test -p cyberos-obs-sdk red::
cd services && cargo test -p cyberos-ai-gateway trace_ctx
cd services && cargo test -p cyberos-ai-gateway --test otel_propagation_test
# jwt_roundtrip (ignored without Postgres):
cd services && cargo test -p cyberos-auth jwt_roundtrip -- --ignored
```

| Path | Covers |
|------|--------|
| `cyberos-obs-sdk/src/tracecontext.rs` (inline `#[cfg(test)]`) | W3C parse, extract, inject, hash16 |
| `cyberos-obs-sdk/src/logging.rs` (inline `#[cfg(test)]`) | Span-scope log enrichment |
| `cyberos-obs-sdk/src/red.rs` (inline `#[cfg(test)]`) | Exemplar + tracecontext counter no-op safety |
| `ai-gateway/src/server/mod.rs` (inline tests) | `trace_ctx` generate + echo |
| `ai-gateway/tests/otel_propagation_test.rs` | TASK-AI-022 inject/extract roundtrip |
| `auth/tests/jwt_roundtrip_test.rs` | JWT `traceparent` claim |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built `cyberos-obs-sdk` + ai-gateway `trace_ctx` + auth JWT claim; dropped TASK-OBS-004 from `depends_on`; ledgered OBS-004 / CI gate / chat-memory middleware as Out of scope.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-005 re-spec against as-built TraceContext correlation.*
