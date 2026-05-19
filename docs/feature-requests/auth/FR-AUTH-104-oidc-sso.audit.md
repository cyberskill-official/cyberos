---
fr_id: FR-AUTH-104
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-AUTH-104 ships standards-compliant OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + PKCE Authorisation Code flow + per-tenant IdP config + JIT subject provisioning + claim → role mapping. Scope: 26 §1 normative clauses covering 3 tables (idp_configs + login_history append-only + subject_link), discovery with 1h TTL + PKCE-support validation, JWKS with 24h cache + kid rotation overlap, full PKCE auth code flow with state + nonce + 10-min TTL, id_token signature + iss + aud + exp + nbf verification with 60s skew, JIT subject provisioning calling FR-AUTH-002 internal helper, per-tenant claim_mapping_yaml validated against FR-AUTH-101 closed role enum, max 3 IdP configs per tenant, 6 memory audit kinds with sev-2 on signature failures, RLS isolation, KMS-encrypted client_secret, locked redirect_uri, implicit + hybrid flow forbidden, per-(idp_id, sub) uniqueness with cross-IdP linking deferred to slice 3, append-only login_history at SQL grant. 19 rationale paragraphs. §3 contains: 3 migrations, discovery + JWKS modules, PKCE flow generator with code_challenge + state + nonce, id_token verifier with constant-time validation, claim mapper validating against AUTH-101 role enum. 27 ACs. 30 failure-mode rows. 21 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Implicit flow allowed
First-pass supported all OIDC flows. Resolved: §1 #21 + DEC-401 + discovery PKCE validation + disallowed_tools.

### ISS-002 — JWKS not rotated
First-pass cached JWKS forever. Resolved: §1 #6 + DEC-402 + 24h cache + unknown-kid refetch + `auth.oidc_jwks_rotated` audit row.

### ISS-003 — PKCE optional
First-pass had PKCE as best-effort. Resolved: §1 #6 + DEC-401 + S256 hard-coded + discovery validates `S256` in `code_challenge_methods_supported`.

### ISS-004 — State + nonce missing
Resolved: §1 #11 + DEC-408 + 10-min state TTL + nonce in id_token; AC #6-#8 + AC #13.

### ISS-005 — id_token signature unverified
First-pass trusted IdP-returned token. Resolved: §1 #7 + DEC-411 + jsonwebtoken verification against JWKS + iss/aud/exp/nbf/nonce checks + 60s skew.

### ISS-006 — JIT provisioning unbounded role assignment
First-pass let claim mapping reference any role. Resolved: §1 #10 + `parse_config` validates against FR-AUTH-101 closed enum; unknown role → 400 at config save.

### ISS-007 — Append-only history not enforced
Resolved: §1 #2 + DEC-407 + `REVOKE UPDATE, DELETE FROM cyberos_app`; AC #21.

### ISS-008 — Cross-IdP same-sub collision
First-pass had global sub uniqueness. Resolved: §1 #24 + DEC-410 + per-(idp_id, sub) uniqueness; cross-IdP linking deferred.

### ISS-009 — client_secret in API responses
First-pass returned full secret. Resolved: §1 #25 + handler omits client_secret from all responses; `auth.oidc_idp_config_changed` memory row excludes secret value.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (RFC 8414 discovery × RFC 7517 JWKS rotation × PKCE S256 × state + nonce × id_token signature × JIT provisioning × claim mapping × per-tenant IdP config × KMS-encrypted secret × locked redirect_uri × append-only history × 6 memory audit kinds × sev-2 alarm), not by line targets.

---

*End of FR-AUTH-104 audit.*
