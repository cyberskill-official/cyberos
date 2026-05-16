---
id: FR-AUTH-105
title: "AUTH Passkey enrolment + login — discoverable credentials (resident keys) + autofill UI + cross-platform sync + closed enrolment FSM + downgrade-resistance + BRAIN audit per lifecycle event"
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
related_frs: [FR-AUTH-002, FR-AUTH-004, FR-AUTH-101, FR-AUTH-102, FR-AI-003, FR-BRAIN-101, FR-OBS-007]
depends_on: [FR-AUTH-102]
blocks: []

source_pages:
  - website/docs/modules/auth.html#passkeys
  - https://www.w3.org/TR/webauthn-3 (WebAuthn Level 3)
  - https://passkeys.dev/ (FIDO Alliance + W3C reference)
source_decisions:
  - DEC-540 (passkey = WebAuthn discoverable credential (resident key) + user verification = required — synonymous in our spec with passkey UX)
  - DEC-541 (closed passkey_origin enum at 3 values: platform_synced (iCloud Keychain / Google Password Manager / Windows Hello with cloud sync) · platform_local (Touch ID / Face ID / Windows Hello without sync) · cross_platform (YubiKey / hardware FIDO2 key))
  - DEC-542 (autofill UI via `mediation: "conditional"` — browser surfaces passkeys in password autofill dropdown; user activates without explicit "use passkey" click)
  - DEC-543 (downgrade-resistance — once a subject enrols a passkey, password-only login for that subject is rejected with `passkey_required`; explicit per-subject opt-out flag exists but defaults FALSE)
  - DEC-544 (per-subject max 5 active passkeys — typical user has phone + laptop + work-laptop + recovery key + one spare; ADR required to raise)
  - DEC-545 (passkey enrolment FSM: requested → confirmed (first use) | abandoned (24h timeout) — only confirmed passkeys count toward login + downgrade-resistance)
  - DEC-546 (BRAIN audit kinds: auth.passkey_enrolment_requested, auth.passkey_enrolment_confirmed, auth.passkey_enrolment_abandoned, auth.passkey_login_succeeded, auth.passkey_login_failed, auth.passkey_removed, auth.passkey_downgrade_blocked, auth.passkey_autofill_used)
  - DEC-547 (REVOKE UPDATE, DELETE on passkey_lifecycle_log from cyberos_app — append-only at SQL grant)
  - DEC-548 (cross-platform key recovery — if subject loses ALL synced passkeys + cross-platform keys, FR-AUTH-102 recovery codes are the only recovery path; passkey-only subjects without recovery codes risk lockout; UI MUST warn during enrolment)
  - DEC-549 (per-tenant `passkey_required_for_roles: [<role>...]` policy in tenant_policy YAML — when subject's roles intersect, password login is blocked; founder always per FR-AUTH-101 DEC-128)
  - DEC-550 (downgrade-blocked attempts emit sev-2 BRAIN row — repeated attempts may signal phishing-via-password trying to bypass passkey)
  - DEC-551 (autofill `mediation: "conditional"` requires `user_verification: "required"` per W3C spec — biometric + autofill = phishing-resistant)
  - DEC-552 (passkey removal requires re-authentication with another passkey or fresh full MFA flow — prevents session-hijack passkey wipe)
  - W3C WebAuthn Level 3 (2024); FIDO2 / CTAP2.1; passkeys.dev best practices
  - NIST SP 800-63B (AAL3 for cross-platform hardware keys; AAL2 for platform-synced)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0025_passkey_enrolment_state.sql
  - services/auth/migrations/0026_passkey_lifecycle_log.sql
  - services/auth/src/passkey/mod.rs
  - services/auth/src/passkey/enrolment.rs                       # enrolment FSM (requested → confirmed | abandoned)
  - services/auth/src/passkey/login.rs                           # discoverable credential login flow
  - services/auth/src/passkey/autofill.rs                        # conditional-mediation flow
  - services/auth/src/passkey/downgrade_gate.rs                  # password-login block when passkey enrolled
  - services/auth/src/passkey/origin.rs                          # passkey_origin enum + AAGUID lookup
  - services/auth/src/passkey/audit.rs                           # 8 BRAIN row builders
  - services/auth/src/passkey/repo.rs                            # CRUD
  - services/auth/src/handlers/passkey.rs                        # enrol/login/list/remove + autofill
  - services/auth/tests/passkey_enrolment_fsm_test.rs
  - services/auth/tests/passkey_login_discoverable_test.rs
  - services/auth/tests/passkey_autofill_conditional_test.rs
  - services/auth/tests/passkey_downgrade_block_test.rs
  - services/auth/tests/passkey_max_5_per_subject_test.rs
  - services/auth/tests/passkey_origin_enum_closed_test.rs
  - services/auth/tests/passkey_removal_requires_reauth_test.rs
  - services/auth/tests/passkey_abandonment_24h_test.rs
  - services/auth/tests/passkey_per_tenant_policy_test.rs
  - services/auth/tests/passkey_append_only_log_test.rs
  - services/auth/tests/passkey_audit_emission_test.rs
modified_files:
  - services/auth/src/mfa/factor_kind.rs                         # cross-reference passkey factor with WebAuthn family from FR-AUTH-102
  - services/auth/src/lib.rs                                     # pub mod passkey
  - services/auth/src/jwt/issuer.rs                              # downgrade-gate hook at password login

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test passkey

disallowed_tools:
  - allow passkey enrolment without resident-key requirement (per DEC-540)
  - allow conditional mediation without UV=required (per DEC-551)
  - allow > 5 passkeys per subject (per DEC-544)
  - bypass downgrade-resistance via env override (per DEC-543)
  - allow passkey removal without re-authentication (per DEC-552)

effort_hours: 8
sub_tasks:
  - "0.4h: 0025_passkey_enrolment_state.sql + closed enrolment_state enum"
  - "0.3h: 0026_passkey_lifecycle_log.sql append-only"
  - "0.5h: enrolment.rs — FSM with 24h abandonment timeout"
  - "0.8h: login.rs — discoverable credential resolution (no email required at login start)"
  - "0.6h: autofill.rs — conditional mediation params + UV-required enforcement"
  - "0.6h: downgrade_gate.rs — password-login block when passkey enrolled"
  - "0.4h: origin.rs — passkey_origin enum + common AAGUID map for platform_synced detection"
  - "0.5h: audit.rs — 8 BRAIN builders"
  - "0.4h: repo.rs"
  - "0.7h: handlers/passkey.rs — full REST surface"
  - "0.3h: jwt/issuer.rs downgrade-gate integration"
  - "2.5h: tests — 11 test files"

risk_if_skipped: "Passkeys are the leading-edge phishing-resistant authentication primitive in 2026; without DEC-540's discoverable credentials + DEC-542's autofill UI, the FR-AUTH-102 WebAuthn substrate ships unused. FR-AUTH-101's founder-role passkey requirement (DEC-128) is inert without an enrolment + login flow. Without DEC-543's downgrade-resistance, an attacker who steals a password skips the passkey entirely. Without DEC-552's re-authentication for removal, a session-hijack lets the attacker wipe legitimate passkeys + re-enrol their own. Without DEC-548's recovery-code warning, passkey-only subjects discover the lockout only when they lose their device. The 8h effort lands the modern auth UX that downstream FR-PORTAL-* + FR-CRM-005 + (every external-facing surface) depend on for credibility."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship passkey enrolment + login via WebAuthn discoverable credentials (resident keys) with autofill UI + downgrade-resistance + closed enrolment FSM + recovery warnings. Each requirement:

1. **MUST** define `passkey_enrolment_state` table: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL REFERENCES auth.subjects(id), factor_id UUID REFERENCES mfa_factors(id), challenge BYTEA NOT NULL, passkey_origin passkey_origin, state TEXT NOT NULL CHECK (state IN ('requested','confirmed','abandoned')), requested_at TIMESTAMPTZ NOT NULL DEFAULT now(), abandoned_at TIMESTAMPTZ, confirmed_at TIMESTAMPTZ, expires_at TIMESTAMPTZ NOT NULL)`. Inserted at enrolment start; transitions to `confirmed` on first successful use within 24h, `abandoned` if expires_at passes without confirmation.

2. **MUST** declare the closed `passkey_origin` Postgres enum with exactly 3 values (per DEC-541): `'platform_synced'`, `'platform_local'`, `'cross_platform'`. Adding a 4th is an ADR.

3. **MUST** define `passkey_lifecycle_log` table: `(id BIGSERIAL, tenant_id UUID, subject_id UUID, factor_id UUID, event TEXT NOT NULL, passkey_origin passkey_origin, source_ip_hash16 TEXT, ts TIMESTAMPTZ)`. `REVOKE UPDATE, DELETE FROM cyberos_app` (per DEC-547).

4. **MUST** enforce RLS with `USING + WITH CHECK` on both tables; root-admin escape clause.

5. **MUST** require discoverable credentials at enrolment (per DEC-540). WebAuthn creation challenge MUST set `authenticatorSelection: { residentKey: "required", userVerification: "required" }`. Authenticators not supporting resident keys (rare in 2026) → 400 `authenticator_unsupported`.

6. **MUST** ship `POST /v1/auth/passkey/enrol/begin` handler (per DEC-545):
    - Generates WebAuthn creation challenge with `residentKey=required + userVerification=required + attestation=none`.
    - INSERT `passkey_enrolment_state` row with `state='requested'`, `expires_at = now() + 24 hours`.
    - Returns `PublicKeyCredentialCreationOptions` for browser to invoke `navigator.credentials.create()`.
    - Emit `auth.passkey_enrolment_requested` BRAIN row.

7. **MUST** ship `POST /v1/auth/passkey/enrol/finish` handler (per DEC-545):
    - Receives `PublicKeyCredential` from browser.
    - Validates attestation per FR-AUTH-102's WebAuthn module.
    - Detects `passkey_origin` from AAGUID lookup (per origin.rs map).
    - Creates `mfa_factors` row with `factor_kind = webauthn_platform` or `webauthn_cross_platform` per FR-AUTH-102.
    - UPDATE `passkey_enrolment_state` to `state='confirmed', confirmed_at=now()`.
    - Emit `auth.passkey_enrolment_confirmed` BRAIN row.
    - If subject has zero recovery codes → return WARNING in response body suggesting recovery-code generation (per DEC-548).

8. **MUST** ship background job to mark abandoned enrolments. Hourly: `UPDATE passkey_enrolment_state SET state='abandoned', abandoned_at=now() WHERE state='requested' AND expires_at < now()`. Per row, emit `auth.passkey_enrolment_abandoned` BRAIN row (sev-3 informational).

9. **MUST** ship `POST /v1/auth/passkey/login/begin` handler (per DEC-540):
    - Generates WebAuthn assertion challenge with `allowCredentials: []` (empty — discoverable credential mode).
    - `userVerification: "required"`.
    - Returns `PublicKeyCredentialRequestOptions` for browser.
    - Does NOT require subject email/username at this step (resident-key resolution).

10. **MUST** ship `POST /v1/auth/passkey/login/finish` handler:
    - Receives assertion from browser including `userHandle` (= subject_id).
    - Lookup `mfa_factors` by credential_id; validate signature via FR-AUTH-102's verifier.
    - Counter-monotonicity check (per FR-AUTH-102 §1 #10).
    - On success: issue FR-AUTH-004 JWT; emit `auth.passkey_login_succeeded`; update `mfa_factors.last_used_at`.
    - On fail: emit `auth.passkey_login_failed` with reason; counter increment for FR-AUTH-102 lockout.

11. **MUST** support **autofill conditional mediation** (per DEC-542 + DEC-551):
    - `POST /v1/auth/passkey/autofill-options` returns options with `mediation: "conditional"` for browsers supporting WebAuthn autofill (Chrome 108+, Safari 16+).
    - `user_verification: "required"` is mandatory for conditional mediation per W3C spec — enforced at handler.
    - On successful autofill login, emit `auth.passkey_autofill_used` BRAIN row (informational; useful for adoption metrics).

12. **MUST** enforce **downgrade-resistance** (per DEC-543 + DEC-550). When subject attempts password-only login:
    - JWT issuer (FR-AUTH-004) consults `downgrade_gate::is_passkey_required(subject_id)`.
    - Predicate: subject has ≥ 1 confirmed passkey AND per-tenant policy doesn't have a per-subject opt-out flag set.
    - True → 401 `passkey_required` + emit `auth.passkey_downgrade_blocked` BRAIN row (sev-2 per DEC-550).
    - Repeated attempts: alarm sev-2 at > 5 within 1 hour for a subject (phishing signal).

13. **MUST** enforce max 5 active passkeys per subject (per DEC-544). 6th enrolment attempt → 409 `passkey_limit_exceeded` with body listing existing passkey display_names. Removal of an existing passkey first → enrolment proceeds.

14. **MUST** ship `DELETE /v1/auth/passkey/factors/{id}` handler (per DEC-552):
    - Caller MUST present a fresh MFA proof in the request (header `X-MFA-Challenge-Token: <recently-verified-challenge-id>`). Challenge must be < 5 minutes old AND must have been verified for THIS subject.
    - Missing/stale proof → 401 `recent_mfa_required`.
    - On success: soft-delete passkey (mfa_factors.status → 'removed'); emit `auth.passkey_removed` BRAIN row; if this was the last passkey for the subject + per-tenant policy requires passkey → emit warning to operator.

15. **MUST** detect passkey_origin via AAGUID lookup (per DEC-541). The origin.rs module ships a map of well-known AAGUIDs → origin:
    - Apple iCloud Keychain AAGUID → `platform_synced`.
    - Google Password Manager (Chrome on Android) → `platform_synced`.
    - Windows Hello + Microsoft sync → `platform_synced`.
    - macOS Touch ID local (no iCloud sync) → `platform_local`.
    - YubiKey series → `cross_platform`.
    - Unknown AAGUID → `cross_platform` (conservative default; logged for analyst review).

16. **MUST** consult per-tenant `passkey_required_for_roles: [<role>...]` policy from tenant_policy YAML (per DEC-549). If subject's roles intersect this set AND subject has zero passkeys → JWT issuance returns 401 `passkey_enrolment_required` with hint to call enrol/begin. Founder role always per FR-AUTH-101 DEC-128.

17. **MUST** emit 8 BRAIN audit row kinds (per DEC-546):
    - `auth.passkey_enrolment_requested` — enrol/begin.
    - `auth.passkey_enrolment_confirmed` — enrol/finish success.
    - `auth.passkey_enrolment_abandoned` — sev-3 informational.
    - `auth.passkey_login_succeeded` — login/finish success.
    - `auth.passkey_login_failed` — login/finish failure.
    - `auth.passkey_removed` — DELETE success.
    - `auth.passkey_downgrade_blocked` — password-login rejected (sev-2).
    - `auth.passkey_autofill_used` — conditional mediation success.

18. **MUST** PII-scrub `display_name` (inherited from FR-AUTH-102's display_name) before chain commit.

19. **MUST** ship `GET /v1/auth/passkey/factors` returning subject's passkeys: `[{factor_id, display_name, passkey_origin, enrolled_at, last_used_at, status}]`. NEVER expose credential public key or AAGUID via API (operator-only via direct DB query).

20. **MUST** enforce no downgrade-resistance bypass via env (per DEC-543). The `downgrade_gate::is_passkey_required` function is hard-coded — no env override accepted. Per-subject opt-out flag exists in the schema (defaulting to FALSE) but setting it requires root-admin role + sev-2 audit row.

21. **MUST** complete enrolment + login handlers in ≤ 200 ms p95 (excludes browser-side WebAuthn ceremony). `passkey_perf_test`.

22. **MUST** emit OTel span `auth.passkey.{enrol_begin,enrol_finish,login_begin,login_finish,remove,downgrade_blocked,autofill}` with `outcome` attribute (success | abandoned | timeout | invalid_attestation | limit_exceeded | downgrade_blocked | recent_mfa_required | passkey_required | passkey_enrolment_required).

23. **MUST** emit OTel metrics:
    - `auth_passkey_enrolment_total{tenant_id, passkey_origin, outcome}` (counter).
    - `auth_passkey_login_total{tenant_id, passkey_origin, outcome}` (counter).
    - `auth_passkey_downgrade_blocked_total{tenant_id, subject_id_hash16}` (counter — sev-2 alarm at > 5/h per subject).
    - `auth_passkey_active_count{tenant_id, passkey_origin}` (gauge — adoption metric).
    - `auth_passkey_autofill_used_total{tenant_id}` (counter — UX adoption metric).
    - `auth_passkey_enrolment_latency_ms` (histogram).

24. **MUST** warn during enrolment if subject has zero MFA recovery codes (per DEC-548 + §1 #7). Response body includes `recovery_warning: true, recommended_action: "POST /v1/auth/mfa/recovery-codes/regen"`. Frontend SHOULD display warning prominently.

25. **MUST** support **per-subject opt-out flag** (`mfa_factors.passkey_downgrade_optout BOOLEAN DEFAULT false`). Setting requires root-admin role + sev-2 BRAIN row + reason. Used for exceptional cases (e.g. legacy integration that cannot WebAuthn).

26. **MUST** ship `POST /v1/auth/passkey/downgrade-optout` for root-admin to set the flag. Body `{subject_id, enabled, reason}`. Validates reason (1–500 chars); emit `auth.passkey_downgrade_optout_changed` audit row.

---

## §2 — Why this design (rationale for humans)

**Why discoverable credentials mandatory (DEC-540, §1 #5)?** Discoverable credentials enable resident-key UX — user picks "Sign in with passkey" without typing username first. This is the passkey UX that competes with password autofill. Non-resident keys require username first → worse UX than passwords → adoption fails. Requiring resident-key at enrolment ensures the UX promise holds.

**Why 3-value closed passkey_origin enum (DEC-541)?** Operator analytics need origin granularity: "how many users are on platform_synced (recoverable via cloud)?" vs "platform_local (single-device risk)?" vs "cross_platform (hardware key)?". Adding a 4th (e.g. `external_browser_extension` for password managers) is an ADR — most fit the 3 categories.

**Why autofill conditional mediation (DEC-542, §1 #11)?** Browser autofill is the primary discovery surface for new passkey users. Without it, users don't know they have passkeys until prompted. Conditional mediation surfaces in password field dropdown — zero new UI required. Adoption metric `auth_passkey_autofill_used_total` tracks this.

**Why downgrade-resistance (DEC-543, §1 #12)?** Phishing remains the dominant credential attack. If user enrols a passkey then later types password on a phishing site, password works (no passkey enforcement) — passkey investment was waste. Downgrade-resistance: once you have a passkey, password login is rejected. Forces phishers to attack the passkey itself (which is unphishable by design).

**Why per-subject opt-out exists but defaults FALSE (DEC-543, §1 #25)?** Exceptional cases (legacy CLI tools that can't WebAuthn, edge-case integration). Defaulting FALSE means the security stance is "passkey-required-when-enrolled"; opt-out is explicit + audited + root-admin gated.

**Why sev-2 alarm on downgrade-blocked (DEC-550, §1 #12)?** Repeated password attempts on a passkey-enrolled subject signal (a) phishing-via-password, (b) credential-stuffing attack, (c) legitimate user confusion. All three warrant operator notice within an hour.

**Why max 5 passkeys per subject (DEC-544, §1 #13)?** Typical user: personal phone + work phone + personal laptop + work laptop + recovery hardware key = 5. > 5 is power-user; ADR to raise. Bound prevents unbounded growth of the credential list (UX + storage).

**Why removal requires fresh MFA (DEC-552, §1 #14)?** Session hijack scenario: attacker steals session token + uses it to remove victim's passkeys + enrol their own. Requiring fresh MFA (< 5 min) for removal prevents this — attacker without the second factor can't wipe.

**Why warn on enrolment if zero recovery codes (DEC-548, §1 #24)?** Passkey-only user without recovery codes who loses ALL devices → permanent lockout. Recovery codes are the bridge. The warning prompts user to set them up; explicit operator UX choice (don't silently leave users at risk).

**Why per-tenant `passkey_required_for_roles` policy (DEC-549, §1 #16)?** Different tenants have different risk tolerances. Tenant A: all employees require passkey. Tenant B: only financial-role subjects. Per-tenant config gives flexibility; founder role hard-codes baseline.

**Why AAGUID-based origin detection (DEC-541, §1 #15)?** AAGUID is the W3C-defined authenticator identifier. Apple, Google, Microsoft publish their AAGUIDs. Map them at enrolment time to populate origin column. Unknown AAGUID → conservative `cross_platform` default (e.g. brand-new YubiKey model).

**Why login/begin without email (§1 #9)?** Discoverable credential = browser already knows which subject's key matches the relying party. Asking for email first defeats the resident-key UX advantage. The `userHandle` field in the assertion response carries subject_id.

**Why same WebAuthn factor table from FR-AUTH-102 (§1 #7, modified_files)?** Passkeys ARE WebAuthn credentials. Storing in the same `mfa_factors` table preserves the unified factor view (subject's full credential inventory in one place). The passkey-specific lifecycle log is supplementary.

**Why enrolment FSM with abandonment timeout (DEC-545, §1 #8)?** Users start enrolment, get distracted, never finish. Unconfirmed rows would pile up. Hourly job marks them abandoned with audit row — clean lifecycle, no orphans.

**Why opt-out audit row at sev-2 (§1 #20)?** Operator setting opt-out for a subject = relaxing a security control. Sev-2 ensures it's visible in OBS digests; reason field captures the justification.

**Why credential public key never exposed via API (§1 #19)?** Public key + signature counter = enough for offline attestation forgery if combined with stolen private key. Defense-in-depth: API never returns the public key; operator queries DB directly with audit trail.

**Why "passkey_enrolment_required" 401 (§1 #16)?** Subject with required role but no passkey would otherwise see opaque 401 on JWT issuance. Specific error code + hint to call enrol/begin gives clear remediation. Frontend handles by redirecting to enrol UX.

**Why downgrade_gate cached inline at FR-AUTH-004 issuer (§1 #20)?** Hot path — JWT issuance happens on every login. Cache (60s TTL matching FR-AUTH-109 pattern) keeps it fast. Invalidate on passkey enrol/remove + opt-out change.

**Why platform_synced vs platform_local distinction (DEC-541)?** platform_synced = recoverable via cloud (lose device → restore on new device). platform_local = device-bound (lose device → lose credential). Operator analytics + UX warnings differ between the two.

**Why FR-AUTH-105 has empty `blocks: []`?** No downstream FRs explicitly depend on passkeys at slice 1 — passkeys are the UX layer on top of FR-AUTH-102 WebAuthn. Future FRs (FR-PORTAL-*) implicitly benefit but don't have a direct dependency.

**Why explicit "recovery_warning" in enrol response (§1 #24)?** Default frontend UI shows the warning; tenants with custom UX can decide how to surface (popover, modal, banner). Backend signals the intent; frontend executes.

**Why no SLO / passkey-side logout (§1 #14)?** Passkey is for authentication; session lifecycle is FR-AUTH-004's concern. Logout = expire token; no passkey-specific action needed.

---

## §3 — API contract

### 3.1 — Migration 0025 — enrolment_state

```sql
-- services/auth/migrations/0025_passkey_enrolment_state.sql

