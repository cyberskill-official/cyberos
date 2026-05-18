---
id: NFR-AUTH-006
title: "AUTH TOTP drift tolerance — accept ±1 step; reject ±2; rate-limit retries"
module: AUTH
category: security
priority: MUST
verification: T
phase: P0
slo: "Accept TOTP codes within ±1 30s step (90s window); reject ±2; rate-limit 5 attempts per minute"
owner: CSO
created: 2026-05-18
related_frs: [FR-AUTH-102]
---

## §1 — Statement (BCP-14 normative)

1. The TOTP verify endpoint **MUST** accept codes within ±1 30-second step (effectively a 90-second window: current ± 30s).
2. The endpoint **MUST** reject codes ±2 steps away (60s outside the window); return HTTP 401.
3. Failed TOTP verify attempts **MUST** be rate-limited per (subject_id) at 5 attempts per minute. The 6th attempt within a sliding 60s window returns HTTP 429 with `Retry-After: 60`.
4. Successful verify resets the rate-limit counter for that subject.
5. Each verify attempt (success or fail) **MUST** emit a BRAIN audit row `auth.totp.verify` with `{subject_id, result, attempt_n, ts}`.

## §2 — Why this constraint

±1 step accommodates clock drift between the user's device (phone) and the AUTH server (~5-30s drift is common). ±2 would extend the window to 150s, which weakens TOTP security (an attacker who shoulder-surfs a code has a longer replay window). The 5-per-minute rate limit defends against brute-force — a TOTP code is 6 digits = 1M possibilities; 5/min × 60min = 300/hour limits brute force to one expected success per ~3300 hours. The audit row makes brute-force attempts visible.

## §3 — Measurement

- Counter `auth_totp_verify_total{result}` where result ∈ {`success`, `fail_outside_window`, `fail_invalid_code`, `rate_limited`}.
- Counter `auth_totp_rate_limited_subjects_total` per subject; sev-3 alarm if any subject hits rate-limit > 3 times in an hour (likely attack).
- BRAIN query `view kind=auth.totp.verify` — security reviews weekly.

## §4 — Verification

- Unit test `services/auth/src/totp_test.rs` (T) — drives codes at current, ±1, ±2, ±5 offsets; asserts accept/reject per spec.
- Integration test `services/auth/tests/totp_rate_limit_test.rs` (T) — drives 6 failed attempts in 60s; asserts 6th returns HTTP 429.

## §5 — Failure handling

- Subject hits rate-limit > 3x in an hour → sev-3; security investigates whether user is locked out (call support) or under attack (escalate).
- Cluster-wide rate-limit triggering > 100 subjects/hour → sev-2; possible coordinated attack; CSO involvement.
- TOTP verify endpoint p99 > 50ms → sev-3 (likely DB query for the secret got slow); investigate.

---

*End of NFR-AUTH-006.*
