---
id: NFR-CHAT-005
title: "CHAT VN-search recall floor — hybrid lexical + diacritic-folded recall@10 ≥ 0.90"
module: CHAT
category: functional_suitability
priority: MUST
verification: T
phase: P0
slo: "Hybrid Vietnamese search (lexical + diacritic-folded + vector) recall@10 ≥ 0.90 on VN test corpus"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CHAT-004]
---

## §1 — Statement (BCP-14 normative)

1. The CHAT search endpoint **MUST** support Vietnamese diacritic-folding: a query "Nguyen" **MUST** match messages containing "Nguyễn". Conversely, a query with full diacritics **MUST** match diacritic-less messages.
2. The hybrid search (lexical + diacritic-folded + vector) **MUST** achieve recall@10 ≥ 0.90 on the curated CHAT-VN test corpus (`services/chat-plugins/vn-search/tests/fixtures/vn_corpus_v*.jsonl`).
3. The test corpus **MUST** carry ≥ 150 (query, expected_message_ids) pairs across Vietnamese-only and mixed Vi/En messages.
4. Diacritic folding **MUST** be performed at index-time AND query-time (both sides folded for the lexical match); the diacritic-full form is preserved for display.
5. Folding **MUST** follow the standard Vietnamese folding table (`á→a, à→a, ả→a, ã→a, ạ→a, ă→a, …`) and **MUST NOT** fold Vietnamese letters with no diacritic-less equivalent (e.g., `đ` stays `đ`, or maps to `d` per the configured policy — must be deterministic).

## §2 — Why this constraint

Vietnamese users routinely type without diacritics (faster on most keyboards). Without diacritic-folding, search misses 60-80% of legitimate matches and users perceive CHAT search as broken. The 0.90 recall floor is the threshold for "search just works for Vi" — below that, customers complain. The deterministic folding rule prevents the cross-index inconsistency mode where one tenant's `đ→d` and another's `đ→đ` produce different match sets.

## §3 — Measurement

- Recall@10 reported quarterly to `docs/audits/chat-vn-search-recall/YYYY-Q*.json`.
- CI gate fails on drop below 0.90.

## §4 — Verification

- Recall test `services/chat-plugins/vn-search/tests/recall_test.rs` (T) — runs the corpus; asserts threshold.
- Quarterly review (A) — Vietnamese-native engineer reviews the corpus; new examples added per quarter.

## §5 — Failure handling

- Recall < 0.90 → block release; investigate folding table drift or ranker tuning.
- Customer reports missed VN search → add the (query, expected_message) to next corpus.
- Diacritic-fold inconsistency between tenants → sev-2; reseed the folding policy uniformly.

---

*End of NFR-CHAT-005.*
