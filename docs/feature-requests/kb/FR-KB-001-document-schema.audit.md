---
fr_id: FR-KB-001
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

FR-KB-001 ships the KB Document schema — slug + markdown body + YAML frontmatter + closed category enum + 3-tier ACL + immutable versions + translation_of. Scope: 27 §1 normative clauses covering 4 closed Postgres enums (category 5, permission 3, language 2, status 3), append-only versions via SQL grant, per-tenant per-language slug uniqueness, translation_of cross-language enforcement, role_restricted role validation against FR-AUTH-101, frontmatter YAML schema with deny_unknown_fields, server-computed body_sha256, monotonic version_number trigger, REST surface (create/get/list/version/archive/patch), 4 memory audit kinds with sev-2 ACL events, two SQL views + version chain walker, owner FK, performance budget. 18 rationale paragraphs. §3 contains: migration 0001 (2 tables + 4 enums + 4 triggers + RLS + REVOKE), migration 0002 (views + walker), Rust types, frontmatter validator, REST handler. 30 ACs. 33 failure-mode rows. 23 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Mutable versions could rewrite history
First-pass allowed UPDATE on document_versions. Resolved: §1 #8 + DEC-247 + `REVOKE UPDATE, DELETE FROM cyberos_app`; AC #14 + #15.

### ISS-002 — Slug uniqueness scope ambiguous
First-pass had `UNIQUE (slug)`; broke translation pairs. Resolved: §1 #9 + DEC-244 + `UNIQUE (tenant_id, language, slug)`; AC #7 + #8.

### ISS-003 — translation_of could form invalid pairs
First-pass had no trigger checking cross-tenant / same-language linkage. Resolved: §1 #10 + DEC-245 + `enforce_translation_of` trigger; AC #9–#11.

### ISS-004 — role_restricted with empty/unknown roles silently over-permissions
First-pass had no validator. Resolved: §1 #11 + DEC-250 + handler validation + DB trigger; AC #12 + #13.

### ISS-005 — Frontmatter unknown keys silently ignored
First-pass used default serde deserialisation. Resolved: §1 #12 + `serde(deny_unknown_fields)`; AC #21.

### ISS-006 — applicability_tags allowed on non-runbook
Resolved: §1 #12 + validator + AC #22.

### ISS-007 — body_sha256 client-supplied
First-pass let client compute. Resolved: §1 #13 + server-side `sha2::Sha256` hash; AC #18.

### ISS-008 — version_number races under concurrent saves
First-pass had application-layer numbering. Resolved: §1 #26 + `assign_version_number` trigger inside transaction; AC #16.

### ISS-009 — ACL changes lacked sev-2 audit row
First-pass emitted only `kb.document_versioned`. Resolved: §1 #16 + DEC-249 + dedicated `kb.document_acl_changed` at sev-2; AC #24.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (4 closed enums × RLS × append-only versions × monotonic version numbering × translation pair enforcement × role-restricted validation × frontmatter schema × server-side hashing × atomic INSERT-and-update-pointer × 4 memory audit kinds × 2 SQL views + walker × REST + idempotency × OTel × archive workflow), not by line targets.

---

*End of FR-KB-001 audit.*
