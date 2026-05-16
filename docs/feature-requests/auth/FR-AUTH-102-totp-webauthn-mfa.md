---
id: FR-AUTH-102
title: "AUTH TOTP (RFC 6238) + WebAuthn Level 3 MFA — closed factor enum + enrolment FSM + challenge/response + recovery codes + sev-1 lockout + BRAIN audit per factor lifecycle event"
module: AUTH
priority: MUST
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-AUTH-002, FR-AUTH-004, FR-AUTH-101, FR-AI-003, FR-BRAIN-101, FR-AUTH-105, FR-AUTH-106, FR-OBS-007]
depends_on: [FR-AUTH-002]
blocks: [FR-AUTH-105, FR-AUTH-106]

source_pages:
  - website/docs/modules/auth.html#mfa
  - https://www.w3.org/TR/webauthn-3 (WebAuthn Level 3)
  - https://datatracker.ietf.org/doc/html/rfc6238 (TOTP)
source_decisions:
  - DEC-480 (closed factor_kind enum at 3 values: totp · webauthn_platform · webauthn_cross_platform — adding a 4th (sms, email_otp, biometric_external) is an ADR; webauthn split by platform per W3C terminology)
  - DEC-481 (TOTP per RFC 6238 — HMAC-SHA1 (RFC 4226 base) at 30-second time step, 6-digit code; SHA-256 variant is an ADR addition — most authenticator apps support SHA-1 only)
  - DEC-482 (TOTP secret 160-bit (20-byte) random, base32-encoded, KMS-encrypted at rest, never exposed after enrolment — only the QR-code provisioning URI shown once)
  - DEC-483 (WebAuthn Level 3 — relying party id = `cyberos.world` at slice 1; user verification = required; resident keys (discoverable credentials) = preferred for passkey UX)
  - DEC-484 (closed challenge lifecycle FSM: pending → consumed | expired — challenge TTL 5 minutes; reuse → rejected with sev-2 audit; expired → 401 challenge_expired)
  - DEC-485 (recovery codes — 10 codes per subject, 8 chars base32, single-use, bcrypt-hashed at rest, regenerated on demand; consuming one emits sev-2 BRAIN row)
  - DEC-486 (BRAIN audit kinds: auth.mfa_factor_enrolled, auth.mfa_factor_removed, auth.mfa_challenge_issued, auth.mfa_challenge_succeeded, auth.mfa_challenge_failed, auth.mfa_recovery_code_consumed, auth.mfa_locked_out, auth.mfa_unlocked)
  - DEC-487 (lockout policy: 5 failed challenges in 15min → lock subject's MFA for 30min; emit sev-1 `auth.mfa_locked_out` BRAIN row; root-admin can unlock early via dedicated handler emitting `auth.mfa_unlocked`)
  - DEC-488 (REVOKE UPDATE, DELETE on mfa_challenge_log + mfa_factor_history from cyberos_app — append-only at SQL grant)
  - DEC-489 (TOTP step skew tolerance = ±1 (60s total window) per RFC 6238 §5.2 — covers clock drift; widening tolerance increases brute-force surface so capped at 1)
  - DEC-490 (WebAuthn challenge signature verified against the credential's stored public key via `webauthn-rs` crate; counter-monotonicity check to detect cloned authenticators per W3C §10.4)
  - DEC-491 (per-tenant MFA policy in tenant_policy YAML: `mfa_required_roles: [<role>...]` — when a subject's roles intersect this set, login requires MFA challenge; founder role always requires per FR-AUTH-101 DEC-128)
  - DEC-492 (recovery codes regeneration invalidates ALL prior codes — emit `auth.mfa_factor_enrolled` row with `factor_kind='recovery_codes_batch'`; never partial regeneration to avoid bookkeeping ambiguity)
  - DEC-493 (challenge response carries `factor_id` so verifier knows which credential to check against — prevents credential confusion attack)
  - DEC-494 (WebAuthn attestation = `none` at slice 1 — `direct`/`indirect` attestation deferred to FR-AUTH-2xx; suitable for our risk model since we control the relying party + enforce user verification)
  - DEC-495 (FIDO2 + Passkey conformance — the platform-authenticator path IS the passkey path; FR-AUTH-105's passkey enrolment uses this FR's WebAuthn factor with `resident_key=preferred`)
  - NIST SP 800-63B (AAL2 for TOTP; AAL3 for WebAuthn cross-platform with attestation)
  - W3C WebAuthn Level 3 (2024); RFC 6238 TOTP; RFC 4226 HOTP

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0016_mfa_factors.sql
  - services/auth/migrations/0017_mfa_factor_history.sql
  - services/auth/migrations/0018_mfa_challenge_log.sql
  - services/auth/migrations/0019_mfa_recovery_codes.sql
  - services/auth/migrations/0020_mfa_lockout_state.sql
  - services/auth/src/mfa/mod.rs
  - services/auth/src/mfa/factor_kind.rs                    # closed 3-value enum
  - services/auth/src/mfa/totp.rs                           # RFC 6238 generator + verifier
  - services/auth/src/mfa/webauthn.rs                       # webauthn-rs wrapper
  - services/auth/src/mfa/challenge.rs                      # challenge issuance + verification + lifecycle FSM
  - services/auth/src/mfa/recovery.rs                       # 10-code batch generator + verifier (bcrypt-hashed)
  - services/auth/src/mfa/lockout.rs                        # 5/15min → 30min lock state machine
  - services/auth/src/mfa/policy.rs                         # per-tenant MFA-required-roles policy lookup
  - services/auth/src/mfa/repo.rs                           # CRUD across mfa_factors + history + log + recovery + lockout
  - services/auth/src/mfa/audit.rs                          # 8 BRAIN row builders
  - services/auth/src/handlers/mfa.rs                       # enrol + challenge + verify + recovery + unlock
  - services/auth/tests/mfa_factor_kind_closed_test.rs
  - services/auth/tests/mfa_totp_rfc6238_test.rs
  - services/auth/tests/mfa_totp_secret_kms_encrypted_test.rs
  - services/auth/tests/mfa_webauthn_enrol_test.rs
  - services/auth/tests/mfa_webauthn_counter_monotonic_test.rs
  - services/auth/tests/mfa_challenge_ttl_test.rs
  - services/auth/tests/mfa_challenge_replay_rejected_test.rs
  - services/auth/tests/mfa_recovery_codes_single_use_test.rs
  - services/auth/tests/mfa_recovery_regen_invalidates_all_test.rs
  - services/auth/tests/mfa_lockout_5_in_15_test.rs
  - services/auth/tests/mfa_root_admin_unlock_test.rs
  - services/auth/tests/mfa_policy_role_gate_test.rs
  - services/auth/tests/mfa_append_only_log_test.rs
  - services/auth/tests/mfa_audit_emission_test.rs
modified_files:
  - services/auth/src/jwt/issuer.rs                         # MFA-required gate consults policy before issuing JWT
  - services/auth/src/lib.rs                                # pub mod mfa

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test mfa

disallowed_tools:
  - allow TOTP step skew > 1 (per DEC-489 — brute force surface)
  - skip WebAuthn counter-monotonicity check (per DEC-490 — cloned-authenticator defense)
  - skip user verification on WebAuthn (per DEC-483)
  - allow partial recovery-code regeneration (per DEC-492 — invalidate-all only)
  - expose TOTP secret after enrolment (per DEC-482)
  - allow > 5 failed challenges before lockout (per DEC-487)
  - allow tenant-admin to unlock (per DEC-487 — root-admin only)

effort_hours: 10
sub_tasks:
  - "0.5h: 0016_mfa_factors.sql + ENUM"
  - "0.3h: 0017_mfa_factor_history.sql append-only"
  - "0.4h: 0018_mfa_challenge_log.sql append-only + RLS"
  - "0.4h: 0019_mfa_recovery_codes.sql"
  - "0.4h: 0020_mfa_lockout_state.sql"
  - "0.7h: factor_kind.rs + closed enum"
  - "1.0h: totp.rs — RFC 6238 implementation + tests against published vectors"
  - "1.4h: webauthn.rs — webauthn-rs integration + counter check"
  - "0.8h: challenge.rs — lifecycle FSM + TTL"
  - "0.6h: recovery.rs — bcrypt + single-use + regen"
  - "0.6h: lockout.rs — 5/15min → 30min state machine"
  - "0.4h: policy.rs — per-tenant role-gate lookup"
  - "0.5h: repo.rs + audit.rs + handlers/mfa.rs"
  - "1.0h: tests (14 files)"

risk_if_skipped: "Without MFA, FR-AUTH-101's `requires_webauthn` flag on the founder role is inert — there's no second factor to require. Every downstream sensitive operation (CFO disbursement approval, root-admin tenant provisioning, ESOP grant signoff, legal-hold apply) falls back to password-only auth — phishable + replayable. FR-AUTH-105 passkey enrolment can't ship without DEC-495's WebAuthn platform-authenticator foundation. Without DEC-490's counter-monotonicity check, cloned WebAuthn authenticators succeed (real attack — see W3C §10.4). Without DEC-487's lockout, password-guessing attackers get unlimited attempts. Without DEC-492's invalidate-all-recovery-code-regen, half-rotated batches create permanent recovery confusion. The 10h effort lands the NIST AAL2/AAL3 substrate that every privileged operation depends on."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship TOTP (RFC 6238) + WebAuthn Level 3 MFA with closed factor enum + enrolment FSM + challenge/response + recovery codes + lockout policy. Each requirement:

1. **MUST** define `mfa_factors` table: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL REFERENCES auth.subjects(id), factor_kind factor_kind NOT NULL, display_name TEXT NOT NULL, totp_secret_kms_blob BYTEA, totp_kms_key_id TEXT, webauthn_credential_id BYTEA, webauthn_public_key BYTEA, webauthn_aaguid UUID, webauthn_signature_count BIGINT NOT NULL DEFAULT 0, enrolled_at TIMESTAMPTZ NOT NULL DEFAULT now(), last_used_at TIMESTAMPTZ, status TEXT NOT NULL CHECK (status IN ('active','removed')) DEFAULT 'active')`. UNIQUE per (subject_id, factor_kind, webauthn_credential_id) — same authenticator cannot enrol twice; subject MAY have multiple WebAuthn devices.

2. **MUST** declare the closed `factor_kind` Postgres enum with exactly 3 values (per DEC-480): `'totp'`, `'webauthn_platform'`, `'webauthn_cross_platform'`. Adding a 4th is an ADR.

3. **MUST** define `mfa_factor_history` table for append-only enrol/remove events: `(id BIGSERIAL, tenant_id UUID, subject_id UUID, factor_id UUID, action TEXT NOT NULL CHECK (action IN ('enrolled','removed','status_changed')), factor_kind factor_kind NOT NULL, display_name TEXT, changed_at TIMESTAMPTZ, changed_by_subject_id UUID, reason TEXT)`. `REVOKE UPDATE, DELETE FROM cyberos_app`.

4. **MUST** define `mfa_challenge_log` table: `(id BIGSERIAL, tenant_id UUID, subject_id UUID, challenge_id UUID NOT NULL UNIQUE, factor_id UUID, challenge_kind TEXT NOT NULL, status TEXT NOT NULL CHECK (status IN ('pending','consumed','expired','failed')), issued_at TIMESTAMPTZ NOT NULL DEFAULT now(), expires_at TIMESTAMPTZ NOT NULL, consumed_at TIMESTAMPTZ, source_ip_hash16 TEXT)`. Append-only via SQL grant.

5. **MUST** define `mfa_recovery_codes` table: `(id UUID PRIMARY KEY, tenant_id UUID, subject_id UUID NOT NULL REFERENCES auth.subjects(id), code_bcrypt_hash TEXT NOT NULL, batch_id UUID NOT NULL, consumed BOOLEAN NOT NULL DEFAULT false, consumed_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Partial unique `(subject_id) WHERE batch_id = <current batch>` enforced via separate batch UUID.

6. **MUST** define `mfa_lockout_state` table: `(subject_id UUID PRIMARY KEY REFERENCES auth.subjects(id), tenant_id UUID NOT NULL, failed_count INT NOT NULL DEFAULT 0, window_started_at TIMESTAMPTZ NOT NULL DEFAULT now(), locked_until TIMESTAMPTZ, last_attempt_at TIMESTAMPTZ)`. UPDATE allowed via privileged `mfa_lockout_writer` role only.

7. **MUST** enforce RLS with `USING + WITH CHECK` on all 5 tables. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid OR current_setting('auth.is_root_admin', true) = 'true'`. Root-admin sees all (for support + unlock).

8. **MUST** implement TOTP per RFC 6238 (per DEC-481):
    - Algorithm: HMAC-SHA1 (RFC 4226 base).
    - Time step: 30 seconds.
    - Code length: 6 digits.
    - Step skew tolerance: ±1 step (60s total window) per DEC-489.
    - Secret: 160-bit random, base32-encoded.
    - Provisioning URI: `otpauth://totp/CyberOS:{tenant_slug}/{subject_email}?secret={base32_secret}&issuer=CyberOS&algorithm=SHA1&digits=6&period=30`.

9. **MUST** KMS-encrypt the TOTP secret at rest (per DEC-482). `totp_secret_kms_blob` column holds the ciphertext; `totp_kms_key_id` records the key used. Plaintext secret is exposed ONLY at enrolment time (as QR code) — never queryable thereafter. Verification flow: decrypt → HMAC-SHA1 compute → compare → discard plaintext.

10. **MUST** implement WebAuthn Level 3 per DEC-483 + DEC-490 via the `webauthn-rs` crate:
    - Relying party id: `cyberos.world` (slice 1; per-tenant subdomains in slice 3).
    - User verification: `required` (no `preferred` fallback).
    - Resident keys: `preferred` (enables passkey UX in FR-AUTH-105).
    - Attestation: `none` at slice 1 (per DEC-494).
    - Counter-monotonicity: on each verify, fetch stored `signature_count`; if response's count ≤ stored → reject with `cloned_authenticator_detected` + sev-1 audit + revoke factor.

11. **MUST** ship the challenge lifecycle FSM (per DEC-484):
    - `pending` (issued) → `consumed` (verified successfully).
    - `pending` → `expired` (TTL passed).
    - `pending` → `failed` (verification failed; rate-limit applies).
   `consumed → *`, `expired → *`, `failed → *` are all terminal. Reuse of a `consumed` or `expired` challenge → 401 + sev-2 audit.

12. **MUST** TTL challenges at 5 minutes (per DEC-484). Background job (or lazy lookup) marks `pending` rows whose `expires_at < now()` as `expired`. The verify handler explicitly checks `status='pending' AND expires_at > now()` before consuming.

13. **MUST** generate 10 recovery codes at first MFA enrolment (per DEC-485):
    - Format: 8 chars base32 (excludes 0/O/1/L for legibility).
    - bcrypt-hashed at cost 12 at rest.
    - Single-use: on consumption, set `consumed=true, consumed_at=now()` and emit `auth.mfa_recovery_code_consumed` (sev-2).
    - Subject MAY regenerate via `POST /v1/auth/mfa/recovery-codes/regen` — invalidates ALL prior codes (new batch_id; old batch's codes orphaned but kept for audit history).

14. **MUST** implement lockout (per DEC-487):
    - Count failed challenges per subject in a sliding 15-minute window.
    - 5 failures in window → set `locked_until = now() + 30 minutes`; emit `auth.mfa_locked_out` BRAIN row at sev-1.
    - During lockout, all MFA verify attempts return 423 `mfa_locked`; failure counter does NOT increment.
    - At `locked_until` expiry, the next attempt succeeds normally if credentials valid.
    - Root-admin can early-unlock via `POST /v1/auth/mfa/unlock` (emits `auth.mfa_unlocked` sev-2).

15. **MUST** consult per-tenant MFA policy (per DEC-491) at JWT issuance time. The tenant policy YAML (FR-AI-005) declares `mfa_required_roles: [<role>...]`. JWT issuer checks: if subject's roles ∩ mfa_required_roles ≠ ∅ AND no recent successful MFA challenge → return 401 `mfa_challenge_required` with `factor_kinds: [<available factors for subject>]`. Founder role ALWAYS requires (DEC-128).

16. **MUST** emit BRAIN audit rows for 8 kinds (per DEC-486):
    - `auth.mfa_factor_enrolled` — POST /enrol success.
    - `auth.mfa_factor_removed` — POST /remove success.
    - `auth.mfa_challenge_issued` — POST /challenge success.
    - `auth.mfa_challenge_succeeded` — POST /verify success.
    - `auth.mfa_challenge_failed` — POST /verify failure (counter increments).
    - `auth.mfa_recovery_code_consumed` — sev-2.
    - `auth.mfa_locked_out` — sev-1 with subject_id_hash16 + failed_count + locked_until.
    - `auth.mfa_unlocked` — sev-2 with reason + root-admin actor.

17. **MUST** PII-scrub `display_name`, `reason`, and `source_ip_hash16` via FR-BRAIN-111 before chain commit.

18. **MUST** expose REST handlers:
    - `POST /v1/auth/mfa/factors/totp/enrol` — generates secret + returns QR provisioning URI; subject confirms first code; row created.
    - `POST /v1/auth/mfa/factors/webauthn/enrol/begin` — generates WebAuthn creation challenge.
    - `POST /v1/auth/mfa/factors/webauthn/enrol/finish` — validates attestation + stores credential.
    - `DELETE /v1/auth/mfa/factors/{id}` — soft-delete (status=removed); audit row.
    - `POST /v1/auth/mfa/challenges` — issue challenge for `factor_id`; returns challenge_id + nonce.
    - `POST /v1/auth/mfa/verify` — verify challenge response; transitions to consumed.
    - `POST /v1/auth/mfa/recovery-codes/regen` — invalidates current batch + returns 10 new codes (once).
    - `POST /v1/auth/mfa/recovery-codes/consume` — consume one recovery code.
    - `POST /v1/auth/mfa/unlock` — root-admin only.

19. **MUST** complete MFA verify in ≤ 100 ms p95 (TOTP) / 300 ms p95 (WebAuthn — includes signature verify). `mfa_perf_test`.

20. **MUST** emit OTel span `auth.mfa.{enrol,challenge,verify,recovery,lockout,unlock}` with attributes: `tenant_id`, `subject_id_hash16`, `factor_kind`, `outcome` (success | challenge_expired | challenge_reused | invalid_code | cloned_authenticator | locked | unknown_factor | policy_required | unlock_not_root_admin).

21. **MUST** emit OTel metrics:
    - `auth_mfa_factor_enrolled_total{tenant_id, factor_kind}` (counter).
    - `auth_mfa_challenge_issued_total{tenant_id, factor_kind}` (counter).
    - `auth_mfa_challenge_succeeded_total{tenant_id, factor_kind}` (counter).
    - `auth_mfa_challenge_failed_total{tenant_id, factor_kind, reason}` (counter).
    - `auth_mfa_recovery_consumed_total{tenant_id}` (counter — sev-2 alarm at sustained > 5/h indicates compromise).
    - `auth_mfa_locked_out_total{tenant_id}` (counter — sev-1 alarm always).
    - `auth_mfa_active_factors{tenant_id, factor_kind}` (gauge).

22. **MUST** ship the `mfa_lockout_writer` SQL role distinct from cyberos_app. Only this role can UPDATE `mfa_lockout_state.failed_count`, `window_started_at`, `locked_until`. cyberos_app can SELECT.

23. **MUST** require `display_name` 1–80 chars on every factor enrolment (e.g. "iPhone 15 Touch ID", "Authy", "Yubikey 5C") — operator needs human-readable identifier to choose which factor to use on challenge.

24. **MUST** support factor enumeration via `GET /v1/auth/mfa/factors` returning `[{id, factor_kind, display_name, enrolled_at, last_used_at, status}]` for caller's subject. NEVER expose totp_secret_kms_blob or webauthn_public_key in API responses.

25. **MUST** validate per-RFC 6238 against the published test vectors at unit-test time — `mfa_totp_rfc6238_test` asserts known-good codes for known-good secrets at known timestamps.

26. **MUST** invalidate ALL recovery codes on TOTP secret regeneration or WebAuthn factor removal at the user's last remaining factor — security invariant: a subject with no remaining primary MFA factor cannot rely on stale recovery codes (recovery requires having had a working factor).

27. **MUST** require subject to confirm one valid TOTP code before completing enrolment (`POST /enrol/finish` includes a `confirmation_code` field; mismatch → enrolment row never persisted). Prevents misconfigured QR codes silently rolling forward.

---

## §2 — Why this design (rationale for humans)

**Why closed 3-factor enum (DEC-480, §1 #2)?** Limiting to (totp, webauthn_platform, webauthn_cross_platform) gives full NIST AAL2/AAL3 coverage without the operational complexity of SMS OTP (phishable + telco-vulnerable) or email OTP (phishable + delivery-unreliable). Adding a 4th is an ADR with explicit threat-model justification.

**Why TOTP HMAC-SHA1 not SHA-256 (DEC-481, §1 #8)?** Most authenticator apps (Google Authenticator, Authy, 1Password) implement RFC 6238 with HMAC-SHA1 only. SHA-256 is a TOTP extension not widely supported; using it would fragment compatibility. SHA-1 in TOTP is HMAC-keyed (not vulnerable to SHAttered-style attacks); collision resistance is irrelevant for OTP generation.

**Why 30-second time step + 6-digit code (DEC-481)?** Industry-standard parameters; all major authenticator apps default to these. Increasing digit count or shortening period breaks compatibility.

**Why ±1 step skew tolerance (DEC-489, §1 #8)?** Clock drift between user device + server typically < 1 second; ±1 step (30s) handles edge cases without widening the brute-force surface to multiple windows. Each additional step doubles the brute-force attempts a verifier accepts.

**Why KMS-encrypt TOTP secret + never expose post-enrol (DEC-482, §1 #9)?** Secret leak = TOTP code generation by attacker. KMS encryption requires KMS-decrypt permission for verification — meaningful additional barrier. Never-reveal-after-enrol means a compromised DB dump doesn't yield TOTP secrets if KMS keys are properly protected.

**Why WebAuthn user_verification=required (DEC-483, §1 #10)?** "Required" forces the authenticator to verify the user (biometric, PIN, etc.) before signing — provides "something you have AND something you are" rather than just "something you have". This is the AAL3 requirement.

**Why attestation=none at slice 1 (DEC-494)?** Direct/indirect attestation lets us verify the authenticator manufacturer (e.g. "only YubiKeys allowed"). For our risk model — internal app + commercial tenants — none is acceptable. FR-AUTH-2xx adds direct attestation for high-security tenant tiers.

**Why counter-monotonicity check (DEC-490, §1 #10)?** WebAuthn authenticators increment a signature counter per use. Two authenticators sharing the same private key (cloned) will produce non-monotonic counters across calls — we detect + revoke. This is the W3C-defined cloned-authenticator defense.

**Why 5-minute challenge TTL (DEC-484, §1 #12)?** Long enough for users to fetch their TOTP code or trigger their WebAuthn device; short enough that compromised challenges don't sit usable indefinitely. 5 min is the industry consensus middle.

**Why challenge-reuse rejection at sev-2 (§1 #11)?** Reuse = replay attack (attacker captured a valid challenge response + replays). Sev-2 because legitimate clients never reuse — every reuse is suspicious.

**Why 10 recovery codes + bcrypt cost 12 (DEC-485)?** 10 codes = enough for "I lost my phone" + multiple recovery events; bcrypt cost 12 matches FR-AUTH-002's password hash + makes offline brute-force impractical even if codes leak.

**Why regen invalidates all (DEC-492, §1 #13)?** Partial regen creates ambiguity: "which codes are still valid?" — error-prone, support-burden-heavy. Invalidate-all-on-regen is the unambiguous rule.

**Why 5/15 → 30min lockout (DEC-487, §1 #14)?** Limits brute-force to ~10 attempts/hour AND throttles to once-every-3-hours after lockout. Below 5 attempts is too aggressive (legitimate fat-finger fails); above is too permissive. 30-min lockout deters brute-force without permanent user impact.

**Why sev-1 on lockout (DEC-487)?** Lockouts indicate either (a) credential stuffing attack on a specific subject, or (b) compromised account being probed. Either way, operator needs to know within the hour. Sev-1 = page-on-call equivalent.

**Why per-tenant MFA-required-roles policy (DEC-491, §1 #15)?** Different tenants have different risk profiles. CyberSkill default: founder + CFO + cseco require MFA. Enterprise tenants may require all employees with `tenant-admin` role to MFA. Per-tenant config gives flexibility without forking the code.

**Why founder ALWAYS requires MFA (DEC-128 + DEC-491)?** Founder role grants cross-module privileged read — single most-sensitive role. MFA-always is the design assertion baked into FR-AUTH-101's `requires_webauthn` flag.

**Why root-admin-only unlock (DEC-487)?** Tenant-admin self-unlock = trivial bypass of lockout policy. Cross-tenant operator (CyberSkill ops) handles unlocks — same human-judgment gate as other privileged operations.

**Why display_name required + 1–80 chars (§1 #23)?** Multi-factor subjects need to distinguish "iPhone Touch ID" from "YubiKey at office desk" at challenge time. Display name is the disambiguator; required at enrol to enforce.

**Why confirm code on TOTP enrol (§1 #27)?** Misconfigured QR codes (operator screens TOTP secret incorrectly) silently roll forward without confirmation — user discovers at first login. Confirm-on-enrol catches misconfigurations immediately.

**Why invalidate recovery codes on last-factor removal (§1 #26)?** Security invariant: recovery is "alternative path when primary factor unavailable". Subject with zero primary factors has no path to authenticate; recovery codes alone shouldn't grant access (they're meant to bridge factor loss, not replace authentication).

**Why RLS allows root-admin (§1 #7)?** Support + unlock flows need root-admin to query any subject's MFA state. RLS predicate includes the root-admin escape; non-root-admin queries scope to their tenant.

**Why append-only history at SQL grant (§1 #3, §1 #4, DEC-488)?** Factor lifecycle + challenge events are forensic. SQL grant blocks even buggy app code from rewriting.

**Why `webauthn-rs` crate vs hand-roll (§1 #10)?** WebAuthn spec is complex (CBOR encoding, attestation formats, signature verification, multiple key types). `webauthn-rs` is the Rust-ecosystem standard; hand-rolling invites subtle bugs in cryptographic verification.

**Why 8 BRAIN audit kinds (DEC-486)?** Each represents a distinct operator-query category. "Show me all lockouts this week" → kind = `auth.mfa_locked_out`. "Who consumed recovery codes recently?" → kind = `auth.mfa_recovery_code_consumed`. Split kinds = selective queries; merged would force operator-side filtering.

---

## §3 — API contract

### 3.1 — Migration 0016 — mfa_factors

```sql
-- services/auth/migrations/0016_mfa_factors.sql

BEGIN;

CREATE TYPE factor_kind AS ENUM ('totp', 'webauthn_platform', 'webauthn_cross_platform');

CREATE TABLE mfa_factors (
    id                          UUID         PRIMARY KEY,
    tenant_id                   UUID         NOT NULL,
    subject_id                  UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    factor_kind                 factor_kind  NOT NULL,
    display_name                TEXT         NOT NULL CHECK (length(display_name) BETWEEN 1 AND 80),
    totp_secret_kms_blob        BYTEA,
    totp_kms_key_id             TEXT,
    webauthn_credential_id      BYTEA,
    webauthn_public_key         BYTEA,
    webauthn_aaguid             UUID,
    webauthn_signature_count    BIGINT       NOT NULL DEFAULT 0,
    enrolled_at                 TIMESTAMPTZ  NOT NULL DEFAULT now(),
    last_used_at                TIMESTAMPTZ,
    status                      TEXT         NOT NULL CHECK (status IN ('active','removed')) DEFAULT 'active'
);

CREATE UNIQUE INDEX uniq_subject_webauthn_cred ON mfa_factors (subject_id, webauthn_credential_id)
    WHERE webauthn_credential_id IS NOT NULL;
CREATE INDEX mfa_factors_subject_idx ON mfa_factors (subject_id, status);

ALTER TABLE mfa_factors ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_factors_tenant_iso ON mfa_factors
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.2 — Migration 0017 — mfa_factor_history (append-only)

```sql
-- services/auth/migrations/0017_mfa_factor_history.sql

BEGIN;

CREATE TABLE mfa_factor_history (
    id                       BIGSERIAL    PRIMARY KEY,
    tenant_id                UUID         NOT NULL,
    subject_id               UUID         NOT NULL,
    factor_id                UUID,
    action                   TEXT         NOT NULL CHECK (action IN ('enrolled','removed','status_changed')),
    factor_kind              factor_kind  NOT NULL,
    display_name             TEXT,
    changed_at               TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id    UUID,
    reason                   TEXT
);

CREATE INDEX mfa_factor_history_subject_idx ON mfa_factor_history (subject_id, changed_at DESC);

ALTER TABLE mfa_factor_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_factor_history_tenant_iso ON mfa_factor_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON mfa_factor_history FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0018 — challenge log

```sql
-- services/auth/migrations/0018_mfa_challenge_log.sql

BEGIN;

CREATE TABLE mfa_challenge_log (
    id                  BIGSERIAL    PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL,
    challenge_id        UUID         NOT NULL UNIQUE,
    factor_id           UUID         REFERENCES mfa_factors(id),
    challenge_kind      TEXT         NOT NULL CHECK (challenge_kind IN ('totp','webauthn')),
    status              TEXT         NOT NULL CHECK (status IN ('pending','consumed','expired','failed')),
    issued_at           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ  NOT NULL,
    consumed_at         TIMESTAMPTZ,
    source_ip_hash16    TEXT
);

CREATE INDEX mfa_challenge_log_subject_idx ON mfa_challenge_log (subject_id, issued_at DESC);
CREATE INDEX mfa_challenge_log_status_idx ON mfa_challenge_log (status, expires_at);

ALTER TABLE mfa_challenge_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_challenge_log_tenant_iso ON mfa_challenge_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON mfa_challenge_log FROM cyberos_app;
-- Status transitions are via privileged mfa_challenge_writer role (granted to handlers)
CREATE ROLE mfa_challenge_writer;
GRANT INSERT ON mfa_challenge_log TO mfa_challenge_writer, cyberos_app;
GRANT UPDATE (status, consumed_at) ON mfa_challenge_log TO mfa_challenge_writer;

COMMIT;
```

### 3.4 — Migration 0019 — recovery codes

```sql
-- services/auth/migrations/0019_mfa_recovery_codes.sql

BEGIN;

CREATE TABLE mfa_recovery_codes (
    id                  UUID         PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    code_bcrypt_hash    TEXT         NOT NULL,
    batch_id            UUID         NOT NULL,
    consumed            BOOLEAN      NOT NULL DEFAULT false,
    consumed_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX mfa_recovery_subject_batch_idx ON mfa_recovery_codes (subject_id, batch_id, consumed);

ALTER TABLE mfa_recovery_codes ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_recovery_tenant_iso ON mfa_recovery_codes
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Update of `consumed` is the only allowed mutation
REVOKE DELETE ON mfa_recovery_codes FROM cyberos_app;
REVOKE UPDATE ON mfa_recovery_codes FROM cyberos_app;
GRANT UPDATE (consumed, consumed_at) ON mfa_recovery_codes TO cyberos_app;

COMMIT;
```

### 3.5 — Migration 0020 — lockout state

```sql
-- services/auth/migrations/0020_mfa_lockout_state.sql

BEGIN;

CREATE TABLE mfa_lockout_state (
    subject_id           UUID         PRIMARY KEY REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    tenant_id            UUID         NOT NULL,
    failed_count         INT          NOT NULL DEFAULT 0 CHECK (failed_count >= 0),
    window_started_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    locked_until         TIMESTAMPTZ,
    last_attempt_at      TIMESTAMPTZ
);

CREATE INDEX mfa_lockout_state_locked_idx ON mfa_lockout_state (locked_until) WHERE locked_until IS NOT NULL;

ALTER TABLE mfa_lockout_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_lockout_state_tenant_iso ON mfa_lockout_state
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true');

REVOKE UPDATE, DELETE ON mfa_lockout_state FROM cyberos_app;
CREATE ROLE mfa_lockout_writer;
GRANT INSERT, UPDATE (failed_count, window_started_at, locked_until, last_attempt_at) ON mfa_lockout_state TO mfa_lockout_writer;
GRANT SELECT ON mfa_lockout_state TO cyberos_app, mfa_lockout_writer;

COMMIT;
```

### 3.6 — TOTP module

```rust
// services/auth/src/mfa/totp.rs
use hmac::{Hmac, Mac};
use sha1::Sha1;
use base32::Alphabet;

type HmacSha1 = Hmac<Sha1>;

pub const TIME_STEP_SECONDS: u64 = 30;
pub const CODE_DIGITS: u32 = 6;
pub const SKEW_TOLERANCE: i64 = 1;   // ±1 step (DEC-489)

pub fn generate_secret() -> Vec<u8> {
    use rand::RngCore;
    let mut secret = vec![0u8; 20];   // 160 bits per RFC 6238 recommendation
    rand::thread_rng().fill_bytes(&mut secret);
    secret
}

pub fn provisioning_uri(tenant_slug: &str, subject_email: &str, secret: &[u8]) -> String {
    let b32 = base32::encode(Alphabet::RFC4648 { padding: false }, secret);
    let label = format!("CyberOS:{tenant_slug}/{subject_email}");
    format!(
        "otpauth://totp/{label}?secret={b32}&issuer=CyberOS&algorithm=SHA1&digits={CODE_DIGITS}&period={TIME_STEP_SECONDS}",
        label = urlencoding::encode(&label),
    )
}

/// Compute TOTP code for a given Unix timestamp.
pub fn compute_code(secret: &[u8], unix_ts: u64) -> u32 {
    let counter = unix_ts / TIME_STEP_SECONDS;
    let counter_bytes = counter.to_be_bytes();
    let mut mac = HmacSha1::new_from_slice(secret).expect("valid key");
    mac.update(&counter_bytes);
    let hmac = mac.finalize().into_bytes();
    let offset = (hmac[19] & 0x0f) as usize;
    let binary = ((hmac[offset] as u32 & 0x7f) << 24)
        | ((hmac[offset + 1] as u32) << 16)
        | ((hmac[offset + 2] as u32) << 8)
        | (hmac[offset + 3] as u32);
    binary % 10u32.pow(CODE_DIGITS)
}

/// Verify against current ±SKEW_TOLERANCE steps.
pub fn verify(secret: &[u8], submitted_code: u32, now_unix_ts: u64) -> bool {
    let current_step = (now_unix_ts / TIME_STEP_SECONDS) as i64;
    for delta in -SKEW_TOLERANCE..=SKEW_TOLERANCE {
        let step = current_step + delta;
        if step < 0 { continue; }
        let candidate_ts = (step as u64) * TIME_STEP_SECONDS;
        if compute_code(secret, candidate_ts) == submitted_code {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 §B test vectors (SHA-1, 20-byte ASCII secret "12345678901234567890")
    const RFC6238_SECRET_SHA1: &[u8] = b"12345678901234567890";

    #[test]
    fn rfc6238_t_59() {
        // T=59 → counter=1 → expected code 94287082
        assert_eq!(compute_code(RFC6238_SECRET_SHA1, 59) % 100_000_000, 94287082);
    }

    #[test]
    fn rfc6238_t_1111111109() {
        assert_eq!(compute_code(RFC6238_SECRET_SHA1, 1111111109) % 100_000_000, 7081804);
    }

    #[test]
    fn verify_accepts_current() {
        let secret = b"12345678901234567890";
        let code = compute_code(secret, 1700000000) % 1_000_000;
        assert!(verify(secret, code, 1700000000));
    }

    #[test]
    fn verify_accepts_within_skew() {
        let secret = b"12345678901234567890";
        let code = compute_code(secret, 1700000000) % 1_000_000;
        assert!(verify(secret, code, 1700000030));   // +1 step
        assert!(verify(secret, code, 1699999970));   // -1 step
    }

    #[test]
    fn verify_rejects_beyond_skew() {
        let secret = b"12345678901234567890";
        let code = compute_code(secret, 1700000000) % 1_000_000;
        assert!(!verify(secret, code, 1700000060));  // +2 steps
        assert!(!verify(secret, code, 1699999940));  // -2 steps
    }
}
```

### 3.7 — Lockout module

```rust
// services/auth/src/mfa/lockout.rs
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub const WINDOW_MINUTES: i64 = 15;
pub const FAIL_THRESHOLD: i32 = 5;
pub const LOCKOUT_MINUTES: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockoutState { Open, Locked { until: DateTime<Utc> } }

pub async fn check_state(pool: &sqlx::PgPool, subject_id: Uuid) -> anyhow::Result<LockoutState> {
    let row: Option<(i32, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT failed_count, window_started_at, locked_until FROM mfa_lockout_state WHERE subject_id = $1"
    ).bind(subject_id).fetch_optional(pool).await?;
    match row {
        None => Ok(LockoutState::Open),
        Some((_, _, Some(locked_until))) if locked_until > Utc::now() =>
            Ok(LockoutState::Locked { until: locked_until }),
        Some((_, _, _)) => Ok(LockoutState::Open),
    }
}

pub async fn record_failure(
    pool: &sqlx::PgPool, subject_id: Uuid, tenant_id: Uuid,
) -> anyhow::Result<LockoutState> {
    // sliding window: if window_started_at older than WINDOW_MINUTES, reset
    let now = Utc::now();
    let mut tx = pool.begin().await?;
    let row: Option<(i32, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT failed_count, window_started_at, locked_until FROM mfa_lockout_state WHERE subject_id = $1 FOR UPDATE"
    ).bind(subject_id).fetch_optional(&mut *tx).await?;

    let new_state = match row {
        None => {
            // Insert first failure
            sqlx::query(
                "INSERT INTO mfa_lockout_state (subject_id, tenant_id, failed_count, window_started_at, last_attempt_at)
                 VALUES ($1, $2, 1, $3, $3)"
            ).bind(subject_id).bind(tenant_id).bind(now).execute(&mut *tx).await?;
            LockoutState::Open
        }
        Some((failed_count, window_started_at, _)) => {
            let elapsed = now - window_started_at;
            let (new_count, new_window) = if elapsed > Duration::minutes(WINDOW_MINUTES) {
                (1, now)
            } else {
                (failed_count + 1, window_started_at)
            };
            if new_count >= FAIL_THRESHOLD {
                let locked_until = now + Duration::minutes(LOCKOUT_MINUTES);
                sqlx::query(
                    "UPDATE mfa_lockout_state
                     SET failed_count = $2, window_started_at = $3, locked_until = $4, last_attempt_at = $5
                     WHERE subject_id = $1"
                ).bind(subject_id).bind(new_count).bind(new_window).bind(locked_until).bind(now)
                 .execute(&mut *tx).await?;
                LockoutState::Locked { until: locked_until }
            } else {
                sqlx::query(
                    "UPDATE mfa_lockout_state
                     SET failed_count = $2, window_started_at = $3, last_attempt_at = $4
                     WHERE subject_id = $1"
                ).bind(subject_id).bind(new_count).bind(new_window).bind(now)
                 .execute(&mut *tx).await?;
                LockoutState::Open
            }
        }
    };
    tx.commit().await?;
    Ok(new_state)
}

pub async fn record_success(pool: &sqlx::PgPool, subject_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE mfa_lockout_state SET failed_count = 0, locked_until = NULL, last_attempt_at = now() WHERE subject_id = $1"
    ).bind(subject_id).execute(pool).await?;
    Ok(())
}

pub async fn unlock(pool: &sqlx::PgPool, subject_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE mfa_lockout_state SET failed_count = 0, locked_until = NULL WHERE subject_id = $1"
    ).bind(subject_id).execute(pool).await?;
    Ok(())
}
```

### 3.8 — Recovery codes module

```rust
// services/auth/src/mfa/recovery.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use uuid::Uuid;
use zeroize::Zeroizing;

const CODE_COUNT: usize = 10;
const CODE_LENGTH: usize = 8;
const BCRYPT_COST: u32 = 12;
const ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";  // exclude 0, 1, O, L

pub fn generate_batch() -> (Uuid, Vec<Zeroizing<String>>) {
    let batch_id = Uuid::new_v4();
    let mut rng = rand::thread_rng();
    let codes: Vec<Zeroizing<String>> = (0..CODE_COUNT).map(|_| {
        let s: String = (0..CODE_LENGTH).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect();
        Zeroizing::new(s)
    }).collect();
    (batch_id, codes)
}

pub fn hash_code(code: &str) -> anyhow::Result<String> {
    Ok(hash(code, BCRYPT_COST)?)
}

pub fn verify_code(code: &str, hash_str: &str) -> bool {
    verify(code, hash_str).unwrap_or(false)
}
```

### 3.9 — Handler scaffold (excerpt)

```rust
// services/auth/src/handlers/mfa.rs
use axum::{Json, extract::{Path, State}, http::StatusCode};
use crate::mfa::{lockout, totp, recovery, audit};
use cyberos_auth::rbac::Role;

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: uuid::Uuid,
    pub factor_id: uuid::Uuid,
    pub code: Option<String>,                     // TOTP path
    pub webauthn_response: Option<serde_json::Value>,  // WebAuthn path
}

pub async fn verify_challenge(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<VerifyRequest>,
) -> Result<StatusCode, ApiError> {
    // 1. Lockout check
    if let lockout::LockoutState::Locked { until } = lockout::check_state(&state.db, claims.subject_id()).await? {
        return Err(ApiError::MfaLocked { until });
    }

    // 2. Load challenge + check pending+unexpired
    let challenge = state.mfa.repo.load_challenge(req.challenge_id).await?
        .ok_or(ApiError::ChallengeNotFound)?;
    if challenge.status != "pending" {
        audit::emit_challenge_reused(claims.subject_id(), req.challenge_id).await;
        return Err(ApiError::ChallengeReused);
    }
    if challenge.expires_at < chrono::Utc::now() {
        state.mfa.repo.mark_challenge_expired(req.challenge_id).await?;
        return Err(ApiError::ChallengeExpired);
    }

    // 3. Verify based on factor_kind
    let factor = state.mfa.repo.load_factor(req.factor_id).await?
        .ok_or(ApiError::FactorNotFound)?;
    let ok = match factor.factor_kind {
        crate::mfa::factor_kind::FactorKind::Totp => {
            let code: u32 = req.code.ok_or(ApiError::CodeRequired)?.parse()?;
            let secret = state.kms.decrypt(&factor.totp_secret_kms_blob.unwrap()).await?;
            totp::verify(&secret, code, chrono::Utc::now().timestamp() as u64)
        }
        crate::mfa::factor_kind::FactorKind::WebauthnPlatform | crate::mfa::factor_kind::FactorKind::WebauthnCrossPlatform => {
            let resp = req.webauthn_response.ok_or(ApiError::WebauthnRespRequired)?;
            crate::mfa::webauthn::verify_assertion(&factor, &resp, &challenge).await?
        }
    };

    if ok {
        state.mfa.repo.mark_challenge_consumed(req.challenge_id).await?;
        lockout::record_success(&state.db, claims.subject_id()).await?;
        audit::emit_challenge_succeeded(claims.subject_id(), req.factor_id).await;
        Ok(StatusCode::OK)
    } else {
        state.mfa.repo.mark_challenge_failed(req.challenge_id).await?;
        let new_state = lockout::record_failure(&state.db, claims.subject_id(), claims.tenant_id()).await?;
        audit::emit_challenge_failed(claims.subject_id(), req.factor_id).await;
        if let lockout::LockoutState::Locked { until } = new_state {
            audit::emit_locked_out(claims.subject_id(), until).await;
            return Err(ApiError::MfaLocked { until });
        }
        Err(ApiError::ChallengeInvalid)
    }
}
```

---

## §4 — Acceptance criteria

1. **factor_kind enum closed at 3** — totp, webauthn_platform, webauthn_cross_platform.
2. **TOTP RFC 6238 test vectors pass** — known-secret/known-time → known-code.
3. **TOTP skew ±1 accepted** — codes from current ± 1 step verify; ±2 reject.
4. **TOTP secret KMS-encrypted at rest** — totp_secret_kms_blob never plaintext.
5. **TOTP secret never exposed after enrol** — GET /factors response excludes the blob.
6. **WebAuthn UV=required enforced** — enrolment payload without UV → reject.
7. **WebAuthn attestation=none accepted at slice 1** — `direct` rejected.
8. **WebAuthn counter monotonicity enforced** — counter ≤ stored → reject + revoke factor.
9. **Challenge TTL 5min** — verify after 5min → 401 challenge_expired.
10. **Challenge reuse rejected** — second verify on same challenge_id → 401 + sev-2.
11. **10 recovery codes generated at first enrol** — 8 chars base32 ASCII (excludes 0/1/O/L).
12. **Recovery code single-use** — second consumption of same code → 401.
13. **Recovery regen invalidates all prior** — new batch_id; old codes orphaned (kept for history).
14. **Lockout at 5 failures in 15min** — 5th fail → 423 mfa_locked + sev-1 audit.
15. **Lockout duration 30min** — verify before locked_until → 423; after → normal flow.
16. **Lockout counter resets on success** — failed_count → 0 after verified.
17. **Root-admin can unlock early** — POST /unlock by non-root-admin → 403.
18. **Per-tenant MFA policy enforced** — subject with role in policy + no recent MFA → 401 mfa_challenge_required.
19. **Founder always requires MFA** — regardless of per-tenant policy.
20. **append-only mfa_factor_history** — UPDATE/DELETE blocked from cyberos_app.
21. **append-only mfa_challenge_log** — UPDATE allowed only via mfa_challenge_writer role.
22. **mfa_lockout_state writes via mfa_lockout_writer role only**.
23. **8 BRAIN audit kinds emit correctly** — one per lifecycle event.
24. **PII-scrubbed display_name + reason in BRAIN rows**.
25. **TOTP verify p95 < 100ms** — perf test.
26. **WebAuthn verify p95 < 300ms** — perf test.
27. **GET /factors never exposes secrets** — totp_secret_kms_blob + webauthn_public_key absent from response.
28. **TOTP enrol requires confirmation code** — wrong code on /enrol/finish → no row persisted.
29. **Last-factor removal invalidates recovery codes** — security invariant.
30. **OTel span emitted per handler** — outcome attribute populated.

---

## §5 — Verification

```rust
// services/auth/tests/mfa_totp_rfc6238_test.rs
use cyberos_auth::mfa::totp;

#[test]
fn rfc6238_test_vector_t_59() {
    let secret = b"12345678901234567890";
    let code = totp::compute_code(secret, 59);
    // RFC 6238 §B expected truncated to 8 digits: 94287082
    assert_eq!(code, 94287082 % 1_000_000);
}

#[test]
fn rfc6238_test_vector_t_1111111109() {
    let secret = b"12345678901234567890";
    let code = totp::compute_code(secret, 1111111109);
    assert_eq!(code, 7081804 % 1_000_000);
}
```

```rust
// services/auth/tests/mfa_lockout_5_in_15_test.rs
#[tokio::test]
async fn five_failures_in_15min_triggers_lockout(ctx: TestCtx) {
    let subject_id = ctx.create_test_subject().await;
    for _ in 0..4 {
        let state = cyberos_auth::mfa::lockout::record_failure(&ctx.pool, subject_id, ctx.tenant).await.unwrap();
        assert!(matches!(state, cyberos_auth::mfa::lockout::LockoutState::Open));
    }
    let state = cyberos_auth::mfa::lockout::record_failure(&ctx.pool, subject_id, ctx.tenant).await.unwrap();
    assert!(matches!(state, cyberos_auth::mfa::lockout::LockoutState::Locked { .. }));
    let rows = ctx.brain_audit_rows("auth.mfa_locked_out").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["severity"], "sev-1");
}

#[tokio::test]
async fn failures_outside_window_dont_accumulate(ctx: TestCtx) {
    let subject_id = ctx.create_test_subject().await;
    cyberos_auth::mfa::lockout::record_failure(&ctx.pool, subject_id, ctx.tenant).await.unwrap();
    ctx.advance_clock_minutes(16).await;   // outside 15min window
    let state = cyberos_auth::mfa::lockout::record_failure(&ctx.pool, subject_id, ctx.tenant).await.unwrap();
    assert!(matches!(state, cyberos_auth::mfa::lockout::LockoutState::Open));
}
```

```rust
// services/auth/tests/mfa_challenge_replay_rejected_test.rs
#[tokio::test]
async fn consumed_challenge_cannot_be_reused(ctx: TestCtx) {
    let factor = ctx.create_totp_factor().await;
    let challenge = ctx.issue_challenge(factor.id).await;
    ctx.verify_challenge(challenge.id, factor.id, &valid_totp_code()).await.unwrap();
    // Try replay
    let err = ctx.verify_challenge(challenge.id, factor.id, &valid_totp_code()).await.unwrap_err();
    assert!(format!("{err:?}").contains("ChallengeReused"));
    let rows = ctx.brain_audit_rows("auth.mfa_challenge_failed").await;
    assert!(rows.iter().any(|r| r["reason"] == "challenge_reused"));
}
```

```rust
// services/auth/tests/mfa_webauthn_counter_monotonic_test.rs
#[tokio::test]
async fn cloned_authenticator_detected(ctx: TestCtx) {
    let factor = ctx.create_webauthn_factor_with_counter(100).await;
    // First valid use: counter advances to 101
    ctx.verify_webauthn(factor.id, /* counter */ 101).await.unwrap();
    // Replay with counter 100 (cloned authenticator)
    let err = ctx.verify_webauthn(factor.id, 100).await.unwrap_err();
    assert!(format!("{err:?}").contains("cloned_authenticator_detected"));
    let post = ctx.fetch_factor(factor.id).await;
    assert_eq!(post.status, "removed");
}
```

```rust
// services/auth/tests/mfa_recovery_codes_single_use_test.rs
#[tokio::test]
async fn recovery_code_single_use(ctx: TestCtx) {
    let (codes, _batch_id) = ctx.enrol_factor_and_get_recovery_codes().await;
    let code = codes[0].clone();
    ctx.consume_recovery(&code).await.unwrap();
    let err = ctx.consume_recovery(&code).await.unwrap_err();
    assert!(format!("{err:?}").contains("recovery_code_already_consumed"));
}

#[tokio::test]
async fn regen_invalidates_all_prior(ctx: TestCtx) {
    let (codes, _) = ctx.enrol_factor_and_get_recovery_codes().await;
    let _ = ctx.regenerate_recovery_codes().await.unwrap();
    let err = ctx.consume_recovery(&codes[0]).await.unwrap_err();
    assert!(format!("{err:?}").contains("recovery_code_invalid"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 8 BRAIN row builders follow the canonical pattern; webauthn-rs handles WebAuthn cryptographic plumbing.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-002** — subject creation; this FR's factor rows reference auth.subjects.

**Downstream (1 placeholder):**
- **FR-AUTH-105** — passkey enrolment + login (uses this FR's WebAuthn `resident_key=preferred` foundation).

**Cross-module:**
- **FR-AUTH-004** — JWT issuer consults MFA policy before issuing.
- **FR-AUTH-101** — RBAC; founder role MFA-always; root-admin role required for unlock.
- **FR-AI-003** — BRAIN audit bridge.
- **FR-BRAIN-111** — PII scrubbing.
- **FR-AI-005** — per-tenant policy YAML (mfa_required_roles).
- **FR-OBS-007** — sev-1 alarms on lockout; sev-2 on recovery consumption + reuse.

---

## §8 — Example payloads

### 8.1 — POST /v1/auth/mfa/factors/totp/enrol

```json
{ "display_name": "Authy on iPhone 15" }
```

Response:

```json
{
  "factor_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "provisioning_uri": "otpauth://totp/CyberOS:acme-corp/alice@acme.example?secret=JBSWY3DPEHPK3PXP&issuer=CyberOS&algorithm=SHA1&digits=6&period=30",
  "recovery_codes": [
    "K7N9PQRS", "T4VWXYZ2", "3456ABCD", "EFGHJKLM", "NPQRSTUV",
    "WXYZ2345", "6789ABCD", "EFGHJKLM", "NPQRSTUV", "WXYZ2345"
  ],
  "recovery_codes_displayed_once": true
}
```

### 8.2 — POST /v1/auth/mfa/factors/totp/enrol/finish

```json
{ "factor_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M", "confirmation_code": "527821" }
```

### 8.3 — auth.mfa_factor_enrolled BRAIN row

```json
{
  "kind": "auth.mfa_factor_enrolled",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "factor_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "factor_kind": "totp",
  "display_name_scrubbed": "[REDACTED-DEVICE]",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — auth.mfa_locked_out BRAIN row (sev-1)

```json
{
  "kind": "auth.mfa_locked_out",
  "severity": "sev-1",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "failed_count": 5,
  "locked_until": "2026-05-16T13:00:00Z",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — POST /v1/auth/mfa/unlock (root-admin)

```json
{ "subject_id": "9b1deb4d-...", "reason": "User contacted support; identity verified via callback" }
```

---

## §9 — Open questions

Deferred:
- **WebAuthn direct attestation for high-security tenants** — FR-AUTH-2xx.
- **SMS / email OTP factor** — out of scope (phishable); ADR required.
- **Biometric without WebAuthn** — out of scope.
- **Per-tenant lockout policy override** — slice 2.
- **MFA recovery via verified email link** — FR-AUTH-2xx.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| TOTP secret KMS decrypt fail | aws-sdk error | 500 + sev-1 | KMS health |
| TOTP code outside skew | verifier | 401 invalid_code + counter increment | Re-enter |
| TOTP code at boundary (±1 step) | verifier | Accept | Designed |
| Challenge expired (> 5min) | TTL check | 401 challenge_expired | Re-issue |
| Challenge reused (consumed) | status check | 401 + sev-2 audit | Re-issue |
| WebAuthn counter not monotonic | counter compare | Revoke factor + sev-1 audit | Re-enrol |
| WebAuthn UV missing | attestation parse | 401 user_verification_required | Re-attempt |
| WebAuthn signature invalid | webauthn-rs error | 401 signature_invalid | Re-attempt |
| 5 fails in 15min | counter check | 423 mfa_locked + sev-1 | Wait 30min or root-admin unlock |
| Locked subject attempts verify | locked_until check | 423 + counter NOT incremented | Wait |
| Recovery code reused | consumed=true check | 401 + audit | Generate new batch |
| Recovery regen mid-flow | batch_id swap | All prior codes orphaned | Designed |
| Last factor removed but recovery codes valid | factor-removal hook | Invalidate recovery codes | Re-enrol |
| Per-tenant policy missing | YAML lookup | Default deny-MFA-required | Operator adds policy |
| Founder bypass attempt | policy hard-codes | 401 mfa_challenge_required | Designed |
| TOTP secret leak via API | handler omits | None | Designed |
| WebAuthn public key in API | handler omits | None | Designed |
| Append-only history UPDATE from app | SQL grant | permission denied | Designed |
| append-only challenge_log UPDATE from app | SQL grant | permission denied | Designed |
| Lockout state mutated from app | SQL grant | permission denied | Designed |
| Concurrent challenge issuance | UNIQUE on challenge_id | Designed | None |
| Concurrent verify attempts | row lock | Serial | Designed |
| Non-root-admin unlock attempt | role check | 403 | Designed |
| Cross-tenant factor lookup | RLS | 0 rows | Designed |
| BRAIN audit fail mid-tx | rollback | 500 + retry | brain_writer health |
| TOTP confirmation_code mismatch on enrol | handler | 400 confirmation_code_mismatch + no row persisted | Re-attempt |
| WebAuthn challenge replay | challenge_id reuse | 401 + sev-2 | Designed |
| Subject deleted while factors exist | FK RESTRICT | DELETE auth.subjects fails | Soft-delete via status |
| Recovery codes exceed 10 | generator bound | Code generator returns exactly 10 | Test asserts |
| OTel span attribute missing | otel_test | CI fails | Fix |
| Sev-1 lockout alarm not firing | OBS rule | CI fails | Fix rule |
| display_name > 80 chars | DB CHECK | INSERT fails | Shorten |
| WebAuthn AAGUID missing | optional field | Stored as NULL | None |

---

## §11 — Implementation notes

- **TOTP HMAC-SHA1 30s/6digit** — RFC 6238 compatible with all major authenticator apps.
- **±1 step skew** — clock-drift tolerance without brute-force window expansion.
- **160-bit secret, base32-encoded** — RFC 6238 recommendation.
- **KMS-encrypt at rest, never expose post-enrol** — secret leak resistance.
- **WebAuthn Level 3 + webauthn-rs crate** — standards-compliant + battle-tested.
- **UV=required + attestation=none at slice 1** — AAL3 user verification + suitable for our risk model.
- **Counter-monotonicity check** — W3C-defined cloned-authenticator defense.
- **Challenge TTL 5min + reuse rejected at sev-2** — replay defense.
- **10 recovery codes + bcrypt cost 12** — same parameter as FR-AUTH-002 password.
- **Regen invalidates all** — unambiguous bookkeeping.
- **5/15 → 30min lockout sev-1** — brute-force defense + operator alerting.
- **Root-admin-only unlock** — bypass-resistance.
- **Per-tenant policy via tenant_policy YAML** — flexibility without code fork.
- **Founder always-MFA** — hard-coded design assertion.
- **Append-only via SQL grants** — forensic integrity.
- **8 BRAIN audit kinds** — selective operator queries.
- **PII scrub display_name + reason** — chain holds scrubbed.
- **mfa_lockout_writer role split** — app code can't tamper with lockout.
- **mfa_challenge_writer role split** — challenge status transitions only via handler.
- **TOTP confirm-on-enrol** — misconfiguration catch.
- **Last-factor removal invalidates recovery** — security invariant.
- **NEVER expose secrets/public keys in API** — handler-side filter.
- **`webauthn-rs` for crypto plumbing** — avoid hand-rolled WebAuthn.

---

*End of FR-AUTH-102.*
