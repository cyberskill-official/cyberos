---
id: NFR-OBS-008
title: "Grafana proxy AST-injection coverage — PromQL + LogQL + TraceQL all rewritten with tenant_id"
module: OBS
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of PromQL, LogQL, TraceQL queries through the proxy carry an injected tenant_id filter (verified per-query-language test suite)"
owner: CSO
created: 2026-05-18
related_frs: [FR-OBS-008, FR-OBS-002]
---

## §1 — Statement (BCP-14 normative)

1. The tenant-aware Grafana proxy **MUST** parse incoming queries into the language-specific AST (PromQL, LogQL, TraceQL) — **MUST NOT** use regex-based rewriting which is bypassable via comment-injection.
2. For PromQL, the proxy **MUST** inject the `tenant_id=<jwt_tenant>` label matcher into every metric selector in the AST, including nested subqueries and `topk`/`bottomk` expressions.
3. For LogQL, the proxy **MUST** inject the same matcher into every stream selector, including in `count_over_time` and `rate` subexpressions.
4. For TraceQL, the proxy **MUST** inject `resource.service.tenant_id=<jwt_tenant>` into every span filter expression.
5. Each query language **MUST** have a dedicated test fixture set (≥ 50 queries per language) that exercises the injection rules; CI gate fails on any test miss.

## §2 — Why this constraint

Regex-based query rewriting is the historical vulnerability pattern — every observability platform that has been pwned by query injection used regex. AST-based rewriting is provably correct: the query is parsed, the filter is added at the right scope, and the result is re-serialised. The 50-query fixture per language is the load-bearing safety net; without exhaustive fixtures, edge-case query shapes (e.g., nested binary ops in PromQL) slip through. The three-language coverage matters because Grafana panels mix all three — a tenant-scoped Prometheus panel adjacent to an un-scoped Loki panel is the leak path.

## §3 — Measurement

- Counter `obs_proxy_ast_parse_failed_total{language}` — should be near-zero; high values indicate AST parser drift.
- Counter `obs_proxy_ast_rewrite_total{language}` — should equal `obs_proxy_query_total` (every query rewritten).
- Daily synthetic query smoke — 30 queries per language run with synthetic tenant scopes; verify only matching tenant rows return.

## §4 — Verification

- Fixture test `tests/obs/proxy_ast_promql_test.rs` (T) — 50+ PromQL queries; asserts rewrite output contains `tenant_id="<tenant>"` matcher in every selector.
- Same for `proxy_ast_logql_test.rs` and `proxy_ast_traceql_test.rs`.
- Fuzz test `tests/obs/proxy_ast_fuzz_test.rs` (T) — quickcheck-style random query generation; asserts injection never panics and the output is always tenant-scoped.

## §5 — Failure handling

- AST parse failure on a real-world query → sev-3; user gets HTTP 400 with "unsupported query shape"; ticket files a new fixture.
- Rewrite output missing tenant_id filter → sev-0; same response as NFR-OBS-006 cross-tenant leak.
- New Grafana version ships a new query feature → block deploy until proxy AST parser is upgraded with fixtures.

---

*End of NFR-OBS-008.*
