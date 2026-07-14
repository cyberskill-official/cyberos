---
id: NFR-AUTH-005
title: "AUTH RoleMatrix refresh cadence — ≤ 60s revocation latency; alert on 3 consecutive failures"
module: AUTH
category: reliability
priority: MUST
verification: T
phase: P0
slo: "RoleMatrix refresher runs every AUTH_RBAC_REFRESH_SECS (default 60s); 3 consecutive failures triggers alert"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-AUTH-101]
---

## §1 — Statement (BCP-14 normative)

1. The `services/auth/src/rbac/refresher.rs` background task **MUST** call `RoleMatrix::load_from_db()` every `AUTH_RBAC_REFRESH_SECS` seconds (default 60s, floor 5s).
2. A successful refresh **MUST** atomically swap the snapshot via the shared `Arc<RwLock<RoleMatrix>>`. Failed refreshes **MUST NOT** clear the prior snapshot — the previous matrix keeps serving.
3. Three consecutive refresh failures **MUST** trigger a sev-2 alert. The on-call investigates whether the DB is unreachable, the matrix schema has drifted, or the AUTH pod has degraded.
4. Revocations (role removed from a subject) **MUST** be honored within `AUTH_RBAC_REFRESH_SECS` seconds (60s default). The 60s window is the DEC-126 documented design assertion; time-critical revocations use the future per-tenant CRL-flush endpoint (TASK-AUTH-111 placeholder).
5. Every refresh attempt **MUST** emit a structured log row `auth.rbac.refresh` with `{result, prev_version, next_version, duration_ms, error_class}`.

## §2 — Why this constraint

60-second revocation latency is the DEC-126 design contract. Shorter (10s) would multiply DB read load (60x more queries); longer (5min) leaves a wider window where a terminated employee retains access. The 60s window is the documented trade. The three-consecutive-failures rule prevents alert storms (one transient DB blip shouldn't page) while still catching real degradation. The "don't clear on failure" rule is critical — clearing would cause every request to fail-closed during a 1s blip, which is much worse than serving slightly-stale roles for 60s.

## §3 — Measurement

- Counter `auth_rbac_refresh_total{result}` where result ∈ {`success`, `error`}.
- Gauge `auth_rbac_matrix_age_seconds` — time since last successful refresh. Sev-3 alarm at > 120s; sev-2 at > 300s.
- Gauge `auth_rbac_matrix_version` — incremented on every successful swap.

## §4 — Verification

- Unit test `services/auth/src/rbac/refresher_test.rs` (T) — simulates 3 consecutive DB failures; asserts prior snapshot still serves, alert metric incremented.
- Integration test `services/auth/tests/rbac_revocation_latency_test.rs` (T) — assigns role, then removes role, then verifies the next call is denied within `AUTH_RBAC_REFRESH_SECS + 5s` grace.

## §5 — Failure handling

- 3 consecutive failures → sev-2 alert; on-call investigates within 15 minutes.
- Matrix age > 5min → sev-2; the in-memory snapshot is now meaningfully stale; consider restarting the AUTH pod to force a fresh load.
- DB schema drifted from `RoleMatrix::load_from_db` expectations → sev-1; emergency code fix; rollback if needed.

---

*End of NFR-AUTH-005.*
