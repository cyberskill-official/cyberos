---
id: NFR-AI-002
title: "AI Gateway cost-ledger pre-call check overhead — p99 < 10ms"
module: AI
category: performance
priority: MUST
verification: T
phase: P0
slo: "p99 < 10ms; p95 < 5ms for cost-ledger pre-call admission check"
owner: CTO
created: 2026-05-18
related_frs: [FR-AI-007, FR-AI-008]
---

## §1 — Statement (BCP-14 normative)

1. Before every upstream provider call, the AI Gateway **MUST** consult the per-tenant cost-ledger to verify the projected token-cost does not exceed the tenant's monthly budget remainder.
2. This pre-call admission check **MUST** complete at **p99 < 10ms** and **p95 < 5ms** measured at the `cost_ledger.admit` span, over a 14-day rolling window.
3. The check **MUST** be performed against an in-memory aggregate (refreshed by background ticker every 30s from the durable Postgres ledger); a cache-miss falls through to the synchronous DB read but **MUST** still complete < 30ms p99.
4. The check **MUST NOT** be skipped on streaming requests — the projected cost is computed from the request's `max_tokens` upper bound, not the actual completion length.
5. On admission denial, the gateway **MUST** return HTTP 402 (Payment Required) with body `{error: "tenant_monthly_budget_exhausted", remaining_usd: 0.0, reset_at: "<ISO8601>"}` — never a 500 or 429.

## §2 — Why this constraint

The cost-ledger is on the critical path of every LLM call. If the check takes 50ms, that's 50ms of latency added to **every** completion — directly degrading user-facing latency budgets. The 10ms p99 ceiling guarantees the check is invisible inside the much larger LLM round-trip (~2-5s typical). Also: budget enforcement is a hard correctness property (CFO's revenue protection contract) — silently skipping the check would let runaway tenants drain the platform LLM budget.

## §3 — Measurement

Histogram metric `ai_gateway_cost_ledger_admit_seconds` emitted by `services/ai-gateway/src/cost/ledger.rs::admit()`. Buckets: 0.001, 0.002, 0.005, 0.01, 0.025, 0.05, 0.1. p99 alarm at > 0.010; p95 alarm at > 0.005. Two separate alarms because cache-miss fall-through is expected to occasionally hit > 10ms (acceptable) but a sustained p95 > 5ms indicates the in-memory cache itself has degraded.

## §4 — Verification

- Criterion benchmark `services/ai-gateway/benches/cost_ledger_admit.rs` (T) runs 100k admit() calls against a populated 5k-tenant cache; asserts p99 < 10ms on the CI runner.
- Integration test `services/ai-gateway/tests/cost_ledger_admit_overhead_test.rs` (T) measures end-to-end /v1/chat/completions overhead with cost-ledger enabled vs disabled; delta must be < 10ms p99.

## §5 — Failure handling

- p99 > 10ms for 5 minutes → sev-3 alert; on-call inspects cache refresh task health.
- Background refresh task fails 3 consecutive cycles → sev-2 alert (the in-memory snapshot is going stale; budget enforcement may approve over-budget requests).
- DB read timeout > 30ms on cache-miss → fail-closed (deny the request with HTTP 503, log row to memory); fail-open is never permitted on the cost path (CFO contract).

---

*End of NFR-AI-002.*
