---
id: NFR-OBS-001
title: "RED metric coverage — every service emits rate/error/duration via cyberos-obs-sdk"
module: OBS
category: observability
priority: MUST
verification: I
phase: P0
slo: "100% of services emit RED metrics; CI fails if a service lacks the cyberos-obs-sdk middleware"
owner: CTO
created: 2026-05-18
related_frs: [FR-OBS-003, FR-OBS-001]
---

## §1 — Statement (BCP-14 normative)

1. Every CyberOS backend service **MUST** emit the three RED metric families on every public route: `request_rate_total{route, method, status}`, `request_errors_total{route, method, error_class}`, `request_duration_seconds{route, method, status}` (histogram).
2. Emission **MUST** flow through the shared `cyberos-obs-sdk` crate/package — services **MUST NOT** open-code their own Prometheus collectors. The SDK provides middleware for `axum`, `actix-web`, `fastify`, and `gin`.
3. Metric labels **MUST** include the standard set: `service`, `route` (templated, not raw — e.g., `/v1/tenants/:id` not `/v1/tenants/abc-123`), `method`, `status`, `tenant_id` (when known and not high-cardinality risk).
4. The `route` label cardinality **MUST** be ≤ 200 unique values per service; the SDK enforces this with a fallback `route=other` bucket on overflow.
5. A CI gate **MUST** scan every service's `main.rs` (or equivalent) for the `cyberos_obs_sdk::install_red_middleware()` call (or framework-specific equivalent); PR is blocked if a backend service lacks the call.

## §2 — Why this constraint

RED metrics are the universal first-line observability signal — rate tells you traffic, errors tells you health, duration tells you performance. Without uniform coverage, the OBS dashboard has dark spots; the on-call has to learn each service's idiosyncratic instrumentation. The SDK enforcement guarantees labels are consistent (so cross-service joins work in Grafana) and that cardinality is bounded (so Prometheus storage stays sane). The CI gate is the load-bearing guard — without it, a new service can ship to prod un-instrumented and only get noticed during an incident.

## §3 — Measurement

- Prometheus query `count by (service) (request_rate_total)` lists services emitting RED. Expected: equal to count of deployed services from the manifest.
- Gauge `obs_red_coverage_gap_total{service}` — emitted by a daily cron that diffs deployed-services vs metrics-seen; should always be zero.
- Sev-3 alarm if any service has been deployed > 1h without RED metrics seen.

## §4 — Verification

- CI gate `tests/obs/red_middleware_present_test.sh` (I) — greps every `services/*/src/main.rs` for the SDK install call.
- Inspection (I) — quarterly OBS audit lists all services + their RED dashboard tile; missing tiles get a remediation ticket.

## §5 — Failure handling

- Service deployed without RED → CI gate caught the PR; revert or hot-fix with SDK install.
- Service in prod stops emitting (regression) → sev-3 alarm; on-call inspects whether `cyberos-obs-sdk` upgrade broke compatibility.
- `route` cardinality > 200 → SDK emits a warning log; investigator reviews route templating in the service's middleware config.

---

*End of NFR-OBS-001.*
