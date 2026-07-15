---
id: NFR-OBS-002
title: "Trace continuity — W3C traceparent propagates across ≥ 2 service hops; CI test enforces"
module: OBS
category: observability
priority: MUST
verification: T
phase: P0
slo: "100% of cross-service requests carry an unbroken traceparent chain ≥ 2 hops"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-OBS-005, TASK-OBS-001]
---

## §1 — Statement (BCP-18 normative)

1. Every CyberOS backend service **MUST** propagate the W3C `traceparent` and `tracestate` HTTP headers on outbound calls (server → service → service) and on NATS subjects (via the `traceparent` message header).
2. The end-to-end trace **MUST** be visible in Tempo with at least 2 service-name spans for any caller-initiated request that touches more than one service. Single-service requests are exempt.
3. The `cyberos-obs-sdk` HTTP and NATS clients **MUST** auto-inject `traceparent` from the current active span; services **MUST NOT** open-code header propagation.
4. CI test `tests/obs/trace_continuity_test.sh` **MUST** drive a synthetic request through ≥ 2 hops, fetch the trace from Tempo, and assert the trace contains expected service names with the same `trace_id`.
5. Sampled-out traces (per NFR-OBS-003 tail sampling) still **MUST** carry the `traceparent` for the duration of the request — only the **storage** is sampled, never the **propagation**.

## §2 — Why this constraint

A trace that breaks mid-flight is worse than no trace — it lies about where work happened. Without enforced propagation, services drop the header silently and incidents become unrunbookable ("the call vanished between auth and memory"). The 2-hop minimum is the test the CI enforces; in practice production traces routinely have 5-8 hops (host shell → graphql → auth → memory → ai-gateway → upstream). The SDK auto-injection is the implementation control; the CI test is the verification control.

## §3 — Measurement

- Counter `obs_trace_propagation_gap_total{from_service, to_service}` emitted when an inbound request lacks `traceparent` but came from a known-internal source.
- Daily CI smoke run of `trace_continuity_test.sh` against staging; failure = sev-3 alert.
- Tempo query `{ trace.spans.count >= 2 && trace.first_span.service != trace.last_span.service }` should match > 95% of caller-initiated traces.

## §4 — Verification

- CI test (T) — runs in nightly OBS CI; spawns 100 synthetic requests, asserts every trace has ≥ 2 service-name spans.
- Integration test (T) — `tests/obs/trace_propagation_nats_test.rs` verifies NATS subject metadata carries the header.

## §5 — Failure handling

- Trace gap counter > 0 → sev-3; investigate which service drops the header. Most common cause: a custom HTTP client bypassing the SDK.
- CI test fails → block merge; the breaking service must restore propagation before any other change merges.
- Tempo storage hits cardinality limit on `trace_id` → sev-2 capacity issue (independent of propagation correctness).

---

*End of NFR-OBS-002.*