BEGIN;

CREATE TYPE passkey_origin AS ENUM ('platform_synced', 'platform_local', 'cross_platform');

CREATE TABLE passkey_enrolment_state (
    id                  UUID         PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL REFERENCES auth.subjects(id),
    factor_id           UUID         REFERENCES mfa_factors(id),
    challenge           BYTEA        NOT NULL,
    passkey_origin      passkey_origin,
    state               TEXT         NOT NULL CHECK (state IN ('requested','confirmed','abandoned')) DEFAULT 'requested',
    requested_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    abandoned_at        TIMESTAMPTZ,
    confirmed_at        TIMESTAMPTZ,
    expires_at          TIMESTAMPTZ  NOT NULL
);

CREATE INDEX passkey_enrolment_state_state_idx ON passkey_enrolment_state (state, expires_at);
CREATE INDEX passkey_enrolment_state_subject_idx ON passkey_enrolment_state (subject_id, state);

ALTER TABLE passkey_enrolment_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY passkey_enrolment_state_tenant_iso ON passkey_enrolment_state
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Extend mfa_factors (from FR-AUTH-102) with passkey-specific columns
ALTER TABLE mfa_factors ADD COLUMN passkey_origin passkey_origin;
ALTER TABLE mfa_factors ADD COLUMN passkey_downgrade_optout BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE mfa_factors ADD COLUMN optout_reason TEXT CHECK (optout_reason IS NULL OR length(optout_reason) BETWEEN 1 AND 500);

