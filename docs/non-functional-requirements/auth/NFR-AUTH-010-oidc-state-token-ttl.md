---
id: NFR-AUTH-010
title: "AUTH OIDC state-token TTL — 10-minute hard expiry; CSRF-resistant; one-shot"
module: AUTH
category: security
priority: MUST
verification: T
phase: P0
slo: "OIDC state tokens expire 10 minutes after issuance; single-use; CSRF-bound to session"
owner: CSO
created: 2026-05-18
related_tasks: [TASK-AUTH-104]
---

## §1 — Statement (BCP-14 normative)

1. OIDC `state` parameters issued during the Authorization Code flow **MUST** expire exactly 10 minutes after issuance. The callback handler rejects expired states with HTTP 400 `state_expired`.
2. Each state value **MUST** be single-use — after the callback consumes it, the same value cannot be replayed. Replay attempts return HTTP 400 `state_already_consumed`.
3. State **MUST** be bound to the originating session (CSRF protection): the callback verifies the cookie's session_id matches the session_id encoded in the state's MAC.
4. State values **MUST** be ≥ 32 bytes of cryptographic randomness, encoded base64url; the MAC uses HMAC-SHA256 over `{session_id, nonce, issued_at}`.
5. Every state consumption (success or fail) **MUST** emit a memory audit row `auth.oidc.state_consumed` with `{session_id, result, time_since_issuance_seconds}`.

## §2 — Why this constraint

The OAuth 2.1 spec mandates state for CSRF protection but doesn't specify a TTL — without one, a malicious link can phish a user days later. 10 minutes is the industry standard (Google, GitHub, Microsoft all use 5-15min); it accommodates a slow user (open the IdP login page, walk away briefly, return) while bounding the phishing window. Single-use is critical — without it, an attacker who captures one state value can replay arbitrarily. The session binding closes the bearer-token shape: even if an attacker captures a state, they can't use it without the originating session cookie.

## §3 — Measurement

- Counter `auth_oidc_state_consumed_total{result}` where result ∈ {`success`, `expired`, `replayed`, `csrf_mismatch`, `invalid_mac`}.
- Sev-2 alarm on `replayed` or `csrf_mismatch` > 0 in 1h (likely attack).
- Histogram `auth_oidc_state_age_seconds_on_consume` — distribution of how long users take.

## §4 — Verification

- Integration test `services/auth/tests/oidc_state_expiry_test.rs` (T) — drives state issuance, waits 10min+1s, asserts callback rejects with `state_expired`.
- Replay test (T) — consumes state, replays, asserts second callback rejects with `state_already_consumed`.
- CSRF test (T) — issues state in session A, attempts callback with session B's cookie, asserts `csrf_mismatch`.

## §5 — Failure handling

- Replay/CSRF mismatch > 0 → sev-2; likely active CSRF probe; security investigates.
- TTL expiry rate > 10% → sev-3 informational; users may be confused by the flow; UX review of IdP redirect.
- State MAC invalid > 0.01% → sev-3; key rotation may have orphaned old states (expected briefly); investigate if sustained.

---

*End of NFR-AUTH-010.*
