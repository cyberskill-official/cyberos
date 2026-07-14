---
id: NFR-AUTH-008
title: "AUTH Idempotency-Key TTL — 24-hour replay window; hourly cleanup sweeper"
module: AUTH
category: reliability
priority: MUST
verification: T
phase: P0
slo: "Idempotency keys retained 24h; cleanup sweeper runs hourly removing expired keys"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-AUTH-001]
---

## §1 — Statement (BCP-14 normative)

1. Idempotency-Key handling on admin POST endpoints (per TASK-AUTH-001 §1 #6) **MUST** retain seen keys for exactly 24 hours from first-seen-at timestamp.
2. A replay of the same key within 24h **MUST** return the prior response body verbatim with the same HTTP status; no new resource is created.
3. A replay after 24h **MUST** be treated as a fresh request (could create a duplicate; caller's responsibility to use a fresh key past 24h).
4. The cleanup sweeper **MUST** run at least hourly (cron `0 * * * *`) and delete rows where `first_seen_at < now() - interval '24 hours'`.
5. Cleanup failures **MUST NOT** block new key reads/writes — the idempotency table is allowed to grow temporarily (with sev-3 alert at > 1M rows).

## §2 — Why this constraint

24h is the standard idempotency window for industry HTTP APIs (Stripe, GitHub, Anthropic all use 24h). Shorter (1h) would force clients to refresh keys mid-job; longer (7d) inflates table size without practical benefit. The hourly sweeper keeps the table O(daily writes); without it, the table grows unbounded. The "don't block on cleanup failure" rule means a stuck cleanup doesn't impair the critical-path admit flow — eventual consistency is fine for cleanup.

## §3 — Measurement

- Gauge `auth_idempotency_keys_total` — current row count. Sev-3 alarm at > 1M.
- Counter `auth_idempotency_cleanup_total{result}` per cleanup run.
- Counter `auth_idempotency_replay_total{within_window}` — how often clients replay within 24h.

## §4 — Verification

- Integration test `services/auth/tests/admin_tenant_idempotency_test.rs` (T) — drives same key twice within 24h; asserts second response is identical.
- TTL test (T) — fast-forwards time 24h+1s; asserts replay treated as fresh.

## §5 — Failure handling

- Table > 1M rows → sev-3; investigate whether cleanup sweeper is failing.
- Cleanup sweeper down > 6h → sev-2; manual cleanup script; investigate root cause.
- Replay rate > 50% of writes → sev-3 informational; clients are retrying heavily, investigate whether upstream caller has a bug.

---

*End of NFR-AUTH-008.*