COMMIT;
```

### 3.2 — Migration 0026 — lifecycle log

```sql
-- services/auth/migrations/0026_passkey_lifecycle_log.sql

BEGIN;

CREATE TABLE passkey_lifecycle_log (
    id                  BIGSERIAL    PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL,
    factor_id           UUID,
    event               TEXT         NOT NULL,
    passkey_origin      passkey_origin,
    source_ip_hash16    TEXT,
    ts                  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX passkey_lifecycle_log_subject_idx ON passkey_lifecycle_log (subject_id, ts DESC);
CREATE INDEX passkey_lifecycle_log_event_idx ON passkey_lifecycle_log (event, ts DESC);

ALTER TABLE passkey_lifecycle_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY passkey_lifecycle_log_tenant_iso ON passkey_lifecycle_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON passkey_lifecycle_log FROM cyberos_app;

COMMIT;
```

### 3.3 — Origin detection

```rust
// services/auth/src/passkey/origin.rs
use uuid::Uuid;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::passkey::PasskeyOrigin;

static AAGUID_MAP: Lazy<HashMap<Uuid, PasskeyOrigin>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Apple iCloud Keychain (well-known AAGUIDs)
    m.insert(Uuid::parse_str("dd4ec289-e01d-41c9-bb89-70fa845d4bf2").unwrap(), PasskeyOrigin::PlatformSynced);
    m.insert(Uuid::parse_str("adce0002-35bc-c60a-648b-0b25f1f05503").unwrap(), PasskeyOrigin::PlatformSynced);
    // Google Password Manager (Chrome on Android)
    m.insert(Uuid::parse_str("ea9b8d66-4d01-1d21-3ce4-b6b48cb575d4").unwrap(), PasskeyOrigin::PlatformSynced);
    // Windows Hello (with Microsoft sync)
    m.insert(Uuid::parse_str("08987058-cadc-4b81-b6e1-30de50dcbe96").unwrap(), PasskeyOrigin::PlatformSynced);
    // YubiKey 5 series (cross-platform hardware key examples)
    m.insert(Uuid::parse_str("ee882879-721c-4913-9775-3dfcce97072a").unwrap(), PasskeyOrigin::CrossPlatform);
    m.insert(Uuid::parse_str("fa2b99dc-9e39-4257-8f92-4a30d23c4118").unwrap(), PasskeyOrigin::CrossPlatform);
    m
});

