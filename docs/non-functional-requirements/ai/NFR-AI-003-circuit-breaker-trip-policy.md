---
id: NFR-AI-003
title: "AI Gateway circuit-breaker trip policy — 3 errors in 30s opens; recovery probe at 60s"
module: AI
category: reliability
priority: MUST
verification: T
phase: P0
slo: "Open within 30s of 3rd error; first recovery probe at exactly 60s post-open"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-AI-009, TASK-AI-008]
---

## §1 — Statement (BCP-14 normative)

1. The AI Gateway circuit-breaker per (provider, route) tuple **MUST** transition from CLOSED → OPEN when 3 consecutive upstream errors occur within a 30-second sliding window. Errors counted: connection-failure, HTTP 5xx, HTTP 429 with no `Retry-After`, and provider-emitted timeouts.
2. The OPEN state **MUST** persist for exactly 60s (no jitter — deterministic for runbook predictability), after which the breaker transitions to HALF-OPEN and admits one probe request.
3. A successful HALF-OPEN probe (2xx response with valid completion) **MUST** transition the breaker back to CLOSED. A failing probe **MUST** transition back to OPEN for another 60s window.
4. While OPEN, the gateway **MUST** route all matching requests to the next-priority provider (failover per NFR-AI-001) without latency penalty; it **MUST NOT** attempt the open provider until the 60s timer elapses.
5. Every circuit transition **MUST** emit a memory audit row `ai_gateway.circuit.{open|close|probe}` carrying `{provider, route, trigger_error_class, ts, prev_state, next_state}`. These rows are immutable and survive consolidation.

## §2 — Why this constraint

Without a circuit-breaker, a hung upstream provider causes every caller to wait the full primary timeout (8s) before failover — multiplying outage impact. With a 3-error trip threshold, the breaker opens within seconds of a real provider outage; with 60s recovery, the breaker re-probes fast enough that a 30-second blip resolves naturally. The deterministic 60s window (no jitter) means runbooks can say "wait one minute and check again" — operators don't need to track per-provider exponential backoff.

## §3 — Measurement

- Gauge `ai_gateway_circuit_state{provider, route}` ∈ {0=closed, 1=half_open, 2=open}.
- Counter `ai_gateway_circuit_transitions_total{provider, route, transition}` per FSM edge.
- The 3-in-30s and 60s-recovery contract is verified by inspecting the memory audit rows: the temporal delta between `circuit.open` and the third `provider_request.error` row **MUST** be ≤ 30s; the delta between `circuit.open` and the next `circuit.probe` row **MUST** be ≥ 60s and < 61s.

## §4 — Verification

- Property test `services/ai-gateway/tests/circuit_breaker_policy_test.rs` (T) drives 1000 random error-injection sequences against the breaker FSM, asserting the trip/recovery invariants hold under all interleavings.
- Chaos drill `deploy/obs/runbooks/ai-gateway-provider-outage.md` (D) — quarterly — kills a provider mid-day and asserts the circuit opens within 30s, recovers within 90s, and no memory audit row is missing from the transition log.

## §5 — Failure handling

- Circuit flaps OPEN↔CLOSED 5 times in 10 minutes → sev-2 alert; provider has intermittent degradation, on-call manually pins the breaker OPEN via `cyberos-ai pin-circuit-open <provider> <route>` until provider recovers.
- memory audit row missing for any circuit transition → sev-1 (chain-of-custody violation); halt new requests on that route until audit catches up.

---

*End of NFR-AI-003.*
