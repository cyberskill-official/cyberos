---
id: NFR-KB-003
title: "KB semantic search recall — bge-m3 + rerank MUST hit recall@10 ≥ 90%"
module: KB
category: usability
priority: SHOULD
verification: T
phase: P1
slo: "Recall@10 ≥ 90% on the platform's labeled query/document corpus"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-KB-005, TASK-KB-006]
---

## §1 — Statement (BCP-14 normative)

1. KB semantic search (`TASK-KB-005` bge-m3 + `TASK-KB-006` rerank) **MUST** achieve recall@10 ≥ 90% on the platform's labeled evaluation corpus (1000+ query/doc pairs).
2. Latency budget: p95 < 800ms end-to-end for `query → vector lookup → rerank → top-10 results`.
3. The eval corpus **MUST** be refreshed quarterly with new query patterns observed in production.
4. Significant regression (recall drop > 5pp) on a new index/model rollout **MUST** block the rollout.
5. Query logs **MUST NOT** include PII; only sanitised intent labels for offline eval.

## §2 — Why this constraint

Semantic search quality determines whether KB feels useful or frustrating. 90% recall@10 is the threshold below which users start preferring grep. The 800ms ceiling keeps interactive use snappy. Quarterly refresh against production queries keeps eval relevant. The rollout-gate prevents accidentally shipping a worse model.

## §3 — Measurement

- Per-quarter benchmark: recall@10, MRR, latency p95.
- Counter `kb_semantic_search_query_total{result_count_bucket}`.
- A/B harness for new model rollouts.

## §4 — Verification

- Quarterly benchmark (T) — assert recall@10 ≥ 90%.
- Smoke test (T) — 20 production queries via CI; assert reasonable top-1.
- Rollout gate — automated A/B against held-out set before promote.

## §5 — Failure handling

- Recall < 90% → sev-3; retune model or rerank.
- Regression detected → block rollout; retune or revert.
- Latency > 800ms → sev-3; investigate index size or rerank cost.

---

*End of NFR-KB-003.*
