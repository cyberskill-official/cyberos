---
id: NFR-CUO-008
title: "CUO route decision latency — natural-language → persona+workflow < 1.5s"
module: CUO
category: performance
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 1.5s for cyberos-cuo route <natural-language>"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-101]
---

## §1 — Statement (BCP-14 normative)

1. The CUO router (`cyberos-cuo route "<query>"`) **MUST** return a persona+workflow decision at **p95 < 1.5s** and **p99 < 3s** for the default Phase-1 two-stage routing algorithm (persona match → workflow match within persona).
2. The router **MUST** return a structured decision: `{persona, workflow, confidence, fallback_used}`. Even on low confidence, a decision is returned with `fallback_used: domain-language`.
3. When LLM-backed routing is enabled (Phase-3 `LLMInvoker`), the p95 budget loosens to 5s.
4. Router **MUST NOT** call the executor — it answers "what would run?" without running anything.
5. Routing decisions **MUST** be logged (not necessarily emitted to BRAIN) for offline analysis of router-quality drift.

## §2 — Why this constraint

The router is the operator-facing front door. 1.5s is the threshold above which iterating on a query feels laggy. The fallback semantics ensure the router always returns SOMETHING — it doesn't punt to "I can't decide", because that's the worst user experience. The decoupling from executor is the architectural correctness rule: routing and execution have different cost profiles and different audit semantics; conflating them would force expensive execution into the routing latency budget.

## §3 — Measurement

- Histogram `cuo_route_latency_seconds{stage=persona_match|workflow_match|llm, invoker}`.
- Counter `cuo_route_fallback_used_total{kind=domain-language|low-confidence}`.
- Quarterly: sample 100 routed queries; manual classification of decision quality.

## §4 — Verification

- Benchmark `modules/cuo/tests/test_route_perf.py` (T) — 100 synthetic queries; assert p95 < 1.5s.
- Smoke test `modules/cuo/tests/test_route_smoke.py` (T) — 20 fixed queries with expected decisions.
- LLM-invoker variant: separate budget, separate benchmark.

## §5 — Failure handling

- p95 > 1.5s → sev-3; investigate slow sub-stage.
- Fallback rate > 30% → sev-3; catalog may be missing personas/workflows for common query patterns.
- Routing decision quality drops (manual sampling) → sev-3; CTO + product brief on whether routing needs LLM upgrade.

---

*End of NFR-CUO-008.*
