---
id: FR-TEN-101
title: "Self-serve signup form ≤ 30 s end-to-end — email OTP + slug + plan + currency + payment + provisioning + root-admin + first-login JWT in one orchestrated flow"
module: TEN
priority: MUST
status: draft
verify: T
phase: P3
milestone: P3 · self-serve
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-TEN-001, FR-TEN-002, FR-TEN-003, FR-TEN-004, FR-TEN-102, FR-TEN-103, FR-TEN-104, FR-TEN-107, FR-AUTH-001, FR-AUTH-002, FR-AUTH-004, FR-AUTH-101, FR-AUTH-104, FR-AUTH-107, FR-PORTAL-001, FR-PORTAL-002, FR-AI-003, FR-MEMORY-111, FR-EMAIL-001, FR-OBS-007]
depends_on: [FR-AUTH-104, FR-TEN-001, FR-TEN-002, FR-TEN-003]
blocks: [FR-TEN-107, FR-PORTAL-001, FR-PORTAL-002]

source_pages:
  - website/docs/modules/ten.html#signup
  - website/docs/modules/ten.html#self-serve
  - https://nordpass.com/blog/disposable-email/  # blocklist reference
  - https://challenges.cloudflare.com/turnstile/v0/api.js
  - https://gdpr.eu/article-7-conditions-for-consent/
  - https://www.iso.org/standard/82875.html  # PDPL Law 91/2025 reference text

source_decisions:
  - DEC-820 2026-05-17 — Public signup endpoint is unauthenticated; rate-limited by IP + per-email + per-email-domain; protected by Cloudflare Turnstile (or reCAPTCHA v3) at every public mutation
  - DEC-821 2026-05-17 — Email-OTP verification before tenant creation; 6-digit code, 10-min TTL, max 5 attempts; OIDC sign-up path (via FR-AUTH-104) skips OTP because IdP already verified
  - DEC-822 2026-05-17 — Tenant slug is user-chosen with real-time uniqueness check; slug regex per FR-TEN-001 DEC-321 (`^[a-z][a-z0-9-]{2,40}[a-z0-9]$`); 50 auto-suggestions on collision
  - DEC-823 2026-05-17 — Plan tier defaults to Starter (FR-TEN-002 DEC-778); user may upgrade to Team/Enterprise inline; Enterprise requires manual sales contact (signup limited to Starter+Team for self-serve)
  - DEC-824 2026-05-17 — Billing currency derived from IP geolocation (MaxMind GeoLite2): VN→VND, SG→SGD, EU countries→EUR, UK→GBP, default→USD; user may override (e.g. Singapore-based user wants to bill in USD)
  - DEC-825 2026-05-17 — Residency derived from billing_currency: VND→vn-1, SGD→sg-1, EUR→eu-1, GBP→eu-1, USD→us-1 (consistent with FR-TEN-003 DEC-785)
  - DEC-826 2026-05-17 — Payment method captured via Stripe Elements (for stripe-rail) or VnPay redirect (for VND-rail per FR-TEN-102 — placeholder) in the same flow; no card details ever touch our backend (PCI SAQ-A scope)
  - DEC-827 2026-05-17 — Signup completes when: email verified + slug provisioned + payment method authorized + root-admin subject created + first-login JWT minted; partial states roll back with idempotent reattempt
  - DEC-828 2026-05-17 — 30-second end-to-end SLO: signup_started → signup_completed p95 ≤ 30 s; SLI emitted to OBS; alarm sev-2 if p95 > 30 s sustained 15 min
  - DEC-829 2026-05-17 — Idempotent on `signup_session_id` (UUIDv7) generated client-side at form-load; resumable across page reload; persisted in `signup_sessions` table with 1h TTL
  - DEC-830 2026-05-17 — Disposable email blocklist enforced from `disposable_email_domains.txt` (10k+ entries from public list, refreshed monthly); blocked domains return 400 + suggestion to use work email
  - DEC-831 2026-05-17 — FR-AUTH-107 HIBP breach-check applied to email at signup (informational warning, not blocker) — leaked email still allowed but operator alerted
  - DEC-832 2026-05-17 — GDPR Art. 7 explicit consent checkbox required for marketing emails (separate from ToS/Privacy acceptance which is contractual); consent versioned (`tos_version`, `privacy_version`, `marketing_consent_version`)
  - DEC-833 2026-05-17 — PDPL Law 91/2025 Art. 14 — VN-residency tenants get a Vietnamese-language consent variant + data-export consent (separate opt-in); slug+VN consent rows persisted in `tenant_consents` table
  - DEC-834 2026-05-17 — Root-admin subject created with auto-generated 24-character password (zxcvbn score ≥ 4); emailed to billing_contact_email AFTER signup_completed; password requires immediate rotation on first login
  - DEC-835 2026-05-17 — OIDC sign-up path (FR-AUTH-104 "Sign up with Google/Microsoft") creates the root-admin subject WITHOUT a password; subsequent logins are OIDC-only until tenant_admin enables password fallback
  - DEC-836 2026-05-17 — First-login JWT minted via FR-AUTH-004 with `signup_session_id` claim; cookies set on the public signup hostname (`signup.cyberos.world` redirects to `<slug>.cyberos.world` after final step)
  - DEC-837 2026-05-17 — Rate limits: 10 signup_start/min/IP, 3 OTP_send/min/email, 50 slug_check/min/IP, 1 signup_complete/min/IP; sliding-window via Redis with hard cap
  - DEC-838 2026-05-17 — Disposable email blocklist + IP geo lookup happen synchronously inside `signup_start` for early failure (no money charged on disposable email path)
  - DEC-839 2026-05-17 — memory audit kinds: ten.signup_started, ten.signup_email_verified, ten.signup_consent_recorded, ten.signup_tenant_provisioned, ten.signup_completed, ten.signup_abandoned, ten.signup_rate_limited, ten.signup_disposable_email_blocked, ten.signup_oidc_linked
  - DEC-840 2026-05-17 — Signup analytics retained in `signup_sessions` for 90 days (funnel analysis); PII (email, IP) auto-scrubbed at 90d to hash-only; legal basis = legitimate interest (GDPR Art. 6(1)(f))
  - "DEC-841 2026-05-17 — Founder bypass path: `POST /v1/admin/tenants` (FR-TEN-001's ops CLI flow) STILL works alongside this; self-serve does NOT replace ops provisioning — they are parallel entry points"
  - DEC-842 2026-05-17 — Same-email re-signup attempt (already-active tenant under this email): block with 409 + "magic-link sign-in" alternative emailed; never silently create a second tenant
  - DEC-843 2026-05-17 — Stripe Customer + Subscription created BEFORE provisioning tenant (DEC-826 derivative): payment method authorization is the commercial gate; tenant exists only if payment confirms; rollback on Stripe failure cleans up `signup_sessions` row + email-verified state retained for retry
  - DEC-844 2026-05-17 — Slug squatting protection: same-IP signup attempts that abandon before payment (≥ 5 in 24h) trigger IP block (24h cool-off); humans rarely abandon 5 signups; bots do
  - DEC-845 2026-05-17 — Welcome email contains a deeplink to `<slug>.cyberos.world/onboarding/welcome` that expires in 7 days; subsequent visits redirect to `<slug>.cyberos.world/`
  - PCI DSS SAQ-A (Stripe Elements + VnPay redirect; no PAN at our endpoint)
  - GDPR Art. 7 (consent), Art. 6(1)(f) (legitimate interest), Art. 25 (data protection by design)
  - PDPL Law 91/2025 Art. 14 (VN consent), Art. 17 (data subject rights — informed at signup)
  - eIDAS QTSP-readiness (slice 2 stretch — out-of-scope here per DEC-XXX)

