---
task_id: TASK-TEN-101
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands the public self-serve signup primitive on top of TASK-AUTH-104 (OIDC) + TASK-TEN-001 (provisioning) + TASK-TEN-002 (plan tiers) + TASK-TEN-003 (Stripe billing). Final form: 1,160 lines, 27 §1 normative clauses (covering 4 migrations, 6 public REST endpoints, OTP HMAC, rate limiting, Turnstile, disposable-email blocklist, GeoIP-derived currency, consent versioning with PDPL VN variant, dual-rail payment intent, OIDC alternate path with CSRF defense, phased commit-or-rollback orchestrator, 9 core + 2 supporting memory audit kinds, 30-second SLO, dual-scope RLS for pre-tenant rows, squatting detection, 90-day PII scrub, idempotency), 20 acceptance criteria, 10 verification tests, 24 failure-mode rows, 27 implementation notes.

The audit identified 9 issues across orchestrator-ordering correctness, missing schema constructs, OIDC CSRF defense, ambiguous failure-mode recoveries, and audit-kind enumeration completeness. All resolved before this 10/10 score.

## §2 — Findings (all resolved)

### ISS-001 — Orchestrator held DB tx open across external Stripe API call

The first draft of §1 #14 put the Stripe `confirm_setup_intent` call (Step 7, external network call to api.stripe.com) **inside** the same Postgres tx that performed provisioning (Steps 6, 8, 10, 12, 13). Holding a Postgres connection open during an external API call (potentially 200–5000 ms) starves the connection pool and risks lock conflicts. Resolved: §1 #14 split into 4 explicit phases — **A (pre-tx validation)**, **B (Stripe call, no tx held)**, **C (atomic provisioning tx, db-only)**, **D (post-commit side effects)**. Phase B records the SetupIntent ID in `signup_sessions.payment_captured_at` so failures in Phase C still know what to void. §6.1 skeleton code rewritten to match.

### ISS-002 — `tenants.billing_contact_email_hash16` referenced but never created

§1 #14 step 5 (duplicate-email guard) requires looking up `tenants WHERE billing_contact_email_hash16 = HMAC(salt, email)`. TASK-TEN-003 added `billing_contact_email` (raw text) but not the hash16 column or index. Without the index, the duplicate-email check is a sequential scan at every signup_complete — kills the 30-second SLO at any non-trivial tenant count. Resolved: §3.1 migration `0011_signup_sessions.sql` now starts with an `ALTER TABLE tenants ADD COLUMN billing_contact_email_hash16 GENERATED ALWAYS AS (...) STORED` and creates the partial index on `WHERE status='active'`.

### ISS-003 — OIDC return URL lacked CSRF defense

§1 #15 said "OIDC return URL is `/v1/signup/oidc-callback`; the handler extracts email + email_verified from the ID token" — but did not bind the inbound callback's `state` parameter to the originating `signup_session_id`. An attacker could trick a victim into completing OIDC against the attacker's signup_session, attaching the victim's IdP identity to the attacker's tenant-in-progress. Resolved: §1 #15 step 2 adds a new endpoint `POST /v1/signup/oidc-init` that mints a single-use server-side `state` nonce bound to the `signup_session_id` with 5-min TTL; the callback validates `state` against the binding before processing. Standard OAuth2 CSRF defense pattern.

### ISS-004 — JWT cookie not set on the redirect target hostname

§1 #14 final step (originally Step 13) returned a JWT in the response body and a `redirect_url` to `<slug>.cyberos.world/onboarding/welcome`, but did NOT set a cookie. The user lands on the welcome page un-authenticated (cookie scoped to `signup.cyberos.world`, not `*.cyberos.world`). Resolved: §1 #14 Phase D Step 14 explicitly sets a Secure HttpOnly SameSite=Lax `cyberos_jwt` cookie scoped to `*.cyberos.world` with 24h TTL; §6.1 skeleton includes the cookie write.

### ISS-005 — "AUTH root-admin creation fails" failure-mode recovery cited TASK-TEN-104

