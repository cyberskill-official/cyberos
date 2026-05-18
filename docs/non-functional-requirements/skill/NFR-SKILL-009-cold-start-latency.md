---
id: NFR-SKILL-009
title: "SKILL cold-start latency — bundle pull + manifest parse + capability admit < 2s"
module: SKILL
category: performance
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 2s, p99 < 5s for first-invocation of a skill not in cache"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-102, FR-SKILL-104]
---

## §1 — Statement (BCP-14 normative)

1. First invocation of a skill not already in local cache **MUST** complete the cold-start steps (`registry pull → manifest parse → schema validate → capability admit → runtime ready`) at **p95 < 2s** and **p99 < 5s**.
2. Bundles **MUST** be cached locally (`skill-cache/`) after first pull; second invocation of the same skill+version **MUST NOT** hit the registry (warm path < 50ms).
3. The local cache **MUST** be persistent across runtime restarts.
4. Cache eviction **MUST** be LRU with a configurable size limit; eviction events are logged but not alarmed.
5. Cold-start latency **MUST NOT** be on the inner loop of a multi-step CUO workflow — chain replanning waits for skill ready, not for per-skill cold-start.

## §2 — Why this constraint

Cold start is the user-perceived "first time I asked for something" experience. 2s is the friction threshold above which users perceive the system as slow. The persistent cache amortises this — most production invocations are warm and sub-50ms. The CUO-chain decoupling avoids amplifying cold-start latency across a 10-step workflow: chain planning happens once, then execution proceeds against the warmed cache.

## §3 — Measurement

- Histogram `skill_cold_start_latency_seconds{step=pull|parse|admit|ready}` — surfaces which step is the bottleneck.
- Counter `skill_cold_start_total{skill, version}` — surfaces cache hit rate.
- Histogram `skill_cache_size_bytes` — capacity planning.

## §4 — Verification

- Integration test (T) — clears cache; invokes 50 distinct skills; asserts p95 < 2s.
- Warm-path test (T) — second invocation of same skill+version asserts < 50ms.
- Chaos test (T) — registry slow (500ms artificial latency); cold-start p95 must still be < 5s.

## §5 — Failure handling

- p95 > 2s for 10 min → sev-3; identify which sub-step regressed.
- p99 > 5s sustained → sev-2; likely registry pull is slow; check CDN.
- Cache miss rate > 30% → sev-3; cache size may be too small or eviction too aggressive.

---

*End of NFR-SKILL-009.*
