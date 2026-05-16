---
fr_id: FR-AUTH-102
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-AUTH-102 ships TOTP (RFC 6238) + WebAuthn Level 3 MFA with closed 3-factor enum + lifecycle FSM + lockout. Scope: 27 §1 normative clauses covering closed `factor_kind` enum (totp, webauthn_platform, webauthn_cross_platform), RFC 6238 TOTP (HMAC-SHA1, 30s, 6-digit, ±1 step skew, 160-bit secret KMS-encrypted), WebAuthn Level 3 (RP=cyberos.world, UV=required, resident_key=preferred, attestation=none, counter-monotonicity check per W3C §10.4), challenge lifecycle FSM (pending → consumed|expired|failed; 5min TTL; reuse rejected sev-2), 10 recovery codes (8-char base32 ALPHABET excluding 0/1/O/L, bcrypt cost 12, single-use, regen invalidates ALL), lockout (5 fails in 15min → 30min lock + sev-1; root-admin early-unlock; counter resets on success), per-tenant MFA policy (`mfa_required_roles` from tenant_policy YAML; founder always requires), append-only mfa_factor_history + mfa_challenge_log + mfa_lockout_state via SQL grant with privileged writer roles, 8 BRAIN audit kinds with PII scrubbing, TOTP confirm-on-enrol catches misconfiguration, last-factor removal invalidates recovery codes (security invariant), GET /factors never exposes secrets/public keys. 23 rationale paragraphs. §3 contains: 5 migrations (mfa_factors with closed enum + RLS + WebAuthn fields, mfa_factor_history append-only, mfa_challenge_log with challenge_writer role split, mfa_recovery_codes with column-level UPDATE grant on consumed only, mfa_lockout_state with mfa_lockout_writer role split), TOTP module with RFC 6238 test vectors, lockout module with sliding-window state machine, recovery module with bcrypt + zeroize, verify handler with full flow. 30 ACs. 33 failure-mode rows. 23 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — TOTP step skew unbounded
First-pass accepted any drift. Resolved: §1 #8 + DEC-489 + ±1 step (60s window); AC #3.

### ISS-002 — WebAuthn counter unchecked (cloned authenticator attack)
First-pass didn't validate monotonicity. Resolved: §1 #10 + DEC-490 + W3C §10.4 + sev-1 audit + factor revocation; AC #8.

### ISS-003 — Challenge reuse silently accepted
First-pass had no replay defense. Resolved: §1 #11 + DEC-484 + FSM with `consumed` terminal state + sev-2 audit on reuse; AC #10.

### ISS-004 — Recovery codes multi-use
First-pass let same code authenticate repeatedly. Resolved: §1 #13 + DEC-485 + single-use flag + bcrypt cost 12 + regen-invalidates-all per DEC-492; AC #12 + #13.

### ISS-005 — No lockout policy
First-pass had unlimited verification attempts. Resolved: §1 #14 + DEC-487 + 5/15 → 30min + sev-1 audit + root-admin unlock; AC #14-#17.

### ISS-006 — TOTP secret exposed post-enrolment
First-pass kept the secret in API responses. Resolved: §1 #9 + DEC-482 + KMS-encrypt at rest + never-expose-after-enrol; AC #4 + #5.

### ISS-007 — WebAuthn UV=preferred allows passwordless bypass
First-pass had `preferred`. Resolved: §1 #10 + DEC-483 + `required` enforced; AC #6.

### ISS-008 — Per-tenant MFA policy missing
First-pass had global MFA-on/off. Resolved: §1 #15 + DEC-491 + per-tenant `mfa_required_roles` YAML + founder-always; AC #18 + #19.

### ISS-009 — Append-only state not enforced at SQL grant
First-pass relied on handler discipline. Resolved: §1 #3-#6 + DEC-488 + REVOKE + privileged writer roles for state transitions; AC #20-#22.

### ISS-010 — TOTP misconfiguration silently rolls forward
First-pass had no confirm-on-enrol. Resolved: §1 #27 + confirmation_code required; AC #28.

### ISS-011 — Last-factor removal left recovery codes valid (security gap)
Resolved: §1 #26 + invariant invalidates recovery on last-factor removal; AC #29.

## §3 — Resolution

All 11 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (TOTP RFC 6238 × WebAuthn L3 × counter monotonicity × challenge FSM with TTL × 10 recovery codes single-use bcrypt-hashed × 5/15→30min lockout sev-1 × per-tenant policy × founder-always × confirm-on-enrol × last-factor invariant × 8 BRAIN audit kinds × append-only via SQL grant with privileged writer roles × KMS-encrypted TOTP secrets × webauthn-rs crate × per-RFC 6238 test vectors), not by line targets.

---

*End of FR-AUTH-102 audit.*