pub fn detect(aaguid: Uuid) -> PasskeyOrigin {
    *AAGUID_MAP.get(&aaguid).unwrap_or(&PasskeyOrigin::CrossPlatform)   // conservative default
}
```

### 3.4 — Enrolment FSM

```rust
// services/auth/src/passkey/enrolment.rs
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub const ENROLMENT_TTL_HOURS: i64 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrolmentState { Requested, Confirmed, Abandoned }

pub async fn begin_enrolment(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    challenge: &[u8],
) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(ENROLMENT_TTL_HOURS);
    sqlx::query(r#"
        INSERT INTO passkey_enrolment_state (id, tenant_id, subject_id, challenge, state, expires_at)
        VALUES ($1, $2, $3, $4, 'requested', $5)
    "#).bind(id).bind(tenant_id).bind(subject_id).bind(challenge).bind(expires_at)
       .execute(pool).await?;
    Ok(id)
}

pub async fn confirm_enrolment(
    pool: &PgPool,
    enrolment_id: Uuid,
    factor_id: Uuid,
    passkey_origin: super::PasskeyOrigin,
) -> anyhow::Result<()> {
    let updated = sqlx::query(r#"
        UPDATE passkey_enrolment_state
        SET state='confirmed', confirmed_at=now(), factor_id=$2, passkey_origin=$3::passkey_origin
        WHERE id=$1 AND state='requested' AND expires_at > now()
    "#).bind(enrolment_id).bind(factor_id).bind(format!("{passkey_origin:?}").to_lowercase())
       .execute(pool).await?;
    if updated.rows_affected() == 0 {
        anyhow::bail!("enrolment_not_pending_or_expired");
    }
    Ok(())
}

pub async fn run_abandonment_job(pool: &PgPool) -> anyhow::Result<usize> {
    let updated = sqlx::query(r#"
        UPDATE passkey_enrolment_state SET state='abandoned', abandoned_at=now()
        WHERE state='requested' AND expires_at < now()
    "#).execute(pool).await?;
    Ok(updated.rows_affected() as usize)
}
```

### 3.5 — Downgrade gate

```rust
// services/auth/src/passkey/downgrade_gate.rs
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DowngradeGate {
    cache: Arc<RwLock<HashMap<Uuid, (bool, Instant)>>>,
}

const CACHE_TTL_SECONDS: u64 = 60;

impl DowngradeGate {
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Returns true if the subject has ≥1 active passkey AND no opt-out flag.
    pub async fn is_passkey_required(&self, subject_id: Uuid, db: &PgPool) -> bool {
        if let Some((req, t)) = self.cache.read().await.get(&subject_id) {
            if t.elapsed() < Duration::from_secs(CACHE_TTL_SECONDS) {
                return *req;
            }
        }
        let row: Option<(i64, bool)> = sqlx::query_as(r#"
            SELECT COUNT(*) AS pk_count,
                   COALESCE(BOOL_OR(passkey_downgrade_optout), false) AS has_optout
            FROM mfa_factors
            WHERE subject_id = $1
              AND status = 'active'
              AND factor_kind IN ('webauthn_platform','webauthn_cross_platform')
        "#).bind(subject_id).fetch_optional(db).await.ok().flatten();

        let required = match row {
            Some((count, has_optout)) => count > 0 && !has_optout,
            None => false,
        };
        self.cache.write().await.insert(subject_id, (required, Instant::now()));
        required
    }

    pub async fn invalidate(&self, subject_id: Uuid) {
        self.cache.write().await.remove(&subject_id);
    }
}
```

### 3.6 — Enrol begin handler

```rust
// services/auth/src/handlers/passkey.rs (excerpt)
use axum::{Json, extract::State, http::StatusCode};
use crate::passkey::{enrolment, audit};

#[derive(Serialize)]
pub struct EnrolBeginResponse {
    pub enrolment_id: uuid::Uuid,
    pub public_key_creation_options: serde_json::Value,
    pub recovery_warning: bool,
    pub recommended_action: Option<String>,
}

pub async fn enrol_begin(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<EnrolBeginResponse>), ApiError> {
    // (1) Check passkey count vs 5-cap
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mfa_factors
         WHERE subject_id=$1 AND status='active' AND factor_kind IN ('webauthn_platform','webauthn_cross_platform')"
    ).bind(claims.subject_id()).fetch_one(&state.db).await?;
    if active_count >= 5 {
        return Err(ApiError::PasskeyLimitExceeded);
    }

    // (2) Generate WebAuthn creation challenge
    let challenge = state.webauthn.start_passkey_registration(claims.subject_id(), claims.tenant_id()).await?;

    // (3) Persist enrolment state
    let enrolment_id = enrolment::begin_enrolment(&state.db, claims.tenant_id(), claims.subject_id(), challenge.challenge_bytes()).await?;

    // (4) Recovery-code warning
    let has_recovery: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM mfa_recovery_codes WHERE subject_id=$1 AND consumed=false)"
    ).bind(claims.subject_id()).fetch_one(&state.db).await?;

    // (5) Audit
    audit::emit_passkey_enrolment_requested(claims.tenant_id(), claims.subject_id()).await;

    Ok((StatusCode::CREATED, Json(EnrolBeginResponse {
        enrolment_id,
        public_key_creation_options: challenge.public_key_credential_creation_options(),
        recovery_warning: !has_recovery,
        recommended_action: if !has_recovery { Some("POST /v1/auth/mfa/recovery-codes/regen".into()) } else { None },
    })))
}
```

### 3.7 — Login finish handler

```rust
// services/auth/src/handlers/passkey.rs (continued)
#[derive(Deserialize)]
pub struct LoginFinishRequest {
    pub assertion_response: serde_json::Value,
}

