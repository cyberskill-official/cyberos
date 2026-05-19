---
fr_id: FR-AUTH-103
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

FR-AUTH-103 ships SAML 2.0 SP-initiated SSO with per-tenant IdP config + XML signature verification + assertion validation + JIT provisioning + attribute mapping + replay defense. Scope: 27 §1 normative clauses covering 4 tables (idp_configs + login_history append-only + authn_request_log with TTL + subject_link), SP-initiated flow only (IdP-initiated rejected per DEC-520), HTTP POST + Redirect bindings, WantAssertionsSigned + AuthnRequestsSigned both required (XSW defense), exc-c14n + enveloped-signature only (XSW transform whitelist), RSA-SHA256 minimum (SHA-1 rejected per NIST), closed NameIDFormat (email + persistent), InResponseTo replay defense with 10-min TTL + consumed flag, ±60s clock skew, Audience + Recipient + InResponseTo + transform whitelist defense-in-depth, JIT provisioning via FR-AUTH-002 helper, per-tenant attribute_mapping_yaml validated against FR-AUTH-101 closed Role enum, max 2 active IdP configs per tenant, KMS-encrypted SP signing key, 24h metadata cache with cert rotation overlap, 7 memory audit kinds with PII scrubbing + sev-2 on signature-invalid + replay-attempt, SP metadata download endpoint for IdP operators, RequestedAuthnContext PasswordProtectedTransport baseline. 22 rationale paragraphs. §3 contains: 4 migrations (idp_configs + login_history append-only + authn_request_log with consumed-flag UPDATE grant + subject_link), AuthnRequest XML builder, response verifier with full 8-step validation flow (samael-crate-based), ACS callback handler with replay check + signature verify + JIT provisioning + AUTH JWT issuance. 32 ACs. 33 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — IdP-initiated unsolicited responses accepted
First-pass allowed any POST to ACS. Resolved: §1 #6 + DEC-520 + InResponseTo required + sev-2 audit on unsolicited; AC #1.

### ISS-002 — Assertion signature not enforced (Azure AD default = response-only signing)
Resolved: §1 #8 + DEC-522 + WantAssertionsSigned=true + dedicated `assertion_signature_required` error; AC #3.

### ISS-003 — SHA-1 signature algorithm accepted
First-pass had no algorithm whitelist. Resolved: §1 #13 + DEC-532 + closed ALLOWED_SIG_ALGS list (RSA-SHA256 + ECDSA-SHA256/384); AC #5.

### ISS-004 — Open transform set enables XSW attack
Resolved: §1 #12 + DEC-531 + exc-c14n + enveloped-signature ONLY whitelist; AC #8.

### ISS-005 — InResponseTo replay accepted
First-pass had no replay defense. Resolved: §1 #10 + DEC-525 + authn_request_log with 10-min TTL + consumed flag + sev-2 audit; AC #9 + #10 + #11.

### ISS-006 — NameIDFormat unbounded
First-pass accepted any format. Resolved: §1 #9 + DEC-524 + closed allowlist (emailAddress + persistent); AC #17-#19.

### ISS-007 — Audience + Recipient checks missing
First-pass relied on signature alone. Resolved: §1 #11 + DEC-526 + DEC-536 + multi-layer validation; AC #12 + #13.

### ISS-008 — Clock skew unbounded
First-pass had no leeway. Resolved: §1 #11 + DEC-526 + ±60s tolerance; AC #14-#16.

### ISS-009 — Per-tenant SP signing key not KMS-encrypted
Resolved: §1 #1 + DEC-527 + sp_signing_key_kms_blob BYTEA + KMS-decrypt on sign; AC #25.

### ISS-010 — Hand-rolled XML signature implementation
First-pass had custom XML parsing. Resolved: §1 #23 + samael crate (battle-tested) + disallowed_tools forbids hand-rolling.

### ISS-011 — Unknown role in attribute_mapping_yaml saved silently
Resolved: §1 #16 + validation at config save + 400 unknown_role; AC #23.

## §3 — Resolution

All 11 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (SP-initiated × HTTP POST/Redirect × WantAssertionsSigned+AuthnRequestsSigned × exc-c14n transform whitelist (XSW defense) × RSA-SHA256 minimum × InResponseTo replay defense × closed NameIDFormat × Audience + Recipient + InResponseTo + transform multi-layer × per-tenant attribute mapping validated against AUTH-101 Roles × 24h metadata cache + cert rotation × KMS-encrypted SP key × 7 memory audit kinds × samael crate × SP metadata download), not by line targets.

---

*End of FR-AUTH-103 audit.*
