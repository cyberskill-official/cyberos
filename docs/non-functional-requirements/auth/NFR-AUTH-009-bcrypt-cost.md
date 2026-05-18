---
id: NFR-AUTH-009
title: "AUTH bcrypt cost — ≥ 12 rounds; tunable via AUTH_BCRYPT_COST env"
module: AUTH
category: security
priority: MUST
verification: I
phase: P0
slo: "Password hashing uses bcrypt cost ≥ 12; production override via AUTH_BCRYPT_COST env"
owner: CSO
created: 2026-05-18
related_frs: [FR-AUTH-002]
---

## §1 — Statement (BCP-14 normative)

1. Password hashing in AUTH **MUST** use bcrypt with cost factor ≥ 12 (default 12; configurable via `AUTH_BCRYPT_COST` env var).
2. The cost factor **MUST** be inspectable at runtime via the `/healthz` endpoint's `bcrypt_cost` field.
3. Hashing **MUST** be performed on the request thread asynchronously (`spawn_blocking` in tokio) to avoid blocking the event loop — bcrypt at cost 12 is ~250ms which would otherwise stall the runtime.
4. The cost factor **MUST** be reviewed annually by CSO; if hardware capacity makes cost 12 brute-forceable in < 10s, increase to 13.
5. Existing password hashes at lower cost **MUST** be transparently upgraded on next successful login (rehash and store at new cost).

## §2 — Why this constraint

Bcrypt cost 12 is the 2026 industry baseline for password storage. Cost 10 (older default) is now brute-forceable in under a second on GPU. Cost 13 doubles the work — overkill for current hardware but a useful headroom-future-proof. The `spawn_blocking` rule is a tokio runtime correctness property — a sync 250ms call on the event loop kills async throughput. Transparent upgrade on login avoids the "we need to force everyone to reset" migration pain.

## §3 — Measurement

- Histogram `auth_password_hash_seconds` — at cost 12 expect ~250ms; at cost 13 expect ~500ms. Alerts at < 100ms (cost too low) and > 1000ms (cost too high or CPU contention).
- `/healthz` field `bcrypt_cost` — operator-inspectable.

## §4 — Verification

- Inspection (I) — quarterly CSO audit of `AUTH_BCRYPT_COST` env in production helm/k8s manifests.
- Unit test `services/auth/tests/password_hash_cost_test.rs` (T) — asserts default cost is 12; asserts env var override works.

## §5 — Failure handling

- Cost < 12 detected in production → sev-2; rotate env var, restart pods; existing hashes upgrade on next login.
- Hash latency > 1s → sev-3; either CPU contention (scale up) or cost too high (review).
- New hash created at lower cost (regression) → sev-1; emergency revert.

---

*End of NFR-AUTH-009.*