pub async fn login_finish(
    State(state): State<AppState>,
    Json(req): Json<LoginFinishRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // (1) Extract credential_id + userHandle (subject_id) from assertion
    let cred_id = extract_credential_id(&req.assertion_response)?;
    let user_handle = extract_user_handle(&req.assertion_response)?;   // subject_id

    // (2) Lookup factor
    let factor = state.repo.find_factor_by_credential_id(&cred_id).await?
        .ok_or(ApiError::PasskeyNotFound)?;
    if factor.subject_id != user_handle {
        return Err(ApiError::PasskeyUserHandleMismatch);
    }

    // (3) Verify signature via FR-AUTH-102 webauthn module + counter-monotonicity
    let result = state.webauthn.verify_assertion(&factor, &req.assertion_response).await;
    match result {
        Ok(new_counter) => {
            state.repo.update_factor_counter_and_last_used(factor.id, new_counter).await?;
            audit::emit_passkey_login_succeeded(factor.tenant_id, factor.subject_id, factor.id, factor.passkey_origin).await;
            let token = state.jwt_issuer.issue_for_subject(factor.subject_id, factor.tenant_id).await?;
            Ok(Json(LoginResponse { access_token: token }))
        }
        Err(crate::mfa::webauthn::VerifyError::ClonedAuthenticator) => {
            // Per FR-AUTH-102 §1 #10 — revoke factor + sev-1 audit
            state.repo.revoke_factor(factor.id, "cloned_authenticator_detected").await?;
            audit::emit_passkey_login_failed(factor.tenant_id, factor.subject_id, factor.id, "cloned_authenticator").await;
            Err(ApiError::ClonedAuthenticatorDetected)
        }
        Err(e) => {
            audit::emit_passkey_login_failed(factor.tenant_id, factor.subject_id, factor.id, &format!("{e:?}")).await;
            Err(ApiError::PasskeyLoginFailed)
        }
    }
}
```

---

## §4 — Acceptance criteria

1. **PasskeyOrigin enum closed at 3** — platform_synced, platform_local, cross_platform.
2. **Enrolment requires resident-key + UV-required** — challenge config asserted.
3. **Enrol begin → 201 + enrolment_id + challenge** — happy path.
4. **Enrol finish within 24h confirms** — state → confirmed.
5. **Enrol abandonment after 24h** — hourly job sets state=abandoned + emits sev-3 audit.
6. **Enrol exceeding 5-cap rejected** → 409 passkey_limit_exceeded.
7. **Recovery warning emitted when zero codes** — response carries recovery_warning=true.
8. **Login begin returns options with allowCredentials=[] (discoverable mode)**.
9. **Login finish with valid signature → 200** + JWT issued + `auth.passkey_login_succeeded` row.
10. **Login finish with cloned authenticator** → factor revoked + sev-1 audit per FR-AUTH-102.
11. **Login finish with bad signature** → 401 + `auth.passkey_login_failed`.
12. **Password login blocked when passkey enrolled** → 401 passkey_required + `auth.passkey_downgrade_blocked` (sev-2).
13. **Per-subject opt-out flag bypasses downgrade-resistance** → password login OK.
14. **Per-subject opt-out requires root-admin + reason** → 403 if non-root-admin.
15. **Opt-out emits sev-2 audit row**.
16. **Conditional mediation requires UV=required** — handler rejects without UV.
17. **Conditional mediation autofill_used emits BRAIN row**.
18. **Per-tenant passkey_required_for_roles enforced** — subject with role in policy + no passkey → 401 passkey_enrolment_required.
19. **Founder role always requires passkey** (DEC-128).
20. **Removal requires X-MFA-Challenge-Token < 5min old** → 401 recent_mfa_required if absent.
21. **Removal soft-deletes + emits `auth.passkey_removed`**.
22. **AAGUID origin detection** — Apple/Google/Microsoft AAGUIDs map to platform_synced; YubiKey → cross_platform; unknown → cross_platform.
23. **GET /factors lists passkeys** — public_key never exposed.
24. **Counter `auth_passkey_downgrade_blocked_total` sev-2 alarm at > 5/h per subject**.
25. **Counter `auth_passkey_autofill_used_total` increments** on conditional mediation success.
26. **append-only lifecycle_log** — UPDATE/DELETE blocked.
27. **Perf p95 < 200ms** — enrol + login handlers.
28. **OTel span emitted per handler** — outcome attribute.

---

## §5 — Verification

```rust
// services/auth/tests/passkey_downgrade_block_test.rs
#[tokio::test]
async fn password_login_blocked_after_passkey_enrol(ctx: TestCtx) {
    let subject = ctx.create_test_subject_with_password().await;
    // Initially: password login succeeds
    let r1 = ctx.password_login(&subject.email, "test_password").await;
    assert!(r1.is_ok());
    // Enrol passkey
    ctx.enrol_passkey_for(subject.id).await;
    // Now password login must fail
    let r2 = ctx.password_login(&subject.email, "test_password").await;
    assert!(matches!(r2, Err(e) if format!("{e:?}").contains("passkey_required")));
    let rows = ctx.brain_audit_rows("auth.passkey_downgrade_blocked").await;
    assert!(!rows.is_empty());
    assert_eq!(rows[0]["severity"], "sev-2");
}

