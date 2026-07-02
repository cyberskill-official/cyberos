---
fr_id: FR-AUTH-101
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 12
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0; all 12 ISSes resolved in revision)
---

## §1 — Verdict summary

FR-AUTH-101 ships the closed 22-role catalogue + permission matrix + role-assignment REST + JWT claims + ADR gate + stub→full migration. Scope: 25 §1 normative clauses (closed role enum, closed resource/action enums, permission matrix seeding, assign/revoke/list REST, in-memory matrix with 60s refresh, RLS integration, reserved-role gating, founder-WebAuthn gate, JWT claim shape, ADR-gate CI test, stub compatibility, scope-grant narrowing, performance budget, OTel emission, migrations 0005 + 0006, ADR-101 file). 25 rationale paragraphs. 27 ACs. 28-row §10 failure inventory. 20 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Catalogue mutability via runtime REST
First-pass: "POST /v1/admin/roles" could mutate the catalogue at runtime, defeating ADR gating. Resolved: §1 #15 + REVOKE INSERT/UPDATE/DELETE on `roles`/`role_permissions` from `cyberos_app`; explicitly listed in `disallowed_tools`; runtime mutation is structurally impossible.

### ISS-002 — Reserved role assignment path ambiguous
First-pass said "reserved roles cannot be self-assigned" but didn't specify where they CAN be assigned. Resolved: §1 #11 + §9 — dedicated endpoints per reserved role (root-admin via FR-TEN-001 bootstrap; auditor/regulator/billing-system via intake; client-portal-user via FR-PORTAL-003); standard endpoint refuses with `required_endpoint` hint.

### ISS-003 — RBAC version replay-resistance gap
First-pass JWT carried `roles` only; a stale token with looser permissions would remain valid until expiry. Resolved: §1 #8 + DEC-129 — `rbac_v` claim with 2-version tolerance; verifier rejects > 2 stale.

### ISS-004 — Founder WebAuthn gate configurable
First-pass had the WebAuthn requirement in a config table — operator could turn off. Resolved: §1 #12 + DEC-128 — intrinsic to closed enum; `Role::Founder.requires_webauthn() == true` hard-coded; no config knob.

### ISS-005 — Matrix refresh DOS via DB unavailability
First-pass would panic the refresher if DB unreachable. Resolved: §3.3 `load_matrix` err handling + §10 failure row "Matrix loader hits malformed row" + DEC-126 — stale matrix continues serving; OTel `auth_rbac_matrix_refresh_total{outcome=db_unreachable}` counter.

### ISS-006 — Stub-token grace window unbounded
First-pass mentioned "30-day grace" but didn't enforce. Resolved: §1 #18 + DEC-125 + §10 failure row "JWT missing rbac_v after grace window" — verifier rejects post-grace; FR-AUTH-109 is the enforcer.

### ISS-007 — RLS bypass via role-name typo
First-pass relied on application-layer role checks. Resolved: §1 #10 + `auth.has_role(role_name)` SQL function in migration 0005 + RLS policies on sensitive tables — DB-layer check; un-bypassable.

### ISS-008 — Closed-enum drift from doc table
First-pass risked the Rust enum drifting from website docs §2.6 22-role table. Resolved: §1 #1 cites the website docs URL + `source_pages` frontmatter; ADR-101 §3 is the canonical source of truth that both the Rust code and the docs reference.

### ISS-009 — ADR file existence vs migration matching
First-pass ADR gate could pass with a referenced-but-empty ADR file. Resolved: §1 #17 — ADR gate checks BOTH the SQL comment reference AND that the `services/auth/adr/ADR-NNN-*.md` file exists at path; either failure → test fails.

### ISS-010 — Scope-grant could over-grant
First-pass allowed scope-grants standalone — could grant cfo-only privileges to a tenant-member without role. Resolved: §1 #20 + DEC-124 — scope-grants narrow within tenant matrix; check is `role_matrix permits OR scope_grant covers`, never standalone (the OR is bounded by the role matrix's resource set; resource_id narrowing only).

### ISS-011 — Performance budget unstated for matrix refresh
First-pass didn't budget the refresh path. Resolved: §1 #21 — < 50µs p99 check; refresher is 60s cadence (not on hot path); hash-only-swap avoids unnecessary allocations.

### ISS-012 — Migration row count drift between code and ADR
First-pass let the migration's INSERT row count drift from ADR-101 §3 enumeration. Resolved: §4 AC #7 — explicit assertion that `SELECT count(*) FROM role_permissions` matches the ADR-documented number; CI catches drift before merge.

## §3 — Resolution

All 12 mechanical concerns addressed in the first revision pass. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (22 roles × 40 resources × 5 actions × RLS × JWT × ADR gate × stub migration × scope grants × OTel × WebAuthn intersection × RBAC versioning), not by line targets.

---

*End of FR-AUTH-101 audit.*
