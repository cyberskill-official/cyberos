---
fr_id: FR-PORTAL-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands per-Engagement external IdP (SAML 2.0 + OIDC) with SCIM 2.0 JIT provisioning on top of FR-AUTH-103 (SAML primitives) + FR-AUTH-104 (OIDC primitives). Final form: 1,030 lines, 27 §1 normative clauses (covering 4 migrations, 2 protocol stacks, SCIM CRUD endpoints, claim → role mapping with many-to-one semantics, signed-attribute trust chain defending against attribute injection, per-Engagement isolation via RLS, required-enforcement SSO-only mode, 8h JWT TTL + re-auth trigger, email-domain discovery hint, SCIM token rotation with 60s overlap, 7 + 1 memory audit kinds), 20 acceptance criteria, 10 verification tests, 22 failure-mode rows, 26 implementation notes. Net-new `cyberos/services/portal/` crate.

The audit caught 7 issues across the security-critical signed-attribute path, SCIM token storage hygiene, RLS scope precision, role-resolution determinism, and a missing migration ref. All resolved before 10/10.

## §2 — Findings (all resolved)

### ISS-001 — `cyberos_app` UPDATE grant on `portal_idp_configs` initially over-broad

First-draft `REVOKE UPDATE, DELETE FROM cyberos_app` was followed by no GRANT, but the rotation handler needs to set `rotated_at` + `status`. Either rotation fails OR we leak full UPDATE to cyberos_app — both wrong. Resolved: §3.1 added `GRANT UPDATE (rotated_at, status) ON portal_idp_configs TO cyberos_app` (column-level grant scoped to the rotation columns only). Same pattern applied to `portal_scim_tokens`.

### ISS-002 — SAML XML signature wrap (XSW) attack not explicitly defended

§1 #8 mandated signature verification but didn't explicitly note XSW defense (where attacker inserts unsigned malicious elements while keeping the signed envelope intact). §10 row added covering XSW detection via canonical-form check + signature reference validation; §11.1 + §11.9 specify the `samael` + `xmlsec-rs` libraries that handle this correctly out of the box. The signed_attr.rs walk (§1 #15) also defends — only signed enclosing scopes count.

### ISS-003 — Role resolver tie-break under-specified

§1 #12 said "highest-privilege match wins" but the original draft didn't enumerate the ordinal. Two reviewers would interpret "highest" differently. Resolved: §11.11 added explicit `client_viewer < client_editor < client_admin` ordinal; tie-break is deterministic-max.

### ISS-004 — `last_sso_at` column referenced but not in any migration this FR controls

§1 #17 + §11.15 read `subjects.last_sso_at`; this column is in the FR-AUTH-002 `subjects` table not owned by PORTAL. Without a migration reference, the implementer would hit "column does not exist" errors. Resolved: §11.8 calls out the required AUTH-side migration `services/auth/migrations/0XXX_subjects_last_sso_at.sql`; build_envelope `modified_files` already lists `services/auth/src/admin/subjects.rs` so the cross-crate change is acknowledged. The migration filename is intentionally a placeholder pending the exact number assignment when this FR builds.

### ISS-005 — SCIM token storage SHA-256 vs HMAC choice unclear

§3.1 stored `token_sha256 CHAR(64)` but DEC-867 didn't specify whether SHA-256 alone (vulnerable to rainbow-table if salt exposed) or HMAC-SHA256 (with server-side secret). For a 32-byte random opaque token, plain SHA-256 is fine (entropy too high for rainbow table) — clarified in §11.7 that tokens are 32 random bytes (256 bits) base64url-encoded; SHA-256 storage is sufficient at that entropy level. No code change; clarification only.

### ISS-006 — Per-Engagement RLS for `portal_idp_groups_map` uses subquery on parent table

§3.1 `portal_idp_groups_map` RLS USING clause does `idp_config_id IN (SELECT id FROM portal_idp_configs WHERE tenant_id = ...)` — a subquery on the parent table. This works but is slower than carrying `tenant_id` denormalised on the child. Considered: add `tenant_id` to `portal_idp_groups_map`. Decision: keep subquery; group-map reads are admin-only (low volume), denormalisation is over-engineering. Clarified in §11 (no new note needed — pattern is reviewable in §3.1 SQL). Marked resolved as documentation decision.

### ISS-007 — Audit `scim_operation` enum missing `token_rotated`

§3.1 originally had 6 SCIM operations (user/group × create/update/delete); §1 #19 + DEC-884 reference `portal.scim_token_rotation` memory row + a `token_rotated` operation. Without the `token_rotated` enum member, the rotation handler's INSERT into the audit log would fail. Resolved: §3.1 CHECK constraint extended to include `'token_rotated'` (7 enum members).

## §3 — Resolution

All 7 mechanical concerns addressed. Security-critical paths (XSW defense, signed-attribute trust, role resolution determinism, RLS scope) are now precise; grant lineage clean; cross-crate dependency on `subjects.last_sso_at` flagged for the implementer.

The 1,030-line length sits just above the 1,000-line soft cap; justified by net-new crate + 2 protocol stacks (SAML + OIDC) + SCIM CRUD + claim/role mapping + per-Engagement isolation + 22 failure modes spanning both protocols. Density comparable to FR-TEN-003 (1,056) and FR-TEN-101 (1,160).

**Score = 10/10.**

---

*End of FR-PORTAL-003 audit.*
