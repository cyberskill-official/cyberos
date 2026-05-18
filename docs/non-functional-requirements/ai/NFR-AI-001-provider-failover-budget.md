---
id: NFR-AI-001
title: "AI Gateway provider failover latency budget — p95 < 5s from primary error to secondary first chunk"
module: AI
category: reliability
priority: MUST
verification: T
phase: P0
slo: "p95 < 5s from primary upstream error to first byte from secondary provider"
owner: CTO
created: 2026-05-18
related_frs: [FR-AI-008, FR-AI-009, FR-AI-010]
---

## §1 — Statement (BCP-14 normative)

1. The AI Gateway **MUST** route to a secondary provider when the primary provider returns a connection error, HTTP 5xx, or fails to emit a first byte within the primary timeout budget (default 8s; configurable per route).
2. The failover transition (primary error observed → secondary first byte received by gateway) **MUST** complete at **p95 < 5s** and **p99 < 8s** measured at the gateway egress span, over a 14-day rolling window.
3. The gateway **MUST** preserve the inbound caller's streaming SSE connection across failover — no client-visible reconnect. Already-emitted tokens from a failed primary attempt are dropped; the secondary stream restarts the completion from zero tokens.
4. The gateway **MUST NOT** retry against the same provider more than once for the same caller request (avoid cost amplification on persistent provider failure).
5. The gateway **MUST** emit a structured log row at provider switch carrying `{tenant_id, route, primary_provider, primary_error_class, secondary_provider, switch_latency_ms}` — consumed by OBS for the failover dashboard.

## §2 — Why this constraint

Provider outages are routine (Anthropic, OpenAI, Mistral each have monthly incidents). Without bounded failover, a caller experiences the full primary timeout (8s) **plus** the secondary completion (~3-5s) — degrading p95 latency past the 10s threshold where the host shell shows a timeout error. Bounded failover is the difference between "the LLM is occasionally slow" and "the LLM is broken." Also: the 5s budget is the contractual ceiling for the platform's CUO text-answer SLO (`NFR catalog · CUO text answer ≤ 2s`) — if failover blows past 5s, the CUO budget breaks.

## §3 — Measurement

Histogram metric `ai_gateway_failover_latency_seconds{primary_provider, secondary_provider, route}` emitted by `services/ai-gateway/src/router.rs` on every provider switch. Recorded as the wall-clock delta from the `tracing` span `provider_request.error` event to the next `provider_request.first_byte` event on the same caller-request trace. p95 alarm at > 5s; p99 alarm at > 8s. Both alarms fire to `#sev-2-ai-gateway` Slack and the alertmanager route for `ai-gateway-failover`.

## §4 — Verification

- Chaos-test `services/ai-gateway/tests/failover_latency_test.rs` (T) drives 100 synthetic requests against a primary stubbed to return HTTP 503 immediately; asserts p95 of `failover_latency_seconds` < 5.0.
- CI gate fails if the p95 from the synthetic run regresses past 5s. Trend monitored in the OBS `ai-gateway-slo` dashboard.

## §5 — Failure handling

- p95 > 5s for 10 minutes → sev-2 alert, on-call investigates whether secondary provider has degraded.
- p99 > 8s for 5 minutes → sev-1 alert, on-call considers manually promoting tertiary provider via `cyberos-ai promote-provider <tertiary>` (FR-AI-021 CLI).
- Two consecutive failovers > 8s on the same secondary → automatic circuit-break of secondary; tertiary becomes new secondary (FR-AI-009 circuit breaker FSM).

---

*End of NFR-AI-001.*
