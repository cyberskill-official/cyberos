---
id: NFR-BRAIN-008
title: "BRAIN search API recall floor — lexical hybrid recall@10 ≥ 0.85 on test corpus"
module: BRAIN
category: functional_suitability
priority: MUST
verification: T
phase: P0
slo: "Hybrid lexical + vector search recall@10 ≥ 0.85 on the BRAIN search test corpus"
owner: CTO
created: 2026-05-18
related_frs: [FR-BRAIN-108, FR-AI-019, FR-AI-020]
---

## §1 — Statement (BCP-14 normative)

1. The `services/brain/src/search.rs` search API **MUST** achieve **recall@10 ≥ 0.85** on the curated BRAIN search test corpus (`services/brain/tests/fixtures/search_corpus_v*.jsonl`).
2. The search MUST be hybrid: BM25 lexical (Postgres full-text) + dense vector (pgvector, BGE-M3 embeddings per FR-AI-019), with reciprocal-rank fusion at top-50 and BGE reranker (FR-AI-020) producing the top-10.
3. The test corpus **MUST** carry ≥ 200 (query, expected_chunk_ids) pairs across English and Vietnamese, refreshed quarterly.
4. Per-language recall **MUST** be ≥ 0.80 individually — neither English-only nor Vietnamese-only may carry the average above the floor.
5. The CI gate **MUST** run the recall test on every `services/brain/src/search.rs` or `services/ai-gateway/src/embeddings/*` change; PR blocks below 0.85 overall.

## §2 — Why this constraint

Search recall is the platform's "did the CUO find the right citation" guarantee. Recall@10 of 0.85 is the threshold below which user trust in citations breaks — they start saying "the AI couldn't find a relevant doc." The hybrid lexical+vector approach is needed because pure vector search has known weak cases (exact identifier matches, proper nouns) and pure lexical fails on paraphrase. The per-language floor prevents one language masking the other's failure — a 0.95 English / 0.65 Vietnamese system shouldn't ship at 0.80 average; both must hold.

## §3 — Measurement

- Recall@10 reported quarterly to `docs/audits/brain-search-recall/YYYY-Q*.json`.
- Per-language breakdown in the same JSON.
- CI gate fails on any drop below 0.85 overall or 0.80 per-language.

## §4 — Verification

- Recall test `services/brain/tests/search_recall_test.rs` (T) — runs the corpus; asserts thresholds.
- Quarterly review (A) — CTO + CSO review the corpus + recall trend; new corpus items must be added per quarter.

## §5 — Failure handling

- Overall < 0.85 → block release; investigate whether ranker, embedder, or corpus drifted.
- Per-language < 0.80 → block release; rebalance corpus and/or retune language-specific stop-words.
- Customer reports missed citation → ticket adds the (query, expected_chunk) to next corpus refresh.

---

*End of NFR-BRAIN-008.*
