---
id: NFR-AUTH-001
title: "AUTH admin endpoint admission latency — verify_jwt + RBAC check < 50ms p95"
module: AUTH
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 50ms for verify_jwt + RBAC admit on /v1/admin/* routes"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-AUTH-004, TASK-AUTH-005, TASK-AUTH-101]
---

## §1 — Statement (BCP-14 normative)

1. The AUTH service **MUST** complete the admission pipeline `verify_jwt → RBAC.check → admit` at **p95 < 50ms** and **p99 < 100ms** measured at the service ingress, over a 14-day rolling window, for every `/v1/admin/*` route.
2. JWT signature verification (the dominant cost) **MUST** consume < 25ms p99 — see NFR-AUTH-003 for the JWT-specific budget.
3. RBAC matrix lookup **MUST** consume < 5ms p99 — the matrix lives in-memory under an `Arc<RwLock<RoleMatrix>>` and is refreshed by the 60s background refresher (`services/auth/src/rbac/refresher.rs`).
4. JWKS fetches (cold cache) are NOT counted in this SLO — they happen at startup and on rotation; warm-cache lookups are sub-millisecond.
5. The admission pipeline **MUST NOT** emit a network call (no DB, no remote RBAC, no OAuth introspection) on the happy path — everything is in-memory.

## §2 — Why this constraint

`/v1/admin/*` endpoints are on the critical path of operator workflows (tenant create, user provision, key revoke). A 200ms admission overhead would make the admin UI feel laggy. The 50ms ceiling preserves the perception that "admin actions feel instant." The pipeline must be in-memory because any network call inside admission would couple the platform's admin surface to network reliability — admin operations should work even during partial outages.

## §3 — Measurement

- Histogram `auth_admit_latency_seconds{route, result}` emitted by `services/auth/src/middleware/admit.rs` over the verify→check→admit span.
- Sub-histograms `auth_jwt_verify_seconds`, `auth_rbac_check_seconds` per NFR-AUTH-003 and §1 #3.
- p95 alarm at > 50ms; p99 alarm at > 100ms.

## §4 — Verification

- Criterion benchmark `services/auth/benches/admit_pipeline.rs` (T) — 100k synthetic admit calls; asserts p99 < 50ms on CI runner.
- Integration test `services/auth/tests/admit_latency_test.rs` (T) — drives 1000 /v1/admin/tenants GET calls; asserts p95 < 50ms end-to-end.

## §5 — Failure handling

- p95 > 50ms for 10 minutes → sev-3; inspect JWT/RBAC sub-histograms to identify culprit.
- p99 > 200ms → sev-2; possible JWKS fetch on every call (cache broken); restart AUTH pod and investigate.
- DB call detected on admit path (via `auth_admit_db_calls_total`) → sev-2; admission has been silently de-optimised; revert.

---

*End of NFR-AUTH-001.*
