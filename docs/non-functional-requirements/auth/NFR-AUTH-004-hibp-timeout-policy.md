---
id: NFR-AUTH-004
title: "AUTH HIBP API timeout policy — 2s hard timeout; fail-open with audit row on timeout"
module: AUTH
category: security
priority: MUST
verification: T
phase: P0
slo: "HIBP API call wrapped in 2s hard timeout; timeout never blocks signup; audit row emitted on every timeout"
owner: CSO
created: 2026-05-18
related_frs: [FR-AUTH-107]
---

## §1 — Statement (BCP-14 normative)

1. The HaveIBeenPwned (HIBP) k-anonymity API call **MUST** be wrapped in a 2s hard timeout. If HIBP doesn't respond in 2s, the call is aborted.
2. On HIBP timeout, AUTH **MUST** fail-open — the signup or password rotation proceeds without breach check — but an audit row `auth.hibp.timeout` is emitted to memory with `{subject_id, route, attempt_n}` for retroactive review.
3. AUTH **MUST NOT** retry HIBP on timeout within the same caller request — one attempt only. A retry would amplify the latency penalty.
4. HIBP API key (the User-Agent header per HIBP ToS) **MUST** be loaded from environment variable `HIBP_USER_AGENT`; never hard-coded.
5. The HIBP module **MUST** be locally bypassable in dev mode via `AUTH_HIBP_MODE=skip` env var — production deployments must set `AUTH_HIBP_MODE=enforce`.

## §2 — Why this constraint

HIBP is a third-party API outside the platform's control. Without a hard timeout, a slow HIBP day cascades into 10-30s signup latency — users abandon. The 2s ceiling fails fast; fail-open is the lesser of two evils (a slightly weaker security check for one signup vs. blocking all signups during HIBP outage). The audit row makes the failure mode visible: weekly review of `auth.hibp.timeout` counts tells security whether HIBP availability is degrading. The no-retry rule prevents amplification — one 2s wait is acceptable, three 2s waits is not.

## §3 — Measurement

- Counter `auth_hibp_timeout_total` — incremented on every 2s abort. Sev-3 alarm at > 10/hour.
- Counter `auth_hibp_check_total{result}` where result ∈ {`clean`, `breached`, `timeout`, `error`}.
- memory audit query `view kind=auth.hibp.timeout` — weekly review by security.

## §4 — Verification

- Integration test `services/auth/tests/hibp_timeout_test.rs` (T) — simulates HIBP server returning 3s delay; asserts signup completes in < 2.5s, audit row emitted, signup succeeds.
- Unit test `hibp_no_retry_test.rs` (T) — verifies one and only one HIBP call per signup.

## §5 — Failure handling

- Timeout count > 10/hour → sev-3; HIBP may be degraded; security reviews whether to switch to `AUTH_HIBP_MODE=skip` temporarily.
- Sustained > 50/hour → sev-2; effective HIBP coverage is degraded; CSO sign-off whether to continue or pause new signups.
- HIBP API key invalid → sev-2; on-call rotates the User-Agent and redeploys.

---

*End of NFR-AUTH-004.*
