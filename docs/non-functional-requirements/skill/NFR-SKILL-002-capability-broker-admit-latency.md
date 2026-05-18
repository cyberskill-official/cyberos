---
id: NFR-SKILL-002
title: "SKILL capability broker admit latency — grant decision p95 < 25ms"
module: SKILL
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 25ms for capability admit (token check + policy lookup + decision)"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-104, FR-SKILL-101]
---

## §1 — Statement (BCP-14 normative)

1. The SKILL capability broker **MUST** answer admit requests (skill requests cap X for tenant Y) at **p95 < 25ms**, **p99 < 75ms**, measured at the broker's `/admit` ingress over a 14-day rolling window.
2. The hot path **MUST NOT** emit a database call — policy decisions are served from an in-memory `Arc<RwLock<PolicyMatrix>>` refreshed every 30s by a background task.
3. On policy cache miss (capability or tenant not yet loaded) the broker **MUST** fall back to a DB lookup but the resulting record **MUST** be cached in-process for the configured TTL.
4. Decision results (admit/deny + reason code) **MUST** be emitted to the audit log asynchronously — the admit response itself **MUST NOT** wait for log durability.
5. Audit log loss **MUST NOT** exceed 1 row per 1M decisions (best-effort durability tier; full BRAIN sync via FR-SKILL-101 batches separately).

## §2 — Why this constraint

The broker sits on the inner loop of every skill invocation. A 100ms broker round-trip adds 100ms × N skills to every CUO workflow — turning the system from "instant" to "stuttering." The 25ms ceiling preserves the perception that capability gating is free. The async audit decoupling is the explicit trade-off: we accept 1-row-per-million audit loss to avoid coupling the hot path to log durability — full reconciliation is handled by BRAIN's Layer-1→Layer-2 ingest with its own (looser) latency budget.

## §3 — Measurement

- Histogram `skill_broker_admit_latency_seconds{decision, capability}` p50/p95/p99.
- Counter `skill_broker_cache_miss_total{capability}` — drives capacity decisions on cache size + refresh cadence.
- Counter `skill_broker_audit_drop_total` — drift indicator for §1 #5 budget.

## §4 — Verification

- Criterion benchmark `modules/skill/benches/broker_admit.rs` (T) — 100k synthetic admits; asserts p99 < 75ms.
- Integration test (T) — drives 1000 admits with a 10% miss rate; asserts overall p95 < 25ms.
- Chaos test (T) — kills the policy refresher mid-run; asserts decisions still served from stale cache (graceful degradation).

## §5 — Failure handling

- p95 > 25ms for 10 min → sev-3; inspect cache hit rate.
- p99 > 200ms → sev-2; broker has lost in-memory cache or background refresher is dead.
- Audit drop rate > budget for 1 hr → sev-2; durability tier broken, ingest fanout pipeline needs investigation.

---

*End of NFR-SKILL-002.*
