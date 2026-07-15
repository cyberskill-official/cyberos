---
id: NFR-KB-007
title: "KB render cache TTL — rendered HTML MUST refresh within 60s of doc save"
module: KB
category: performance
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 60s from doc save to fresh rendered HTML served to all readers"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-KB-002]
---

## §1 — Statement (BCP-14 normative)

1. The KB server-side renderer (`TASK-KB-002`) **MUST** cache rendered HTML keyed by `(doc_id, doc_version, theme)` for fast serving.
2. On doc save, the cache for the prior version **MUST** be marked stale within 60s; new readers receive the freshly-rendered version.
3. The cache **MUST** support content-addressable invalidation — same content + same version produces the same cache key regardless of when rendered.
4. Cache size **MUST** be bounded with LRU eviction; eviction is logged but not alarmed.
5. Cache hit rate **MUST** be > 90% for typical read patterns.

## §2 — Why this constraint

Server-side rendering is the cost-control for high-read KB pages — avoiding markdown-to-HTML conversion on every read. 60s freshness is the perception threshold for "the page updates after my edit." Content-addressable keying simplifies cache invalidation across replicas. The hit rate gauge confirms the cache is actually helping.

## §3 — Measurement

- Histogram `kb_render_cache_stale_after_save_seconds`.
- Counter `kb_render_cache_hit_total` and `kb_render_cache_miss_total`.
- Gauge `kb_render_cache_hit_ratio`.

## §4 — Verification

- Integration test (T) — save doc; assert new version served within 60s.
- Load test (T) — 1000 reads of a popular doc; assert hit rate > 90%.

## §5 — Failure handling

- Stale > 60s → sev-3; investigate invalidation propagation.
- Hit rate < 90% → sev-3; cache size or eviction policy.
- Cache corruption → sev-2; purge + investigate.

---

*End of NFR-KB-007.*