#[tokio::test]
async fn opt_out_allows_password_login(ctx: TestCtx) {
    let subject = ctx.create_subject_with_passkey_enrolled().await;
    ctx.post_as_root_admin("/v1/auth/passkey/downgrade-optout",
        json!({"subject_id": subject.id, "enabled": true, "reason": "legacy CLI integration"})).await.unwrap();
    let r = ctx.password_login(&subject.email, "test_password").await;
    assert!(r.is_ok());
}
```

```rust
// services/auth/tests/passkey_max_5_per_subject_test.rs
#[tokio::test]
async fn sixth_enrolment_rejected(ctx: TestCtx) {
    let subject = ctx.create_test_subject().await;
    for _ in 0..5 { ctx.enrol_passkey_for(subject.id).await; }
    let err = ctx.try_enrol_passkey_for(subject.id).await.unwrap_err();
    assert!(format!("{err:?}").contains("passkey_limit_exceeded"));
}
```

```rust
// services/auth/tests/passkey_removal_requires_reauth_test.rs
#[tokio::test]
async fn remove_without_recent_mfa_rejected(ctx: TestCtx) {
    let subject = ctx.create_subject_with_passkey().await;
    let token = ctx.issue_token_for(subject.id).await;
    // Try removal without X-MFA-Challenge-Token header
    let err = ctx.delete_passkey(&subject.passkey_id, &token, None).await.unwrap_err();
    assert!(format!("{err:?}").contains("recent_mfa_required"));
}

