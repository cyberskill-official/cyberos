---
id: NFR-OBS-006
title: "OBS compliance view scoping — per-tenant RLS-equivalent on Grafana proxy"
module: OBS
category: security
priority: MUST
verification: T
phase: P0
slo: "Tenant A user views ONLY tenant A data in Grafana; zero cross-tenant leakage under property test"
owner: CSO
created: 2026-05-18
related_frs: [FR-OBS-002, FR-OBS-008]
---

## §1 — Statement (BCP-14 normative)

1. The tenant-aware Grafana proxy **MUST** rewrite every PromQL, LogQL, and TraceQL query to inject a `tenant_id=<tenant_from_jwt>` filter before forwarding to Prometheus, Loki, or Tempo respectively (NFR-OBS-008 covers the AST coverage).
2. A tenant-scoped user **MUST** not see any data in Grafana panels, alerts, dashboards, or query results for any tenant other than their own — verified by property test over 1k random JWT pairings.
3. Cross-tenant operators (CTO, CSO with `global-observer` role) **MUST** be the only path to all-tenant views; their JWT carries `tenant_scope=global` and the proxy permits unscoped queries only for those roles.
4. Dashboard JSON **MUST NOT** contain hard-coded `tenant_id` values — all tenant filters come from the proxy injection at query time.
5. Every query the proxy rewrites **MUST** emit a structured audit log `obs.proxy.query_rewrite` carrying `{jwt_subject, original_query, rewritten_query, tenant_filter_added}`; audit retained 7 years per the compliance log retention NFR.

## §2 — Why this constraint

Cross-tenant data visibility in OBS is the same compliance breach as cross-tenant data visibility in the application — the metrics ARE the data (request counts, error rates, p95 latency all reveal tenant business). Without scoping enforcement, a customer-support engineer with read access to Grafana could see another tenant's traffic patterns. The property test is the correctness guarantee; the audit log is the detective control if the property test misses a case.

## §3 — Measurement

- Counter `obs_proxy_unscoped_query_total{role}` — should be zero for any role lacking `tenant_scope=global`.
- Sev-0 alarm on `unscoped_query_total > 0` for any non-global role.
- memory view `view kind=obs.proxy.query_rewrite` — audit-able stream.

## §4 — Verification

- Property test `tests/obs/proxy_tenant_isolation_test.rs` (T) — generates 1000 random (tenant_a_user, tenant_b_data) pairings; asserts tenant_a sees only tenant_a results.
- Pen test (A) — quarterly external pen-tester attempts tenant-A-impersonates-tenant-B via crafted dashboards or query parameters.

## §5 — Failure handling

- Any unscoped query from a non-global role → sev-0; emergency shutdown of Grafana proxy; CSO + CTO call.
- Property test fails → block merge; the breaking change must be reverted or fixed before any other change.
- Tenant reports seeing data they shouldn't → sev-1 immediate investigation; preserve the trace; consider breach notification per PDPL/GDPR if confirmed.

---

*End of NFR-OBS-006.*
