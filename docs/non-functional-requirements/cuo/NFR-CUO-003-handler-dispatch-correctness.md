---
id: NFR-CUO-003
title: "CUO handler dispatch correctness — workflow pattern maps to single handler"
module: CUO
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of workflows dispatch to exactly one handler; 0 fallback-to-default in production"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-106, FR-CUO-104]
---

## §1 — Statement (BCP-14 normative)

1. Every workflow declaring a `pattern:` in its frontmatter **MUST** dispatch to exactly one Handler subclass (`LinearHandler | TimeCriticalHandler | PerInstanceHandler | MultiOutputHandler | SequentialApprovalHandler | PersonaPairHandler`) — there is no "default" fallback.
2. Workflows missing `pattern:` **MUST** be rejected by catalog validation at scan time; running such workflows is impossible by construction.
3. Pattern → handler mapping **MUST** be declared in a single canonical table (`modules/cuo/cuo/supervisor/dispatcher.py`); the table is the contract.
4. New patterns **MUST NOT** be added at runtime — adding a pattern requires editing the dispatch table + adding the Handler class + extending the test matrix.
5. The handler dispatch decision **MUST** be auditable in the per-chain memory row (`pattern` + `handler_class` fields).

## §2 — Why this constraint

Phase-4 supervisor uses pattern-based dispatch to invoke the right execution semantics (time-critical, per-instance, multi-output, etc.). A workflow that fell back to a default handler would silently get wrong semantics — e.g., a time-critical workflow executed with linear semantics misses its SLA. Catalog-validation gate + closed dispatch table ensures every workflow has exactly one declared correct handler. The auditability rule means we can prove which handler ran each chain — important for incident review.

## §3 — Measurement

- CI metric `cuo_workflows_missing_pattern_count` — must be 0.
- Counter `cuo_handler_dispatch_total{handler_class, pattern}` — surfaces unbalanced workloads.
- Counter `cuo_handler_dispatch_default_total` — must always be 0 in production.

## §4 — Verification

- CI gate (T) — walk all workflows; assert every one has a `pattern:` field that maps to a known handler.
- Unit test per handler (T) — execute a workflow with that pattern; assert correct handler class instantiated.
- Smoke test `modules/cuo/tests/test_phase4_dispatch.py` (T) — 9 dispatch tests pass on every CI run.

## §5 — Failure handling

- Workflow without pattern → catalog scan fails; CI blocks merge.
- Default-handler counter > 0 in production → sev-2; means catalog validation gate is broken; investigate.
- Handler class instantiation error → sev-1; chain cannot proceed; full audit of the workflow + handler table.

---

*End of NFR-CUO-003.*