#[tokio::test]
async fn remove_with_recent_mfa_succeeds(ctx: TestCtx) {
    let subject = ctx.create_subject_with_passkey().await;
    let token = ctx.issue_token_for(subject.id).await;
    let mfa_token = ctx.verify_mfa_get_challenge_token(subject.id).await;
    let r = ctx.delete_passkey(&subject.passkey_id, &token, Some(&mfa_token)).await;
    assert!(r.is_ok());
    let rows = ctx.brain_audit_rows("auth.passkey_removed").await;
    assert!(!rows.is_empty());
}
```

```rust
// services/auth/tests/passkey_abandonment_24h_test.rs
#[tokio::test]
async fn enrolment_abandoned_after_24h(ctx: TestCtx) {
    let subject = ctx.create_test_subject().await;
    let enrolment_id = ctx.begin_enrolment_for(subject.id).await;
    ctx.advance_clock_hours(25).await;
    let abandoned = cyberos_auth::passkey::enrolment::run_abandonment_job(&ctx.pool).await.unwrap();
    assert_eq!(abandoned, 1);
    let state = ctx.fetch_enrolment_state(enrolment_id).await;
    assert_eq!(state.state, "abandoned");
    let rows = ctx.brain_audit_rows("auth.passkey_enrolment_abandoned").await;
    assert!(!rows.is_empty());
}
```

```rust
// services/auth/tests/passkey_origin_enum_closed_test.rs
#[test]
fn origin_enum_exactly_3() {
    use cyberos_auth::passkey::PasskeyOrigin;
    assert_eq!(PasskeyOrigin::ALL.len(), 3);
}

