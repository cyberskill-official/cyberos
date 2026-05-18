---
id: NFR-KB-004
title: "KB lexical search recall — FTS5 + pgroonga MUST cover known-keyword queries at ≥ 99%"
module: KB
category: usability
priority: MUST
verification: T
phase: P0
slo: "Recall ≥ 99% for known-keyword queries; p95 latency < 200ms"
owner: CTO
created: 2026-05-18
related_frs: [FR-KB-004]
---

## §1 — Statement (BCP-14 normative)

1. Lexical search (FTS5 for SQLite tier; pgroonga for Postgres tier) **MUST** find documents containing exact keyword matches at recall ≥ 99% — known-keyword queries should not silently miss documents.
2. Latency budget: p95 < 200ms end-to-end (lexical is the "cheap, exact" tier).
3. Vietnamese tokenisation **MUST** be supported (pgroonga's vietnam-specific tokenizer); query terms in Vietnamese diacritics + non-diacritic forms must both match.
4. The lexical index **MUST** be kept in sync with document state; doc save triggers async index update within 5s.
5. Stale index detection runs hourly; drift > 100 docs triggers sev-3.

## §2 — Why this constraint

Lexical search is the platform's "exact" search tier — when the user types a known phrase, they expect a hit. 99% recall is the trust floor; anything lower undermines the search UI. The Vietnamese tokenisation matters because the platform is VN-first. The 5s indexing latency ensures search feels fresh after a save.

## §3 — Measurement

- Per-quarter benchmark: recall on known-keyword test set.
- Histogram `kb_lexical_search_latency_ms`.
- Gauge `kb_lexical_index_lag_seconds`.

## §4 — Verification

- Integration test (T) — known-keyword queries; assert recall ≥ 99%.
- VN-specific test (T) — diacritic vs non-diacritic forms; assert both hit.
- Sync test (T) — save doc; assert searchable within 5s.

## §5 — Failure handling

- Recall < 99% → sev-3; investigate tokenizer or index.
- Index lag > 5s p95 → sev-3; check async indexer.
- Drift > 100 docs → sev-3; manual reindex.

---

*End of NFR-KB-004.*