§10 row 11 ("AUTH root-admin creation fails after provisioning") said "TASK-TEN-104 hard-terminate invoked on just-created tenant". But after the orchestrator restructure (ISS-001), root-admin creation happens INSIDE the Phase-C atomic tx, so its failure triggers `ROLLBACK` and no tenant ever exists post-rollback — TASK-TEN-104 termination has nothing to terminate. The original wording would mislead an incident responder. Resolved: §10 row rewritten to reflect the atomic tx semantics — rollback undoes consents + tenant + root-admin atomically; TASK-TEN-104 NOT invoked; Stripe SetupIntent voided.

### ISS-006 — 9-kind list contradicted §1 #4 + §1 #21 mentions of additional kinds

§1 #19 claimed "9 memory audit row kinds", but §1 #4 referenced `ten.disposable_email_blocklist_refreshed` and §1 #21 referenced `ten.signup_session_scrubbed` as "informational only — not in the 9-kind core list". This is 11 kinds total with confusing labelling. A TASK-AI-003 closed-set extension (per task-audit skill rule 8) needs a complete list. Resolved: §1 #19 restructured to explicitly call out 9 **core** kinds (user-action-triggered) plus 2 **supporting** kinds (system-emitted). TASK-AI-003 closed-set extension updates list all 11.

### ISS-007 — `signup_rate_limit_journal` purpose unclear vs Redis hot path

§3.1 defined `signup_rate_limit_journal` Postgres table but §11.6 said "actual counters live in Redis sliding-window for performance." A reader would assume the Postgres table is redundant. Resolved: §3.1 inline comment clarifies the table is a forensic journal — only HIT events logged for compliance/analytics ("how many EU-region blocks last quarter?"), misses are NOT logged. Hot-path counters remain in Redis.

### ISS-008 — Phase D post-commit Stripe failure undefined

The post-restructure §1 #14 Phase D includes `ensure_customer` + `ensure_subscription` (TASK-TEN-003 calls) AFTER the tx commits. If these fail, the tenant already exists — what happens? The first revision didn't say. Resolved: Phase D Step 12 explicitly states: post-commit Stripe failure leaves the tenant in `dunning_state='retry_1'`, and TASK-TEN-003 §1 #11 dunning state machine takes over recovery. §6.1 skeleton matches.

### ISS-009 — `payment_captured_at` column populated in Phase B but its update grant not in §3.1

§3.1 originally REVOKE'd UPDATE/DELETE from `cyberos_app` on `signup_sessions`. But Phase B (per the ISS-001 restructure) needs to set `payment_captured_at` BEFORE the Phase-C tx opens. The column-level GRANT in the migration didn't include `payment_captured_at`. Resolved: §3.1 `GRANT UPDATE` column list already includes `payment_captured_at` (it was in the first draft; verified post-restructure that the column is in the grant list).

## §3 — Resolution

All 9 mechanical concerns addressed. The orchestrator now follows the cardinal rule of "no external API calls inside DB tx" (ISS-001 + ISS-008); duplicate-email lookup is index-backed (ISS-002); OIDC has CSRF defense (ISS-003); the redirect lands authenticated (ISS-004); failure-mode recoveries are forensically accurate (ISS-005); the audit-kind list is exhaustive (ISS-006); ops surfaces clarified (ISS-007); grant lineage clean (ISS-009).

The 1,160-line length sits above the task-audit skill §3.14 "above 1,000 lines suggests prose padding" soft cap. Justification: the task introduces 4 migrations, 6 public REST endpoints, OIDC alternate path with its own init+callback handlers, 11 memory audit kinds, dual-rail payment intent with placeholder for TASK-TEN-102, 4 rate-limit guards, GeoIP+disposable+squatting defenses, 30-second SLO with breakdown analysis, dual-scope RLS for pre-tenant rows, 90-day PII scrub. Genuine surface complexity. Density of normative clauses + failure modes + tests per line is comparable to peer TASK-TEN-003 at 1,056 lines — both spec the commercial substrate.

**Score = 10/10.**

---

*End of TASK-TEN-101 audit.*