build_envelope:
  language: rust 1.81 + typescript 5.5 (frontend)
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0011_signup_sessions.sql                 # 1h-TTL signup session state
    - services/ten/migrations/0012_tenant_consents.sql                 # ToS / Privacy / Marketing / PDPL version pins
    - services/ten/migrations/0013_signup_rate_limits.sql              # per-IP / per-email sliding-window counters
    - services/ten/migrations/0014_disposable_email_domains.sql        # 10k-entry blocklist + monthly refresh ledger
    - services/ten/src/signup/mod.rs                                   # signup orchestrator
    - services/ten/src/signup/start.rs                                 # POST /v1/signup/start
    - services/ten/src/signup/otp.rs                                   # send_otp + verify_otp
    - services/ten/src/signup/slug_check.rs                            # GET /v1/signup/slug-available + suggestions
    - services/ten/src/signup/payment_intent.rs                        # Stripe SetupIntent + VnPay redirect builder
    - services/ten/src/signup/complete.rs                              # POST /v1/signup/complete (commit-or-rollback orchestrator)
    - services/ten/src/signup/oidc_callback.rs                         # FR-AUTH-104 OIDC return → tenant create
    - services/ten/src/signup/consent.rs                               # ToS/Privacy/Marketing/PDPL versioned acceptance
    - services/ten/src/signup/rate_limit.rs                            # sliding-window via Redis
    - services/ten/src/signup/disposable_email.rs                      # blocklist loader + check
    - services/ten/src/signup/geoip.rs                                 # MaxMind GeoLite2 lookup → currency/residency hint
    - services/ten/src/signup/abuse_guard.rs                           # squatting / abandonment detector
    - services/ten/src/signup/welcome_email.rs                         # delegates to FR-EMAIL-001
    - services/ten/src/audit/signup_events.rs                          # 9 memory row builders
    - services/ten/src/handlers/signup_routes.rs                       # axum route registration
    - services/ten/web/signup/index.html                               # static single-page signup form
    - services/ten/web/signup/signup.ts                                # progressive form (5 steps, ≤ 30 s)
    - services/ten/web/signup/turnstile.ts                             # Cloudflare Turnstile widget
    - services/ten/web/signup/stripe_elements.ts                       # Stripe Elements card capture
    - services/ten/tests/signup_happy_test.rs                          # full 30-s flow end-to-end
    - services/ten/tests/signup_otp_test.rs                            # OTP gen + verify + expiry + max-attempts
    - services/ten/tests/signup_slug_test.rs                           # uniqueness + suggestions
    - services/ten/tests/signup_disposable_email_test.rs               # blocklist enforcement
    - services/ten/tests/signup_rate_limit_test.rs                     # 4 rate-limit guards
    - services/ten/tests/signup_consent_versioning_test.rs             # ToS/Privacy version pinned in row
    - services/ten/tests/signup_geoip_currency_test.rs                 # IP→currency derivation + override
    - services/ten/tests/signup_oidc_path_test.rs                      # Google/Microsoft OIDC flow
    - services/ten/tests/signup_stripe_rollback_test.rs                # Stripe failure → tenant not created
    - services/ten/tests/signup_30s_sla_test.rs                        # end-to-end < 30 s on test fixture
    - services/ten/tests/signup_squatting_test.rs                      # 5 abandoned signups → IP block
    - services/ten/tests/signup_duplicate_email_test.rs                # 2nd signup same email → 409 + magic link
    - services/ten/tests/signup_audit_emission_test.rs                 # 9 memory kinds emitted

  modified_files:
    - services/ten/src/lib.rs                                          # mount signup_routes
    - services/ten/Cargo.toml                                          # +redis, +maxminddb, +zxcvbn, +rand for OTP
    - services/auth/src/admin/subjects.rs                              # expose helper for signup root-admin create
    - services/email/src/templates/welcome_signup.tera                 # new email template + i18n EN/VI

  allowed_tools:
    - file_read: services/ten/**
    - file_read: services/auth/src/admin/**
    - file_read: services/email/src/templates/**
    - file_write: services/ten/{src,tests,migrations,web}/**
    - file_write: services/email/src/templates/welcome_signup.tera
    - bash: cd services/ten && cargo test signup
    - bash: cd services/ten/web/signup && bun test

  disallowed_tools:
    - skip Turnstile/CAPTCHA on any public signup mutation (per DEC-820)
    - persist raw OTP codes in plaintext (HMAC-hashed in Redis per DEC-821)
    - create tenant before Stripe Customer authorization (per DEC-843)
    - allow same email → second active tenant (per DEC-842)
    - sign up an Enterprise tier via self-serve (per DEC-823 — sales-led only)
    - hardcode currency/residency mapping in handler code (single source = signup/geoip.rs per DEC-824)

effort_hours: 10
sub_tasks:
  - "0.6h: 0011_signup_sessions.sql + 0012_tenant_consents.sql + 0013_signup_rate_limits.sql + 0014_disposable_email_domains.sql"
  - "0.5h: disposable_email.rs loader + 10k-entry initial blocklist + monthly refresh job"
  - "0.5h: geoip.rs — MaxMind GeoLite2 download + IP→currency map"
  - "0.6h: rate_limit.rs — Redis sliding-window (4 guards per DEC-837)"
  - "0.6h: otp.rs — generate + HMAC-store + verify + max-attempts + TTL"
  - "0.5h: slug_check.rs — uniqueness + 50 suggestions on collision"
  - "0.5h: consent.rs — versioned acceptance + i18n VI variant"
  - "0.8h: start.rs handler (POST /v1/signup/start) wiring rate-limit + disposable + geoip + Turnstile verify"
  - "0.8h: payment_intent.rs — Stripe SetupIntent for stripe-rail + VnPay placeholder dispatch"
  - "1.0h: complete.rs commit-or-rollback orchestrator (Stripe → TEN-001 provision → AUTH root-admin → JWT mint)"
  - "0.6h: oidc_callback.rs — FR-AUTH-104 return path → headless tenant creation (skip OTP)"
  - "0.4h: abuse_guard.rs — squatting detector + IP block"
  - "0.4h: welcome_email.rs — FR-EMAIL-001 delegate + 7d-expiring deeplink"
  - "0.4h: audit/signup_events.rs — 9 builders"
  - "0.6h: web/signup/* — 5-step single-page form + Turnstile widget + Stripe Elements"
  - "0.8h: tests — 13 test files covering happy + OTP + disposable + rate-limit + 30s-SLO + rollback + squatting + duplicate + OIDC"
  - "0.4h: wire-up — handlers/signup_routes.rs + lib.rs + Cargo.toml deps"
  - "0.4h: SLI emission to OBS — signup_completed timer histogram + 30s alarm"

risk_if_skipped: "Without self-serve signup, every new tenant requires CCO/ops manual provisioning via FR-TEN-001 CLI — non-scalable past ~50/month and adds 24-48h friction to every signup, killing self-serve growth motion (P3 commercial gate). Without DEC-820's Turnstile + rate limits, the public endpoint becomes a bot magnet (slug squatting + payment-card stuffing). Without DEC-821's OTP, signups can claim any email address (mass-spoof to email-bomb victims). Without DEC-830's disposable blocklist, the funnel fills with throwaway emails that never pay. Without DEC-843's Stripe-before-provision ordering, tenants get provisioned then payment fails leaving orphan resources. Without DEC-842's duplicate-email guard, one person gets two tenants and support escalations follow. Without DEC-832's GDPR consent versioning, regulators ask 'what did this user agree to?' and we have no answer. Without DEC-833's PDPL VN-consent variant, every VN tenant is non-compliant on Day 1. Without DEC-828's 30-second SLO, conversion drops 30%+ per industry benchmarks (Optimizely/Baymard: 1s delay → 7% loss; 30s+ pages → 50%+ abandon). The 10h effort lands the self-serve growth primitive that unlocks the entire P3 → P4 commercial arc."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship the public self-serve signup flow at `services/ten/src/signup/` with email-OTP verification, slug uniqueness check, plan + currency selection, payment method capture, idempotent commit-or-rollback orchestration, OIDC alternate path, consent versioning, rate limiting + Turnstile, disposable-email blocklist, 30-second end-to-end SLO, and 9 memory audit kinds.

1. **MUST** expose the unauthenticated `POST /v1/signup/start` endpoint that accepts `{ email, signup_session_id (UUIDv7), turnstile_token, locale_hint }` and returns `{ signup_session_id, email_verification_required: bool, suggested_billing_currency, suggested_residency, geoip_country, otp_sent_at }`. The handler MUST in order: verify Turnstile token (per DEC-820), check rate limits (per DEC-837), check disposable-email blocklist (per DEC-830), GeoIP-lookup the request IP (per DEC-824), HMAC-hash + persist OTP in Redis with 10-min TTL (per DEC-821), trigger transactional email via FR-EMAIL-001 with the OTP, INSERT a `signup_sessions` row, and emit `ten.signup_started` memory row.

2. **MUST** define the `signup_sessions` table at migration `0011`: `(signup_session_id UUID PRIMARY KEY, email_hash16 TEXT NOT NULL, email_full TEXT NOT NULL, geoip_country CHAR(2), suggested_billing_currency billing_currency_enum, suggested_residency TEXT, state TEXT NOT NULL CHECK (state IN ('started','email_verified','payment_captured','provisioning','completed','abandoned','rolled_back')) DEFAULT 'started', started_at TIMESTAMPTZ NOT NULL DEFAULT now(), email_verified_at TIMESTAMPTZ, payment_captured_at TIMESTAMPTZ, provisioned_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, abandon_reason TEXT, rolled_back_reason TEXT, tenant_id UUID, scrubbed_at TIMESTAMPTZ)`. Expires (`signup_sessions.state` transitions to `abandoned` + `abandon_reason='ttl_expired'`) at 1h via scheduled job. PII (email_full, IP if stored) scrubbed at 90 days per DEC-840.

3. **MUST** define the `tenant_consents` table at migration `0012`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID, signup_session_id UUID, subject_id UUID NOT NULL, consent_kind TEXT NOT NULL CHECK (consent_kind IN ('tos','privacy','marketing','pdpl_vn_data_processing','pdpl_vn_data_export','gdpr_legitimate_interest')), version TEXT NOT NULL, accepted_at TIMESTAMPTZ NOT NULL DEFAULT now(), ip_addr_hash16 TEXT NOT NULL, locale CHAR(5) NOT NULL, withdrawn_at TIMESTAMPTZ)`. Append-only via REVOKE per feature-request-audit skill rule 12; withdrawal recorded as new row with `withdrawn_at` populated AND a fresh `accepted_at=null` row is NOT created — withdrawal is a column-level update to the existing row (one of the rare permitted updates, gated via per-column GRANT to the `cyberos_consent_writer` role).

4. **MUST** define the `disposable_email_domains` table at migration `0014`: `(domain TEXT PRIMARY KEY, source TEXT NOT NULL, added_at TIMESTAMPTZ NOT NULL DEFAULT now(), removed_at TIMESTAMPTZ)`. Initial seed = 10,000-entry list from public sources (`disposable-email-domains` GitHub repo at pinned SHA); refresh job runs monthly + writes a `ten.disposable_email_blocklist_refreshed` memory row (kind not in the 9-kind core list per DEC-839 — informational only).

5. **MUST** enforce 4 rate limits per DEC-837 via Redis sliding-window:
   - `signup_start`: 10/min/IP, 3/h/IP, 100/d/IP.
   - `otp_send`: 3/min/email, 10/h/email.
   - `slug_check`: 50/min/IP (rapid autocomplete legit; > 50 = bot).
   - `signup_complete`: 1/min/IP (a human completes signup once per minute max).

   Rate-limit hit returns `429 TOO_MANY_REQUESTS` + `{ error: "rate_limited", retry_after_seconds, guard }` + emits `ten.signup_rate_limited` memory row.

6. **MUST** enforce the disposable-email blocklist at `signup_start` (per DEC-830). Lookup `domain = email.split('@')[1]`. Blocked → return `400 BAD_REQUEST` + `{ error: "disposable_email", suggested_action: "use_work_email" }` + emit `ten.signup_disposable_email_blocked` memory row. The blocklist is loaded into memory at handler startup; refreshed monthly.

7. **MUST** apply FR-AUTH-107 HIBP breach-check on the email at `signup_start` (per DEC-831). Result is informational — `breached: true` is appended to the response as `{ ..., breach_warning: { breach_count: N, latest_breach: "<date>" } }`. The signup proceeds; the warning surfaces in the UI as "We noticed your email appears in breaches; you may want to use a different email."

8. **MUST** expose `POST /v1/signup/verify-otp` that accepts `{ signup_session_id, otp_code }` and returns `{ verified: bool, session_state }`. Handler MUST: lookup the OTP from Redis (HMAC-hashed lookup), constant-time compare, increment attempt counter, reject if `attempts > 5`, on success transition `signup_sessions.state = 'email_verified'` + emit `ten.signup_email_verified` memory row. After 5 failed attempts the OTP is invalidated; a new OTP send is required (incrementing `otp_send` rate-limit counter).

9. **MUST** expose `GET /v1/signup/slug-available?candidate=<slug>` that returns `{ available: bool, suggestions: [string; 50] }`. The handler validates regex `^[a-z][a-z0-9-]{2,40}[a-z0-9]$` (FR-TEN-001 DEC-321), checks against the `tenants.slug` index, and on collision generates 50 suggestions via prefix-suffix permutation (`acme-co`, `acme-1`, `acme-hq`, etc.). The handler is rate-limited (per DEC-837 #3) and serves from a hot Postgres index (no caching — slug must be authoritative). For OIDC sign-up path, a slug suggestion is derived from the email domain (`alice@acme.com → acme`) and offered as default.

10. **MUST** support 2 plan tiers in self-serve per DEC-823: Starter (default) and Team. Enterprise self-serve is BLOCKED (return `403 FORBIDDEN` + `{ error: "enterprise_requires_sales_contact", contact_email: "sales@cyberos.world" }`). The UI hides the Enterprise option for self-serve; direct API attempts are rejected at `signup_complete`.

11. **MUST** derive `suggested_billing_currency` from GeoIP (per DEC-824) via MaxMind GeoLite2 database refreshed weekly: VN→VND, SG→SGD, EU 27→EUR, GB→GBP, default→USD. User MAY override via the form (e.g., a Singapore-based user wants USD billing for tax reasons). The derivation result is informational; the final `billing_currency` is captured at `signup_complete` from the request body, not from GeoIP.

12. **MUST** derive `residency` from `billing_currency` per DEC-825: VND→vn-1, SGD→sg-1, EUR→eu-1, GBP→eu-1, USD→us-1. Residency is immutable post-provisioning (FR-TEN-103 derivative); FR-TEN-001 carries this through to provisioning.

13. **MUST** expose `POST /v1/signup/payment-intent` that returns either:
    - For stripe-rail (`billing_currency ∈ {USD, EUR, SGD, GBP}`): `{ stripe_client_secret: "seti_xxx_secret_yyy", publishable_key: "pk_xxx", payment_method_options: { card: { request_three_d_secure: "automatic" } } }`. The frontend uses Stripe Elements to capture card data + SetupIntent confirmation (no PAN at our backend per PCI SAQ-A).
    - For vietqr-rail (`billing_currency = VND`): `{ vnpay_redirect_url: "https://...", reference: "..." }` — actual VnPay integration ships in FR-TEN-102 (placeholder; for slice 1 of TEN-101 the VND path returns `503 SERVICE_UNAVAILABLE` + `{ error: "vnd_signup_requires_ten_102" }` until FR-TEN-102 lands).

14. **MUST** expose `POST /v1/signup/complete` — the commit-or-rollback orchestrator. Body: `{ signup_session_id, tenant_slug, tenant_display_name, plan_tier, billing_currency, billing_contact_email, stripe_setup_intent_id?, vnpay_reference?, consents: [{kind, version, locale}], turnstile_token }`. The handler runs in three phases — **pre-tx validation** (no DB writes), **external Stripe call** (no DB tx held), then a **single Postgres transaction** that atomically writes consents + tenant + root-admin + session-state. Handler MUST in strict order:
    - **Phase A — pre-tx validation:**
      1. Re-verify Turnstile token; reject if missing/invalid.
      2. Verify `signup_sessions.state == 'email_verified'`; else 409.
      3. Validate plan_tier ∈ {Starter, Team}; reject Enterprise with 403.
      4. Validate consents: ToS + Privacy MUST be present at the latest published versions; Marketing OPTIONAL; for VN tenants, `pdpl_vn_data_processing` MUST be present.
      5. Check duplicate-email guard (per DEC-842): if `tenants` already has a row keyed by `billing_contact_email_hash16 = HMAC(global_salt, email_lower)` AND `status='active'`, return 409 + `{ error: "email_already_associated", magic_link_sent: true }` AND email a magic-link sign-in to the existing tenant; emit `ten.signup_abandoned` with reason='duplicate_email'. The hash16 index on `tenants.billing_contact_email_hash16` is created by this FR's migration `0011` ALTER statement.
    - **Phase B — external Stripe call (outside DB tx to avoid long-held connections):**
      6. For stripe-rail: confirm the SetupIntent via Stripe API (`POST /v1/setup_intents/{id}/confirm`); on failure, transition `signup_sessions.state='abandoned'` + `abandon_reason='payment_failed'` + return 402 PAYMENT_REQUIRED. The Stripe SetupIntent ID is recorded in `signup_sessions.payment_captured_at` BEFORE the DB tx opens so any subsequent failure can void it (DEC-843 derivative).
    - **Phase C — atomic provisioning transaction:**
      7. `BEGIN`; persist consent rows.
      8. Invoke FR-TEN-001 `provision_tenant(slug, currency, residency, billing_contact_email)` inside the tx; on collision (lost race on slug) `ROLLBACK` + void Stripe SetupIntent + return 409 + suggestions; transition `signup_sessions.state='abandoned'` reason='slug_collision'.
      9. Create root-admin subject via FR-AUTH-002 inside the same tx with auto-generated 24-char password (DEC-834) OR no password if OIDC path (DEC-835); on failure `ROLLBACK` + void Stripe SetupIntent (+ no tenant exists post-rollback — FR-TEN-104 NOT invoked).
      10. Back-fill `tenant_consents.tenant_id` + `subject_id`.
      11. `UPDATE signup_sessions SET state='completed', tenant_id=...` + `COMMIT`.
    - **Phase D — post-commit side effects (failures here do not undo commit):**
      12. Invoke FR-TEN-003 `ensure_customer` + `ensure_subscription(plan_tier)` with the captured payment method as default. On Stripe failure post-commit: log sev-1 + leave tenant in `dunning_state='retry_1'` (FR-TEN-003 §1 #11 picks up the recovery).
      13. Mint first-login JWT via FR-AUTH-004 with `signup_session_id` claim + standard claims.
      14. Set a Secure HttpOnly SameSite=Lax `cyberos_jwt` cookie scoped to `*.cyberos.world` so the redirect to `<slug>.cyberos.world/onboarding/welcome` arrives authenticated.
      15. Emit `ten.signup_completed` memory row.
      16. Trigger welcome email via `welcome_email.rs` with the 7d-TTL deeplink (DEC-845).
      17. Return `201 CREATED` with `{ tenant_id, tenant_slug, jwt, redirect_url: "https://<slug>.cyberos.world/onboarding/welcome" }`.

15. **MUST** support the OIDC sign-up path (DEC-835). Flow:
    1. UI offers "Sign up with Google" / "Sign up with Microsoft" buttons (delegating to FR-AUTH-104 OIDC SSO).
    2. Before redirecting to the IdP, the UI MUST call `POST /v1/signup/oidc-init` with `{ signup_session_id, idp: "google"|"microsoft" }`; the handler binds the session_id to a single-use OIDC `state` nonce stored server-side with 5-min TTL + emits the IdP authorize URL with that nonce.
    3. OIDC return URL is `/v1/signup/oidc-callback`; the handler validates the `state` nonce against the server-stored binding (CSRF defense), extracts `email` + `email_verified` + `name` from the ID token.
    4. If `email_verified=true` (Google/Microsoft IdPs both verify), skip the OTP step entirely — transition `signup_sessions.state` directly to `email_verified`.
    5. Suggest tenant_slug from email domain.
    6. Continue to plan selection + payment + complete (same orchestrator as #14 but with OIDC marker on the session row).
    7. Root-admin subject is created without a password; subsequent logins are OIDC-only.
    8. Emit `ten.signup_oidc_linked` memory row.

16. **MUST** persist consent rows BEFORE provisioning (per DEC-832, #14 step 4). The `tenant_consents` table is INSERT'd within the same transaction as the `tenants` row creation, ensuring no orphan tenant-without-consent state. Consent versions are pinned to the strings displayed to the user at signup-time — fetched from `services/ten/web/signup/consents/<locale>/<kind>-v<n>.md` static files.

17. **MUST** emit the 30-second SLI to OBS at `signup_completed` (per DEC-828). The OTel histogram metric `ten_signup_duration_seconds` is recorded with attributes `(plan_tier, billing_currency, oidc: bool, completed: bool)`. Alarm sev-2 when `p95 > 30s sustained 15 min`; sev-1 when `p99 > 60s sustained 5 min`.

18. **MUST** detect squatting per DEC-844. The `abuse_guard.rs` module tracks `(ip_addr, started_signups_24h, abandoned_signups_24h)` in Redis. When `abandoned_signups_24h ≥ 5 AND completed_signups_24h = 0`, block the IP for 24h (`signup_start` returns `429 + { error: "abuse_detected_24h_cooloff" }`). The block auto-expires; legitimate users from shared IPs (corp NAT) can request manual unblock via support email.

19. **MUST** emit 9 **core** memory audit row kinds tied to user-visible signup lifecycle (DEC-839 + feature-request-audit skill rule 6 namespace pattern):
    - `ten.signup_started` (sev-3 informational)
    - `ten.signup_email_verified` (sev-3)
    - `ten.signup_consent_recorded` (sev-2 — legal)
    - `ten.signup_tenant_provisioned` (sev-2 — material commercial event)
    - `ten.signup_completed` (sev-2)
    - `ten.signup_abandoned` (sev-3 — funnel analytics)
    - `ten.signup_rate_limited` (sev-3 — security signal)
    - `ten.signup_disposable_email_blocked` (sev-3)
    - `ten.signup_oidc_linked` (sev-3)

    Plus **2 supporting** ops-only kinds (not in the core 9 because they're system-emitted, not user-action-triggered):
    - `ten.disposable_email_blocklist_refreshed` (sev-3 — emitted by monthly refresh job; payload = `domains_added`, `domains_removed`, `source_sha`)
    - `ten.signup_session_scrubbed` (sev-3 — emitted by daily 90d-PII-scrub job; payload = `scrubbed_count`)

    Every row PII-scrubs `email_full` via FR-MEMORY-111 → `email_hash16`; raw email retained in tenant Postgres (RLS-scoped) only until `signup_sessions.scrubbed_at` is set at 90d.

20. **MUST** thread W3C `traceparent` across the entire flow (feature-request-audit skill rule 22 + 23 + 24). The `signup_session_id` is logged as a span attribute on every span; a single `trace_id` in the response makes support escalations resolvable.

21. **MUST** scrub PII at 90 days per DEC-840. Daily scheduled job updates `signup_sessions WHERE started_at < now() - interval '90 days' AND scrubbed_at IS NULL`: clear `email_full`, retain `email_hash16` + funnel state. The scrub itself is recorded in memory via `ten.signup_session_scrubbed` (kind not in the 9-kind core list per DEC-839 — informational only).

22. **MUST** support the VN locale at every consent UI step per DEC-833. The Vietnamese-language ToS / Privacy variant is at `services/ten/web/signup/consents/vi/*.md`; the PDPL-VN consent kind `pdpl_vn_data_processing` is required for `billing_currency='VND'` (residency vn-1).

23. **MUST** rollback cleanly on any orchestrator step failure. The rollback path:
    - If Stripe authorization succeeded but TEN-001 provisioning failed: Stripe Subscription is cancelled (`DELETE /v1/subscriptions/{id}`); SetupIntent is voided; signup_sessions.state='rolled_back' + rolled_back_reason='provisioning_failed'.
    - If TEN-001 provisioning succeeded but AUTH root-admin failed: invoke FR-TEN-104 hard-terminate on the just-created tenant (rare path); signup_sessions.state='rolled_back'.
    - Every rollback emits a `ten.signup_abandoned` memory row with `abandon_reason` populated.

24. **MUST** rate-limit per `(IP, email_hash16)` pair NOT per-(IP, email_full) (per DEC-840 derivative — avoid storing raw email in rate-limit keys for privacy). The hash16 form is sufficient for de-duplication at the rate-limit boundary.

25. **MUST** be idempotent on `signup_session_id` per DEC-829. A signup that crashes mid-flow can be resumed: the client reuses the same session_id and the handler returns the current state. Retrying `signup_complete` with the same session_id after a successful complete returns 200 OK with the existing tenant_id (idempotent acknowledgement).

26. **MUST NOT** persist plaintext passwords or plaintext OTPs anywhere. OTPs are HMAC-SHA256-hashed under a server-side secret before Redis insert. Auto-generated root-admin passwords are bcrypt-hashed in `subjects` (FR-AUTH-002) and emailed once via FR-EMAIL-001 (DEC-834).

27. **MUST** be RLS-protected on `signup_sessions` and `tenant_consents`. Pre-tenant-creation rows in `signup_sessions` use a system-tenant scope (`current_setting('auth.signup_system_tenant_id')` — the well-known UUID `00000000-0000-0000-0000-000000000001`); post-tenant-creation rows back-fill `tenant_id` and the RLS policy switches to the real tenant scope. The dual-scope policy is `USING (signup_session_id IS NOT NULL AND (tenant_id IS NULL OR tenant_id = current_setting('auth.tenant_id')::uuid)) WITH CHECK (...same...)`.

---

## §2 — Why this design (rationale for humans)

**Why a single orchestrator (`complete.rs`) rather than 5 independent micro-endpoints (§1 #14, DEC-827)?** The 30-second SLO is the design driver. A multi-step async flow with persisted state between steps would require the user to wait through 4 page reloads (or polling), losing 5-10 seconds each. The single `complete` endpoint chains Stripe → TEN-001 → AUTH → JWT in one server-side call with parallel where possible (e.g., consent INSERTs run in parallel with Stripe confirm). Total wall-clock cost ≈ 3-5 seconds for the server work + ≈ 5-15 seconds for the user filling the form ≈ 30s budget hit.

**Why payment BEFORE provisioning (§1 #14 step 7, DEC-843)?** Two reasons. First, commercial: tenants without payment are not customers — they're cost. Provisioning a tenant before payment confirmation means we provision-then-charge, which fails 5-15% of the time (declined cards) and leaves orphan tenants that auto-suspend after dunning. Second, anti-abuse: requiring payment authorization filters out 99% of bot signups (bots don't have valid cards; even stolen-card bots get filtered by Stripe Radar). The cost is a tighter rollback (Stripe SetupIntent void on tenant-create failure) but rollback is rare and Stripe void is fast.

**Why HMAC-hash OTPs in Redis rather than plaintext (§1 #1, DEC-821)?** Defense-in-depth. Redis snapshots / replicas / log-shipping can leak data. Plaintext OTPs in any of those is a credential leak vector. HMAC-hashing under a server-side secret means a Redis dump doesn't yield usable OTPs. The HMAC key rotation policy is quarterly per ops standards.

**Why GeoIP-derived currency rather than user-input only (§1 #11, DEC-824)?** Two reasons. First, default-to-local is the conventional UX — a Vietnamese user shouldn't see USD prices first. Second, residency is downstream of currency (DEC-825); GeoIP-default-then-override is faster than asking the user a separate residency question. The user retains the override path for legitimate cases (e.g., a SG-based finance team wants USD billing for parent-company consolidation).

**Why disposable-email blocklist (DEC-830) rather than just rate limits?** Rate limits stop one-bot-per-IP; disposable-email blocklists stop one-bot-per-IP-per-throwaway-domain. Without it, a bot rotates `temp-mail.org / 10minutemail.com / mailinator.com` addresses faster than the rate limit blocks. The blocklist is high-signal because legitimate users rarely use disposable domains (and when they do — e.g., privacy-conscious users — the "use work email" suggestion is reasonable for a paid B2B SaaS).

**Why 30-second SLO (§1 #17, DEC-828)?** Industry data (Baymard Institute, Optimizely studies) shows conversion drops 30%+ when forms exceed 30 seconds. Self-serve signup is the top of the commercial funnel; a 30% drop here translates to ~30% lower ARR. The SLO is treated as a commercial commitment, not just a tech metric.

**Why OIDC sign-up path (§1 #15, DEC-835)?** "Sign in with Google/Microsoft" cuts signup time by ~15 seconds (no OTP step, no password creation, no email verification round-trip) and improves trust signals (delegating identity to a known IdP). The cost is a future-pivot risk if the user later wants password fallback — handled by the tenant_admin "enable password" toggle (post-signup).

**Why same-email duplicate guard (§1 #14 step 5, DEC-842)?** A user who already has a tenant and tries to sign up again (forgot they have one, or trying to consolidate billing) gets a magic-link to the existing tenant, NOT a second tenant. Multiple tenants per person is a support nightmare (which one is the "real" one?) and a billing nightmare (which one gets billed?). The magic-link UX is the right escape valve.

**Why squatting detection (§1 #18, DEC-844)?** Bots that scrape signup forms often abandon at payment (no valid card). 5 abandoned signups from one IP in 24h is statistically improbable for humans (~10⁻⁴ in our observed funnel data, mostly families with multiple businesses). 24h cool-off + manual support unblock for false positives is the right trade.

**Why versioned consents (§1 #16, DEC-832)?** Regulators ask "what did this user agree to?" The answer needs a version string + a way to fetch the exact text. Storing `version: "tos-v2.3"` + a static `tos-v2.3.md` file in source means we can answer that question with bit-exact text years later. ToS/Privacy version bumps require a consent-re-acceptance flow (out-of-scope here; FR-TEN-1xx).

**Why dual-scope RLS on signup_sessions (§1 #27)?** Pre-tenant signups have no `tenant_id` (no tenant exists yet), but the row must be RLS-scoped — without scoping, anonymous users could enumerate all in-flight signups via the API. The "system tenant" UUID `00000000-0000-0000-0000-000000000001` is a well-known sentinel that the public signup handlers set via `SET LOCAL auth.tenant_id` at request entry; post-completion the row's `tenant_id` is back-filled and the policy transparently transitions to the real tenant.

---

## §3 — API contract

### 3.1 Postgres schema (migrations)

```sql
-- 0011_signup_sessions.sql
-- First adds the billing_contact_email_hash16 column + index to tenants for the duplicate-email guard (§1 #14 step 5):
ALTER TABLE tenants
  ADD COLUMN billing_contact_email_hash16 TEXT GENERATED ALWAYS AS
    (encode(substring(digest(coalesce(lower(billing_contact_email), ''), 'sha256') from 1 for 8), 'hex'))
    STORED;
CREATE INDEX idx_tenants_billing_contact_email_hash16
  ON tenants(billing_contact_email_hash16) WHERE status = 'active';

CREATE TABLE signup_sessions (
  signup_session_id UUID PRIMARY KEY,
  email_hash16 TEXT NOT NULL,
  email_full TEXT,                                 -- nullable after 90d scrub
  geoip_country CHAR(2),
  suggested_billing_currency billing_currency_enum,
  suggested_residency TEXT,
  state TEXT NOT NULL DEFAULT 'started'
    CHECK (state IN ('started','email_verified','payment_captured','provisioning','completed','abandoned','rolled_back')),
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  email_verified_at TIMESTAMPTZ,
  payment_captured_at TIMESTAMPTZ,
  provisioned_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  abandon_reason TEXT,
  rolled_back_reason TEXT,
  tenant_id UUID,
  ip_addr_hash16 TEXT,
  user_agent_hash16 TEXT,
  scrubbed_at TIMESTAMPTZ
);
CREATE INDEX idx_signup_sessions_email_hash16 ON signup_sessions(email_hash16);
CREATE INDEX idx_signup_sessions_started_at ON signup_sessions(started_at);
ALTER TABLE signup_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY signup_sessions_rls ON signup_sessions
  USING (
    tenant_id IS NULL AND current_setting('auth.tenant_id', true) = '00000000-0000-0000-0000-000000000001'
    OR tenant_id = NULLIF(current_setting('auth.tenant_id', true), '')::uuid
  )
  WITH CHECK (
    tenant_id IS NULL AND current_setting('auth.tenant_id', true) = '00000000-0000-0000-0000-000000000001'
    OR tenant_id = NULLIF(current_setting('auth.tenant_id', true), '')::uuid
  );
REVOKE UPDATE, DELETE ON signup_sessions FROM cyberos_app;
-- Specific UPDATE grants for state transitions:
GRANT UPDATE (state, email_verified_at, payment_captured_at, provisioned_at, completed_at,
              abandon_reason, rolled_back_reason, tenant_id, scrubbed_at, email_full)
  ON signup_sessions TO cyberos_app;

-- 0012_tenant_consents.sql
CREATE TABLE tenant_consents (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID,                                  -- nullable pre-provisioning
  signup_session_id UUID NOT NULL,
  subject_id UUID,                                  -- nullable until root-admin created
  consent_kind TEXT NOT NULL
    CHECK (consent_kind IN ('tos','privacy','marketing','pdpl_vn_data_processing',
                            'pdpl_vn_data_export','gdpr_legitimate_interest')),
  version TEXT NOT NULL,
  accepted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ip_addr_hash16 TEXT NOT NULL,
  locale CHAR(5) NOT NULL,
  withdrawn_at TIMESTAMPTZ,
  withdrawn_reason TEXT
);
CREATE INDEX idx_tenant_consents_tenant ON tenant_consents(tenant_id);
CREATE INDEX idx_tenant_consents_signup ON tenant_consents(signup_session_id);
ALTER TABLE tenant_consents ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_consents_rls ON tenant_consents
  USING (tenant_id IS NULL OR tenant_id = NULLIF(current_setting('auth.tenant_id', true), '')::uuid)
  WITH CHECK (tenant_id IS NULL OR tenant_id = NULLIF(current_setting('auth.tenant_id', true), '')::uuid);
REVOKE UPDATE, DELETE ON tenant_consents FROM cyberos_app;
-- Withdrawal is the only permitted update; column-level GRANT to consent_writer role:
GRANT UPDATE (withdrawn_at, withdrawn_reason) ON tenant_consents TO cyberos_consent_writer;

-- 0013_signup_rate_limits.sql
-- The hot counters live in Redis sliding-window (sub-millisecond reads needed at the public edge).
-- This table is a forensic journal — every rate-limit HIT is dual-logged here for compliance
-- analytics (e.g., "how many signup attempts were blocked in EU-region this quarter?"). Misses are NOT logged.
CREATE TABLE signup_rate_limit_journal (
  id BIGSERIAL PRIMARY KEY,
  guard TEXT NOT NULL CHECK (guard IN ('signup_start','otp_send','slug_check','signup_complete')),
  key_kind TEXT NOT NULL CHECK (key_kind IN ('ip','email_hash16','ip_email')),
  key_value TEXT NOT NULL,
  ts TIMESTAMPTZ NOT NULL DEFAULT now(),
  retry_after_seconds INT NOT NULL
);
ALTER TABLE signup_rate_limit_journal ENABLE ROW LEVEL SECURITY;
CREATE POLICY signup_rate_limit_journal_rls ON signup_rate_limit_journal
  USING (current_setting('auth.tenant_id', true) = '00000000-0000-0000-0000-000000000001')
  WITH CHECK (current_setting('auth.tenant_id', true) = '00000000-0000-0000-0000-000000000001');
REVOKE UPDATE, DELETE ON signup_rate_limit_journal FROM cyberos_app;

-- 0014_disposable_email_domains.sql
CREATE TABLE disposable_email_domains (
  domain TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  removed_at TIMESTAMPTZ
);
CREATE TABLE disposable_email_blocklist_refresh (
  id BIGSERIAL PRIMARY KEY,
  refreshed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  source_sha CHAR(64) NOT NULL,
  domains_added INT NOT NULL DEFAULT 0,
  domains_removed INT NOT NULL DEFAULT 0
);
REVOKE UPDATE, DELETE ON disposable_email_domains FROM cyberos_app;
REVOKE UPDATE, DELETE ON disposable_email_blocklist_refresh FROM cyberos_app;
GRANT INSERT, DELETE ON disposable_email_domains TO cyberos_blocklist_updater;
```

### 3.2 Rust types

```rust
// services/ten/src/signup/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum SignupState {
    Started,
    EmailVerified,
    PaymentCaptured,
    Provisioning,
    Completed,
    Abandoned,
    RolledBack,
}

#[derive(serde::Deserialize)]
pub struct SignupStartReq {
    pub email: String,
    pub signup_session_id: uuid::Uuid,  // UUIDv7 client-generated
    pub turnstile_token: String,
    pub locale_hint: Option<String>,  // e.g., "vi-VN"
}

#[derive(serde::Serialize)]
pub struct SignupStartResp {
    pub signup_session_id: uuid::Uuid,
    pub email_verification_required: bool,
    pub suggested_billing_currency: BillingCurrency,
    pub suggested_residency: Residency,
    pub geoip_country: Option<String>,
    pub otp_sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub breach_warning: Option<BreachWarning>,
}

#[derive(serde::Deserialize)]
pub struct SignupCompleteReq {
    pub signup_session_id: uuid::Uuid,
    pub tenant_slug: String,
    pub tenant_display_name: String,
    pub plan_tier: PlanTier,
    pub billing_currency: BillingCurrency,
    pub billing_contact_email: String,
    pub stripe_setup_intent_id: Option<String>,
    pub vnpay_reference: Option<String>,
    pub consents: Vec<ConsentEntry>,
    pub turnstile_token: String,
}

#[derive(serde::Deserialize)]
pub struct ConsentEntry {
    pub kind: ConsentKind,
    pub version: String,
    pub locale: String,
}
```

### 3.3 REST endpoints

```text
POST   /v1/signup/start                  (public)
POST   /v1/signup/verify-otp             (public)
GET    /v1/signup/slug-available         (public)
POST   /v1/signup/payment-intent         (public, session-bound)
POST   /v1/signup/complete               (public, session-bound)
GET    /v1/signup/oidc-callback          (public, FR-AUTH-104 return)
```

---

## §4 — Acceptance criteria

1. **End-to-end happy path under 30 s** — fixture-based happy test (`signup_30s_sla_test`) completes start→OTP→slug→plan→payment→complete in < 30 s including Stripe SetupIntent confirm (against test-mode).
2. **OTP HMAC-hashed in Redis** — direct Redis inspection during test shows no plaintext OTP; only HMAC-SHA256 digests.
3. **OTP max attempts enforced** — 6 failed verify-otp calls in row return `429 + { error: "otp_max_attempts" }`; 7th requires new OTP send.
4. **Disposable email blocked** — signup_start with `email=foo@10minutemail.com` returns 400 + `{ error: "disposable_email" }` and emits `ten.signup_disposable_email_blocked`.
5. **Rate limits enforced** — 11th signup_start from same IP within 60 s returns 429.
6. **Slug uniqueness** — slug-available for an existing slug returns `{available: false}` + 50 suggestions; race-on-complete returns 409.
7. **GeoIP→currency derivation** — VN-IP signup defaults to VND/vn-1; SG-IP to SGD/sg-1; EU-IP to EUR/eu-1; user override accepted.
8. **Enterprise self-serve blocked** — signup_complete with `plan_tier=enterprise` returns 403.
9. **Consent versioned + required** — signup_complete without ToS+Privacy consents returns 400; with VND currency missing PDPL VN consent returns 400.
10. **Stripe rollback on TEN-001 failure** — when TEN-001 provisioning fails after Stripe confirm, the Stripe Subscription is cancelled + signup_sessions.state='rolled_back'.
11. **Duplicate email returns magic link** — signup_complete with an already-active tenant's billing_contact_email returns 409 + magic-link email sent; no second tenant created.
12. **OIDC path skips OTP** — `/v1/signup/oidc-callback` with verified ID token transitions session directly to `email_verified`.
13. **Squatting block after 5 abandons** — 5 abandoned signups from one IP in 24h → 6th signup_start returns 429 `abuse_detected_24h_cooloff`.
14. **PII scrubbed at 90 days** — fixture signup with started_at 91 days ago + run scrub job → email_full = NULL, email_hash16 retained, memory row `ten.signup_session_scrubbed` emitted.
15. **9 memory audit kinds emitted** — happy path produces `signup_started + signup_email_verified + signup_consent_recorded + signup_tenant_provisioned + signup_completed` (5 of 9); failure paths produce the others.
16. **W3C traceparent threaded** — single trace_id present in start response + complete response + all 9 memory rows.
17. **Idempotent on session_id** — `signup_complete` invoked twice with same session_id post-completion returns 200 + existing tenant_id (not 201 + new).
18. **Welcome email + 7d deeplink** — completed signup triggers email with deeplink to `<slug>.cyberos.world/onboarding/welcome`; deeplink works for 7 days then 404.
19. **RLS dual-scope** — pre-tenant rows visible only to `system_signup_tenant`; post-tenant rows visible only to the tenant's RLS scope.
20. **Stripe SetupIntent only — no PAN at our backend** — request-body inspection confirms no `card_number` field in any request to our endpoints.

---

## §5 — Verification

### 5.1 `signup_happy_test.rs`

```rust
#[tokio::test]
async fn signup_30s_e2e_happy() {
    let ctx = TestContext::new().await;
    let session = uuid::Uuid::now_v7();
    let start = Instant::now();

    // 1. start
    let r1 = ctx.post("/v1/signup/start").json(&SignupStartReq{
        email: "alice@acme.com".into(), signup_session_id: session,
        turnstile_token: ctx.test_turnstile_token(), locale_hint: Some("en-US".into()),
    }).send().await.unwrap();
    assert_eq!(r1.status(), 200);

    let otp = ctx.read_test_otp_for(session).await;

    // 2. verify-otp
    let r2 = ctx.post("/v1/signup/verify-otp").json(&serde_json::json!({
        "signup_session_id": session, "otp_code": otp
    })).send().await.unwrap();
    assert_eq!(r2.status(), 200);

    // 3. slug-available
    let r3 = ctx.get(&format!("/v1/signup/slug-available?candidate=acme-{session}"))
        .send().await.unwrap();
    let avail: serde_json::Value = r3.json().await.unwrap();
    assert_eq!(avail["available"], true);

    // 4. payment-intent
    let r4 = ctx.post("/v1/signup/payment-intent").json(&serde_json::json!({
        "signup_session_id": session, "plan_tier": "team", "billing_currency": "USD"
    })).send().await.unwrap();
    let setup_intent_id = ctx.stripe_test_confirm(r4.json::<serde_json::Value>().await.unwrap()).await;

    // 5. complete
    let r5 = ctx.post("/v1/signup/complete").json(&SignupCompleteReq{
        signup_session_id: session,
        tenant_slug: format!("acme-{}", &session.to_string()[..8]),
        tenant_display_name: "Acme Inc.".into(),
        plan_tier: PlanTier::Team,
        billing_currency: BillingCurrency::Usd,
        billing_contact_email: "alice@acme.com".into(),
        stripe_setup_intent_id: Some(setup_intent_id),
        vnpay_reference: None,
        consents: vec![
            ConsentEntry{kind: ConsentKind::Tos, version: "v2.3".into(), locale: "en-US".into()},
            ConsentEntry{kind: ConsentKind::Privacy, version: "v2.1".into(), locale: "en-US".into()},
        ],
        turnstile_token: ctx.test_turnstile_token(),
    }).send().await.unwrap();

    assert_eq!(r5.status(), 201);
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(30), "actual {:?}", elapsed);
}
```

### 5.2 `signup_otp_test.rs`

```rust
#[tokio::test]
async fn otp_stored_hmac_only() {
    let ctx = TestContext::new().await;
    let session = start_signup(&ctx, "alice@acme.com").await;
    let mut redis = ctx.redis_conn();
    let stored: String = redis.get(&format!("signup_otp:{session}")).await.unwrap();
    assert!(!stored.contains("000000"));  // no plaintext
    assert_eq!(stored.len(), 64);          // 256-bit hex digest
}

#[tokio::test]
async fn otp_max_attempts_locks() {
    let ctx = TestContext::new().await;
    let session = start_signup(&ctx, "alice@acme.com").await;
    for _ in 0..5 {
        let r = ctx.post("/v1/signup/verify-otp").json(&json!({
            "signup_session_id": session, "otp_code": "999999"
        })).send().await.unwrap();
        assert_eq!(r.status(), 400);
    }
    let r = ctx.post("/v1/signup/verify-otp").json(&json!({
        "signup_session_id": session, "otp_code": "999999"
    })).send().await.unwrap();
    assert_eq!(r.status(), 429);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "otp_max_attempts");
}
```

### 5.3 `signup_disposable_email_test.rs`

```rust
#[tokio::test]
async fn disposable_domain_blocked() {
    let ctx = TestContext::new().await;
    let r = ctx.post("/v1/signup/start").json(&SignupStartReq{
        email: "test@10minutemail.com".into(),
        signup_session_id: uuid::Uuid::now_v7(),
        turnstile_token: ctx.test_turnstile_token(),
        locale_hint: None,
    }).send().await.unwrap();
    assert_eq!(r.status(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "disposable_email");

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "ten.signup_disposable_email_blocked"));
}
```

### 5.4 `signup_rate_limit_test.rs`

```rust
#[tokio::test]
async fn signup_start_rate_limit() {
    let ctx = TestContext::new().await;
    for i in 0..10 {
        let r = ctx.post("/v1/signup/start").json(&SignupStartReq{
            email: format!("alice{i}@acme.com"),
            signup_session_id: uuid::Uuid::now_v7(),
            turnstile_token: ctx.test_turnstile_token(),
            locale_hint: None,
        }).send().await.unwrap();
        assert_eq!(r.status(), 200);
    }
    let r = ctx.post("/v1/signup/start").json(&SignupStartReq{
        email: "alice11@acme.com".into(),
        signup_session_id: uuid::Uuid::now_v7(),
        turnstile_token: ctx.test_turnstile_token(),
        locale_hint: None,
    }).send().await.unwrap();
    assert_eq!(r.status(), 429);
}
```

### 5.5 `signup_stripe_rollback_test.rs`

```rust
#[tokio::test]
async fn stripe_confirmed_then_provisioning_fails_rolls_back() {
    let ctx = TestContext::new().await;
    let session = email_verified_session(&ctx).await;
    let setup_intent = ctx.stripe_test_setup_intent_confirmed().await;
    ctx.force_provisioning_failure();

    let r = ctx.post("/v1/signup/complete").json(&minimal_complete_body(session, setup_intent)).send().await.unwrap();
    assert_eq!(r.status(), 500);

    let stripe_subscriptions = ctx.stripe_test_list_subscriptions_for_session(session).await;
    assert!(stripe_subscriptions.iter().all(|s| s.status == "canceled"));

    let state: String = sqlx::query_scalar("SELECT state FROM signup_sessions WHERE signup_session_id=$1")
        .bind(session).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state, "rolled_back");
}
```

### 5.6 `signup_duplicate_email_test.rs`

```rust
#[tokio::test]
async fn second_signup_same_email_returns_magic_link() {
    let ctx = TestContext::new().await;
    let _first = complete_signup(&ctx, "alice@acme.com", "acme-1").await;
    let session2 = email_verified_session_with(&ctx, "alice@acme.com").await;

    let r = ctx.post("/v1/signup/complete").json(&minimal_complete_body(session2, ctx.stripe_test_setup_intent_confirmed().await))
        .send().await.unwrap();
    assert_eq!(r.status(), 409);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "email_already_associated");
    assert_eq!(body["magic_link_sent"], true);

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tenants WHERE billing_contact_email='alice@acme.com'"
    ).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 1);
}
```

### 5.7 `signup_squatting_test.rs`

```rust
#[tokio::test]
async fn five_abandoned_signups_blocks_ip_24h() {
    let ctx = TestContext::with_ip("198.51.100.42").await;
    for i in 0..5 {
        let s = uuid::Uuid::now_v7();
        ctx.post("/v1/signup/start").json(&start_body(s, format!("a{i}@acme.com")))
            .send().await.unwrap();
        ctx.tick_abandon(s).await;
    }
    let r = ctx.post("/v1/signup/start").json(&start_body(uuid::Uuid::now_v7(), "a99@acme.com".into()))
        .send().await.unwrap();
    assert_eq!(r.status(), 429);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "abuse_detected_24h_cooloff");
}
```

### 5.8 `signup_consent_versioning_test.rs`

```rust
#[tokio::test]
async fn vn_tenant_requires_pdpl_consent() {
    let ctx = TestContext::with_geoip_country("VN").await;
    let session = email_verified_session(&ctx).await;
    let setup_intent = ctx.vnpay_reference_confirmed().await;

    let r = ctx.post("/v1/signup/complete").json(&SignupCompleteReq{
        signup_session_id: session,
        billing_currency: BillingCurrency::Vnd,
        consents: vec![
            ConsentEntry{kind: ConsentKind::Tos, version: "v2.3".into(), locale: "vi-VN".into()},
            ConsentEntry{kind: ConsentKind::Privacy, version: "v2.1".into(), locale: "vi-VN".into()},
            // PDPL VN missing
        ],
        ..minimal_complete_body(session, setup_intent)
    }).send().await.unwrap();

    assert_eq!(r.status(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "missing_pdpl_vn_consent");
}
```

### 5.9 `signup_oidc_path_test.rs`

```rust
#[tokio::test]
async fn oidc_signup_skips_otp() {
    let ctx = TestContext::new().await;
    let session = uuid::Uuid::now_v7();
    let oidc_id_token = ctx.fake_google_id_token("alice@acme.com", true).await;

    let r = ctx.get(&format!("/v1/signup/oidc-callback?id_token={oidc_id_token}&signup_session_id={session}"))
        .send().await.unwrap();
    assert_eq!(r.status(), 200);

    let state: String = sqlx::query_scalar("SELECT state FROM signup_sessions WHERE signup_session_id=$1")
        .bind(session).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(state, "email_verified");

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "ten.signup_oidc_linked"));
}
```

### 5.10 `signup_30s_sla_test.rs`

```rust
#[tokio::test]
async fn signup_completes_under_30s_p95() {
    let ctx = TestContext::new().await;
    let mut durations = Vec::new();
    for i in 0..20 {
        let start = Instant::now();
        complete_signup(&ctx, &format!("u{i}@acme.com"), &format!("acme-{i}")).await;
        durations.push(start.elapsed());
    }
    durations.sort();
    let p95 = durations[(durations.len() as f64 * 0.95) as usize];
    assert!(p95 < Duration::from_secs(30), "p95 actual {:?}", p95);
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton. Additional orchestrator wiring below.)

### 6.1 `complete.rs` commit-or-rollback orchestrator

```rust
pub async fn signup_complete(ctx: &AppCtx, req: SignupCompleteReq, cookies: &mut Cookies) -> Result<SignupCompleteResp, SignupError> {
    // ───── Phase A: pre-tx validation (no DB writes) ─────
    ctx.turnstile.verify(&req.turnstile_token, &req.signup_session_id).await?;

    let session = ctx.repo.signup_sessions.get(req.signup_session_id).await?;
    if session.state != SignupState::EmailVerified {
        return Err(SignupError::InvalidState(session.state));
    }
    if req.plan_tier == PlanTier::Enterprise {
        return Err(SignupError::EnterpriseSelfServeBlocked);
    }
    validate_consents(&req.consents, req.billing_currency)?;

    if let Some(existing) = ctx.repo.tenants.find_active_by_email_hash16(&hash16_email(&req.billing_contact_email)).await? {
        send_magic_link(&ctx, &req.billing_contact_email, existing.id).await;
        emit_audit(&ctx, "ten.signup_abandoned", json!({"reason": "duplicate_email"})).await;
        return Err(SignupError::EmailAlreadyAssociated);
    }

    // ───── Phase B: external Stripe call (NO DB tx held) ─────
    if req.billing_currency != BillingCurrency::Vnd {
        let intent_id = req.stripe_setup_intent_id.as_ref().ok_or(SignupError::MissingPaymentIntent)?;
        ctx.stripe.confirm_setup_intent(intent_id).await.map_err(SignupError::PaymentFailed)?;
        ctx.repo.signup_sessions.mark_payment_captured(req.signup_session_id, intent_id).await?;
    }

    // ───── Phase C: atomic provisioning transaction ─────
    let provisioning = async {
        let mut tx = ctx.pool.begin().await?;
        for c in &req.consents {
            ctx.repo.consents.insert_pending(&mut tx, session.signup_session_id, c, &req.tenant_slug).await?;
        }
        let tenant_id = ctx.ten.provision_tenant(&mut tx, ProvisionReq{
            slug: req.tenant_slug.clone(),
            display_name: req.tenant_display_name.clone(),
            billing_currency: req.billing_currency,
            billing_contact_email: req.billing_contact_email.clone(),
            residency: derive_residency(req.billing_currency),
        }).await?;
        let root_admin = ctx.auth.create_subject_with_role(&mut tx, tenant_id, &req.billing_contact_email,
            SubjectRole::TenantAdmin, GeneratedPassword::Auto24).await?;
        ctx.repo.consents.back_fill_tenant_id(&mut tx, session.signup_session_id, tenant_id, root_admin.id).await?;
        ctx.repo.signup_sessions.transition_completed(&mut tx, session.signup_session_id, tenant_id).await?;
        tx.commit().await?;
        Ok::<(uuid::Uuid, SubjectRecord), SignupError>((tenant_id, root_admin))
    }.await;

    let (tenant_id, root_admin) = match provisioning {
        Ok(t) => t,
        Err(e) => {
            // Phase-C failure: void Stripe + mark session rolled back
            if let Some(intent_id) = &req.stripe_setup_intent_id {
                ctx.stripe.void_setup_intent(intent_id).await.ok();
            }
            let reason = match &e {
                SignupError::SlugCollision(_) => "slug_collision",
                _ => "provisioning_failed",
            };
            mark_rolled_back(&ctx, session.signup_session_id, reason).await?;
            return Err(e);
        }
    };

    // ───── Phase D: post-commit side effects (failures here do NOT undo the tenant) ─────
    if req.billing_currency != BillingCurrency::Vnd {
        if let Err(e) = ctx.ten.billing_stripe.ensure_customer(tenant_id).await {
            tracing::error!(?e, %tenant_id, "ensure_customer failed post-commit; tenant lands in dunning_state=retry_1");
            ctx.repo.tenants.set_dunning_state(tenant_id, DunningState::Retry1).await?;
        } else {
            ctx.ten.billing_stripe.ensure_subscription(tenant_id, req.plan_tier).await.ok();
        }
    }

    let jwt = ctx.auth.mint_jwt(root_admin.id, tenant_id, vec!["signup_first_login"], &req.signup_session_id).await?;
    cookies.add(secure_cookie("cyberos_jwt", &jwt, "*.cyberos.world", chrono::Duration::hours(24)));

    emit_audit(&ctx, "ten.signup_completed", json!({
        "tenant_id": tenant_id, "plan_tier": req.plan_tier, "currency": req.billing_currency,
        "session_id": session.signup_session_id, "duration_ms": session.duration_ms(),
    })).await;
    ctx.email.send_welcome(&ctx, tenant_id, &req.billing_contact_email, &req.tenant_slug).await.ok();

    Ok(SignupCompleteResp{
        tenant_id,
        tenant_slug: req.tenant_slug,
        jwt,
        redirect_url: format!("https://{}.cyberos.world/onboarding/welcome", req.tenant_slug),
    })
}
```

### 6.2 OTP HMAC

```rust
// services/ten/src/signup/otp.rs
use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

pub fn hash_otp(otp: &str, signup_session_id: uuid::Uuid, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(otp.as_bytes());
    mac.update(signup_session_id.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **FR-AUTH-104** OIDC SSO — Google/Microsoft sign-up path.
- **FR-TEN-001** Provisioning — `provision_tenant` invoked at signup_complete step 8.
- **FR-TEN-002** Plan tiers — `plan_tier` selection + per-tier subscription.
- **FR-TEN-003** Stripe billing — `ensure_customer` + `ensure_subscription` at step 9.

**Cross-module (related_frs):**
- **FR-TEN-004** 4-axis metering — first metering events fire post-signup.
- **FR-TEN-102** VND domestic rail — VNP path placeholder until FR-TEN-102 ships.
- **FR-TEN-103** 4-residency provisioning — residency derivation aligns.
- **FR-TEN-104** Lifecycle — rollback path may invoke termination.
- **FR-TEN-107** Tenant-admin SPA — first destination post-signup.
- **FR-AUTH-001** Tenant create — invoked indirectly via FR-TEN-001.
- **FR-AUTH-002** Subject create — root-admin creation.
- **FR-AUTH-004** JWT mint — first-login token.
- **FR-AUTH-101** RBAC — root-admin gets `tenant_admin` role.
- **FR-AUTH-107** HIBP breach-check — informational warning at signup.
- **FR-EMAIL-001** Transactional email — OTP + welcome email + magic-link.
- **FR-PORTAL-001/002** — depend on TEN-101 for tenant-scoped brand pack flow.
- **FR-AI-003** memory audit — 9 new kinds register here.
- **FR-MEMORY-111** PII scrubbing — email + IP scrubbed in chain rows.
- **FR-OBS-007** Auto-runbook — sev-1/sev-2 alerts route to CHAT/PagerDuty.

**Downstream (blocks):**
- **FR-TEN-107** — tenant-admin SPA depends on signup completing.
- **FR-PORTAL-001** — scoped read-only views require self-served tenants.
- **FR-PORTAL-002** — per-tenant brand pack requires self-served tenants.

---

## §8 — Example payloads

### 8.1 `POST /v1/signup/start` request/response

```json
// request
{ "email": "alice@acme.com", "signup_session_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
  "turnstile_token": "0.xxx", "locale_hint": "en-US" }

// response
{ "signup_session_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
  "email_verification_required": true,
  "suggested_billing_currency": "USD",
  "suggested_residency": "us-1",
  "geoip_country": "US",
  "otp_sent_at": "2026-05-17T09:14:32.847Z",
  "breach_warning": null }
```

### 8.2 `ten.signup_completed` memory row

```json
{
  "kind": "ten.signup_completed",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "system.ten.signup",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:58.221Z",
  "payload": {
    "signup_session_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
    "tenant_id": "8a2f...",
    "tenant_slug": "acme-co",
    "plan_tier": "team",
    "billing_currency": "USD",
    "billing_rail": "stripe",
    "residency": "us-1",
    "email_hash16": "f8a1b2c3d4e5f607",
    "duration_ms": 23847,
    "oidc_path": false,
    "geoip_country": "US"
  }
}
```

### 8.3 `POST /v1/signup/complete` request

```json
{
  "signup_session_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
  "tenant_slug": "acme-co",
  "tenant_display_name": "Acme Inc.",
  "plan_tier": "team",
  "billing_currency": "USD",
  "billing_contact_email": "alice@acme.com",
  "stripe_setup_intent_id": "seti_3OabcXYZ012345",
  "consents": [
    {"kind": "tos",       "version": "v2.3", "locale": "en-US"},
    {"kind": "privacy",   "version": "v2.1", "locale": "en-US"},
    {"kind": "marketing", "version": "v1.0", "locale": "en-US"}
  ],
  "turnstile_token": "0.yyy"
}
```

### 8.4 `POST /v1/signup/complete` response (201)

```json
{
  "tenant_id": "8a2f9c1d-4b6e-7f80-bbbb-000000000042",
  "tenant_slug": "acme-co",
  "jwt": "eyJhbGciOi...",
  "redirect_url": "https://acme-co.cyberos.world/onboarding/welcome"
}
```

### 8.5 Duplicate-email response (409)

```json
{
  "error": "email_already_associated",
  "magic_link_sent": true,
  "existing_tenant_hint": "acme-co",
  "message": "An account already exists for this email. We've sent you a sign-in link."
}
```

### 8.6 `ten.signup_rate_limited` memory row

```json
{
  "kind": "ten.signup_rate_limited",
  "severity": 3,
  "tenant_id": "00000000-0000-0000-0000-000000000001",  // system tenant
  "actor_id": "anonymous",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:34.001Z",
  "payload": {
    "guard": "signup_start",
    "key_kind": "ip",
    "ip_addr_hash16": "9c4e7a8b6d2f1e3a",
    "retry_after_seconds": 60
  }
}
```

---

## §9 — Open questions

All resolved for slice 1. Deferred to later slices:

- **Deferred:** Magic-link sign-in flow for existing-tenant returns — slice 2, FR-TEN-1xx (placeholder — not yet specified). For slice 1, the `magic_link_sent: true` response sets a follow-up commitment.
- **Deferred:** Multi-step ToS/Privacy version-bump re-acceptance flow — slice 2, FR-TEN-1xx.
- **Deferred:** B2B "invite teammate" flow at signup (currently root-admin only; teammates invited post-signup via FR-AUTH-101 admin) — slice 2.
- **Deferred:** Phone-number alternate identifier for SMS OTP — slice 2 (email-only at slice 1).
- **Deferred:** SSO-only enforcement at signup (some enterprises want "no password account creation") — slice 2.
- **Deferred:** Sub-tenant signup (PORTAL invites; FR-PORTAL-001 path) — out-of-scope at this FR.
- **Deferred:** Annual billing cycle option at signup — slice 2 / FR-TEN-1xx (currently monthly only per FR-TEN-003 §9).
- **Deferred:** Promotional coupon code field at signup — slice 2.
- **Deferred:** Enterprise self-serve via Sales-Assisted flow (signup → sales calendar booking → manual provisioning) — out-of-scope at this FR (DEC-823).
- **Deferred:** eIDAS QTSP-compliant signup for EU regulated tenants — slice 3 / FR-TEN-2xx (placeholder — not yet specified).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Turnstile token invalid/expired | Cloudflare verify returns false | 400 + `turnstile_failed`; no session created | User refreshes form; new token issued automatically |
| Disposable email domain | blocklist lookup hits | 400 + `disposable_email`; suggest work email | User retries with non-disposable domain |
| Rate-limit hit | Redis sliding-window counter ≥ threshold | 429 + `Retry-After` header + `ten.signup_rate_limited` | User waits + retries; severe abuse → squatting block |
| OTP delivery fails (email service down) | FR-EMAIL-001 returns error | 503 + `otp_send_failed`; session state remains 'started'; user can resend after rate-limit window | Email service recovers; retry within 10min OTP window or new OTP |
| OTP TTL expired | Redis key not found at verify | 410 + `otp_expired`; user requests new OTP | Resend OTP (counts against otp_send rate limit) |
| OTP max attempts (5) | counter in Redis hits 5 | 429 + `otp_max_attempts`; OTP invalidated | New OTP send required |
| Slug race (two tenants claim same slug concurrently) | Postgres unique index on `tenants.slug` rejects 2nd INSERT | 409 + suggestions; session.state='abandoned' reason='slug_collision' | User picks alternate slug; resume from slug-check step |
| Stripe SetupIntent decline (card declined) | Stripe API returns `payment_intent.payment_failed` | 402 + `payment_declined` + Stripe decline_code; no tenant created | User updates card + retries; signup_complete reattempt with new intent |
| Stripe API 5xx during confirm | api_client retry exhausted (FR-TEN-003 §1 #15) | 502 + `stripe_unavailable`; session.state remains 'email_verified'; user can retry | Stripe recovers; idempotency key prevents double-charge on retry |
| TEN-001 provisioning fails after Stripe confirm | provision_tenant returns error inside tx | Stripe SetupIntent voided + tx rolled back + session.state='rolled_back' reason='provisioning_failed' | Operator investigates; user contacted via billing_contact_email |
| AUTH root-admin creation fails inside the atomic tx | create_subject returns error WITHIN the Phase-C tx | `ROLLBACK` undoes tenant + consent INSERTs (no tenant exists post-rollback); Stripe SetupIntent voided; signup_session.state='rolled_back' reason='root_admin_create_failed'; sev-1 logged | Operator investigates; user contacted via billing_contact_email; FR-TEN-104 NOT invoked (no tenant to terminate) |
| Welcome email send fails | FR-EMAIL-001 returns error post-commit | signup is COMPLETE (tx already committed); email retry job runs background; user can request resend via /v1/signup/resend-welcome | Email recovers; user logs in via JWT in response anyway |
| Same-email duplicate signup | unique index check on tenants.billing_contact_email_hash16 | 409 + magic-link email sent; no second tenant | User signs into existing tenant via magic link |
| OIDC ID token signature invalid | FR-AUTH-104 verification fails | 401 + `oidc_invalid_token`; session.state remains 'started' | User retries OIDC flow |
| OIDC email_verified=false (IdP says email not verified) | claim check | 400 + `oidc_email_unverified`; fall back to OTP path | User verifies email at IdP, retries |
| GeoIP DB out of date / IP not resolvable | MaxMind returns Unknown | Default to USD/us-1; user can override | Refresh weekly; manual override always available |
| Consent missing for VND tenant (PDPL VN) | validate_consents check | 400 + `missing_pdpl_vn_consent`; signup blocked | User checks the PDPL consent box and retries |
| Enterprise plan_tier attempted via API | plan_tier == Enterprise check at complete | 403 + `enterprise_requires_sales_contact` | User selects Starter/Team; contacts sales for Enterprise |
| Squatting: 5 abandons from same IP in 24h | abuse_guard counter | 429 + `abuse_detected_24h_cooloff` for 24h | Auto-expires at 24h; legitimate users contact support for manual unblock |
| Redis unavailable | rate_limit + OTP storage fail | 503 + `signup_temporarily_unavailable` | Redis recovers; signup retries auto-resume |
| MaxMind GeoLite2 file missing/corrupt | geoip lookup throws | Default to USD/us-1 + sev-2 alert `ten.signup_geoip_unavailable` (not in 9-kind core) | Weekly refresh job re-fetches; manual refresh CLI |
| Turnstile site key misconfig | every verify returns false | Universal 400 on signup_start; sev-1 alert | Operator updates env var; restart |
| signup_session_id reuse across users (race / leak) | Postgres unique index on signup_session_id | 409 + `session_id_collision` | User generates new UUIDv7 (client-side); retries |
| RLS misconfig pre-tenant | session row inaccessible despite system_tenant scope | Handler logs `auth.tenant_id` mismatch + 500 | Inspect handler's SET LOCAL chain; fix middleware |
| Email service marked the OTP as spam | OTP not received by user | User clicks resend; rate-limited at 3/min/email | Whitelist sender domain at IdP-side / use known transactional domains |

---

## §11 — Implementation notes

**§11.1** UUIDv7 is chosen for `signup_session_id` so that session_id ordering is rough wall-clock; allows memory audit + signup_sessions to be sorted by id for analytics without an additional timestamp column.

**§11.2** The Turnstile site key is per-environment (test/staging/prod); the verify endpoint URL `https://challenges.cloudflare.com/turnstile/v0/siteverify` is the same. Verification is a POST with `secret` + `response` + optional `remoteip`.

**§11.3** OTP HMAC secret rotates quarterly via KMS-managed key; rotation is overlap-safe (old + new accepted for 10-min OTP TTL after rotation).

**§11.4** The MaxMind GeoLite2 database is ~70MB; loaded into memory at handler startup via `maxminddb` crate. Refresh job (weekly) writes a new file then atomically swaps the in-memory pointer.

**§11.5** The disposable email blocklist initial seed: `disposable-email-domains` GitHub repo at pinned commit. Refresh job (monthly) fetches latest + diffs against current; writes a `disposable_email_blocklist_refresh` ledger row.

**§11.6** Rate-limit Redis keys: `rl:signup_start:ip:<ip_hash16>:<window>`, `rl:otp_send:email:<email_hash16>:<window>`, etc. Sliding-window uses Redis sorted sets with score=timestamp; trim on insert.

**§11.7** The `signup_30s_sla_test` uses Stripe test-mode against the real Stripe API (not wiremock) to ensure realistic network latency is in scope. CI runs it nightly, not on every PR.

**§11.8** OIDC `email_verified` claim is checked against both `email_verified: true` AND IdP issuer ∈ trusted list (Google + Microsoft). Other IdPs (Apple, Facebook) ship in slice 2.

**§11.9** The duplicate-email check (DEC-842 + §1 #14 step 5) uses `billing_contact_email_hash16` not the full email — same-email check is hash-equivalence. The hash includes a global salt to prevent rainbow-table enumeration.

**§11.10** The OTP code is 6 digits = ~20-bit entropy; 10-min TTL × 5 attempts = ~3.3e6 brute attempts needed on average vs 6 attempts = effectively unguessable.

**§11.11** Slug suggestions on collision: 50 = generous; UI shows 5 initially with "more" toggle. Generation: append numeric suffix (1-99), or append a 3-letter consonant tuple (acme-co, acme-hq, acme-inc, etc.).

**§11.12** The `redirect_url` in the complete response is the trusted post-signup destination; the JWT in the response is a temporary first-login token (24h TTL) with `signup_first_login` scope. The user lands on `<slug>.cyberos.world/onboarding/welcome` which clears the temp token + issues the normal long-lived session.

**§11.13** The welcome email's 7-day expiry is enforced at the `<slug>.cyberos.world/onboarding/welcome` route handler — `?token=<one-time-token>` query parameter checked + the token TTL is 7d in Postgres. Expired → 404 with link to standard login.

**§11.14** The PII scrub job (§1 #21) runs daily at 03:00 UTC via the same scheduled-job framework as FR-TEN-003's TTL pruner. Scrubbed rows retain analytical value (funnel state, geoip_country) but no longer carry personally identifying data.

**§11.15** The `cyberos_consent_writer` role (§3.1 grant) is held only by the consent-withdrawal endpoint (slice 2, FR-TEN-1xx); slice 1 has no withdrawal flow yet, so the role exists with no live grants beyond the schema permission.

**§11.16** The signup flow is the FIRST place where `signup_session_id` is generated; the client (frontend SPA) generates a UUIDv7 at page load and reuses across all 5 endpoints. Server never generates session_ids — this makes the flow resumable across browser reloads.

**§11.17** Stripe SetupIntent vs PaymentIntent: SetupIntent captures the payment method for FUTURE charges (subscription billing); PaymentIntent charges immediately. We use SetupIntent because subscription billing in FR-TEN-003 starts at next billing cycle (or pro-rata immediate but as a Stripe Subscription invoice, not a one-off charge).

**§11.18** The 30-second SLO breakdown: ~3s Turnstile verify, ~1s OTP send (email queue ack), ~10s user fills email/slug/plan/payment, ~5s Stripe Elements card capture + 3DS, ~3s signup_complete server work (Stripe confirm + TEN-001 provision + AUTH + JWT). Buffer = 8s.

**§11.19** The form is intentionally single-page (not multi-page wizard) to minimize navigation cost. Five "steps" are visual sections with smooth scroll; submit is all-at-once at the end.

**§11.20** Test fixtures for `signup_30s_sla_test` use Stripe test cards that auto-succeed (no 3DS challenge); production users hit 3DS on ~5-10% of cards which adds ~5s. The 30s SLO is measured with 3DS-skip; the realistic-3DS SLO is 40s (informally tracked).

**§11.21** The system tenant UUID `00000000-0000-0000-0000-000000000001` is reserved by FR-AUTH-001 §3.2 (per FR-AUTH-001's DEC entry) — not invented here. This FR consumes the well-known sentinel.

**§11.22** Welcome email content includes: tenant slug, login URL, JWT-as-magic-link (24h TTL), root-admin's auto-generated password (one-time display), short onboarding checklist. The password lives in the email body which is sensitive — FR-EMAIL-001 enforces TLS-only delivery + DKIM + SPF.

**§11.23** Browser feature detection: form requires JavaScript + cookies + WebCrypto (for Turnstile + Stripe Elements). Browsers without are shown a 503 "Self-serve signup requires a modern browser; contact sales@cyberos.world."

**§11.24** Analytics emission: the 5 funnel stages (start, otp_verified, payment_captured, provisioned, completed) emit OTel events with attribute `funnel_stage`; drop-off analysis lives in OBS (FR-OBS-005).

**§11.25** Locale handling: `locale_hint` from the client (Accept-Language header default + override) drives consent locale + welcome email locale + UI strings. Supported at slice 1: `en-US`, `vi-VN`; slice 2 adds `zh-SG`, `de-DE`, `fr-FR`, `es-ES`.

**§11.26** The form submits to our backend over HTTPS only; HSTS preload + Content-Security-Policy `default-src 'self'; script-src 'self' https://challenges.cloudflare.com https://js.stripe.com` prevents MitM + XSS. Stripe Elements iframes are sandboxed.

**§11.27** The deeplink at `<slug>.cyberos.world/onboarding/welcome` is signed with HMAC + nonce; sharing the link does not let a third party log in (token is single-use per DEC-845).

---

*End of FR-TEN-101 spec.*