#[test]
fn unknown_aaguid_defaults_cross_platform() {
    let unknown = uuid::Uuid::new_v4();
    let origin = cyberos_auth::passkey::origin::detect(unknown);
    assert_eq!(origin, cyberos_auth::passkey::PasskeyOrigin::CrossPlatform);
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 8 BRAIN row builders follow canonical pattern; webauthn-rs handles cryptographic plumbing via FR-AUTH-102.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-102** — TOTP + WebAuthn MFA (passkeys ARE WebAuthn discoverable credentials; reuses factor table + verifier).

**Downstream:** none at slice 1.

**Cross-module:**
- **FR-AUTH-002** — subject (PRIMARY KEY for factor + enrolment rows).
- **FR-AUTH-004** — JWT issuer (downgrade-gate hook).
- **FR-AUTH-101** — RBAC (founder = passkey-required; root-admin for opt-out + force-remove).
- **FR-AI-003** — BRAIN audit bridge.
- **FR-BRAIN-111** — PII scrubbing (display_name).
- **FR-AI-005** — per-tenant policy YAML (passkey_required_for_roles).
- **FR-OBS-007** — sev-2 alarm rules.

---

## §8 — Example payloads

### 8.1 — POST /v1/auth/passkey/enrol/begin response

```json
{
  "enrolment_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "public_key_creation_options": {
    "rp": { "id": "cyberos.world", "name": "CyberOS" },
    "user": { "id": "<base64url-subject-id>", "name": "alice@acme.example", "displayName": "Alice Vu" },
    "challenge": "<base64url-32-bytes>",
    "pubKeyCredParams": [{"type":"public-key","alg":-7},{"type":"public-key","alg":-257}],
    "authenticatorSelection": {
      "residentKey": "required",
      "userVerification": "required",
      "authenticatorAttachment": null
    },
    "attestation": "none",
    "timeout": 60000
  },
  "recovery_warning": true,
  "recommended_action": "POST /v1/auth/mfa/recovery-codes/regen"
}
```

### 8.2 — auth.passkey_enrolment_confirmed BRAIN row

```json
{
  "kind": "auth.passkey_enrolment_confirmed",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "factor_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "passkey_origin": "platform_synced",
  "ts_ns": 1747920731000000000
}
```

### 8.3 — auth.passkey_downgrade_blocked BRAIN row (sev-2)

```json
{
  "kind": "auth.passkey_downgrade_blocked",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "source_ip_hash16": "fed0987654321abc",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — POST downgrade-optout

```json
{
  "subject_id": "9b1deb4d-...",
  "enabled": true,
  "reason": "Legacy CLI integration requires password auth; exception approved by CSEC ticket SEC-2026-Q3-042"
}
```

### 8.5 — auth.passkey_autofill_used BRAIN row

```json
{
  "kind": "auth.passkey_autofill_used",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "passkey_origin": "platform_synced",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Multi-tenant passkey portability** — same physical key across tenants treated as distinct credentials at slice 1; cross-tenant binding at slice 3.
- **Passkey backup/recovery via cross-platform key** — recovery codes are the primary fallback; cross-platform-key-as-recovery is FR-AUTH-2xx UX.
- **Passkey enrolment UI on mobile native app** — slice 2 (web flow only at slice 1).
- **Sync detection from authenticator metadata** — extension protocols not standardised yet; AAGUID lookup is the slice 1 proxy.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Enrolment timeout > 24h | abandonment job | state=abandoned + audit | Re-initiate |
| Enrolment exceeds 5-cap | handler check | 409 passkey_limit_exceeded | Remove old passkey first |
| Discoverable credential not supported | challenge fail | 400 authenticator_unsupported | Use different authenticator |
| Counter monotonicity violation (cloned key) | FR-AUTH-102 verifier | Factor revoked + sev-1 audit | Re-enrol |
| Password login on passkey-enrolled subject | downgrade_gate | 401 passkey_required + sev-2 | Use passkey |
| Opt-out by non-root-admin | role check | 403 | Designed |
| Opt-out without reason | validation | 400 | Designed |
| Removal without recent MFA | header check | 401 recent_mfa_required | Verify MFA first |
| AAGUID unknown | conservative default | origin=cross_platform | Designed |
| Conditional mediation without UV=required | handler | 400 | Designed |
| Per-tenant policy requires passkey + subject has none | JWT issuer | 401 passkey_enrolment_required | Enrol |
| Founder bypass attempt | hard-coded | 401 | Designed |
| Enrol/finish after 24h expiry | confirm_enrolment check | "enrolment_not_pending_or_expired" | Re-initiate |
| Concurrent enrolment for same subject | DB row insert | OK (multiple pending) | None |
| Append-only log UPDATE from app | SQL grant | permission denied | Designed |
| Public key exposed in API | handler omits | Never returned | Designed |
| RLS bypass | USING | 0 rows | Designed |
| Cross-tenant passkey | credential_id UNIQUE per subject | 409 if reused | Designed |
| user_handle mismatch in login | handler check | 401 | Designed |
| BRAIN audit fail mid-tx | rollback | 500 | brain_writer health |
| Cache stale during opt-out toggle | 60s TTL + invalidate | Brief delay | Designed |
| > 5/h downgrade-blocked sustained | OBS rule | sev-2 | Investigate phishing |
| Lost all devices + no recovery codes | None (designed risk) | Permanent lockout | Recovery codes mandatory in enrol warning |
| Browser doesn't support conditional mediation | feature detection at frontend | Fallback to explicit "Use passkey" button | None — designed |
| Subject deleted while passkeys exist | FK RESTRICT | DELETE auth.subjects fails | Soft-delete subject |
| Concurrent removal + enrolment | row lock | Serial | Designed |
| OTel span attribute missing | otel_test | CI fails | Fix |
| Enrolment challenge replay | challenge UNIQUE per row | Already-confirmed challenge → no-op | Designed |
| Per-subject `passkey_downgrade_optout` flipped without audit | trigger | sev-2 row mandatory | Designed |
| Counter alarms not firing | OBS rule | CI fails | Coordinate with FR-OBS-007 |

---

## §11 — Implementation notes

- **Passkey = WebAuthn discoverable credential** — same factor table as FR-AUTH-102.
- **Closed 3-value passkey_origin enum** — adoption analytics driver.
- **Autofill conditional mediation** — primary discovery surface for new users.
- **Downgrade-resistance** — once enrolled, password login blocked; opt-out is exception path.
- **Max 5 per subject** — typical user count.
- **Removal requires fresh MFA (< 5min)** — session-hijack defense.
- **AAGUID lookup for origin detection** — Apple/Google/Microsoft + YubiKey known IDs.
- **Recovery warning at enrol** — passkey-only + no recovery = lockout risk.
- **Per-tenant `passkey_required_for_roles` YAML** — flexibility; founder hard-coded.
- **opt-out flag + root-admin + audit** — explicit exception path.
- **Append-only lifecycle log** — forensic.
- **8 BRAIN audit kinds** — selective operator queries.
- **Sev-2 alarm on > 5/h downgrade-blocked** — phishing signal.
- **Conditional mediation requires UV=required** — W3C spec compliance.
- **userHandle = subject_id** — resident-key login resolves subject without username input.
- **Public key never exposed via API** — defense in depth.
- **Cache 60s on downgrade gate** — matches FR-AUTH-109 pattern.
- **Hourly abandonment job** — clean lifecycle, no orphan rows.
- **No SLO** — passkey is auth, not session.
- **AAGUID unknown defaults cross_platform** — conservative.
- **PII scrub display_name** — chain holds scrubbed.

---

*End of FR-AUTH-105.*
