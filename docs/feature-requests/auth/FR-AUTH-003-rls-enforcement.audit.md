---
fr_id: FR-AUTH-003
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-003 expanded from 76 lines to ~900. Added 7 §1 clauses (#2 USING + WITH CHECK both required, #5 cyberos_ops audit row, #8 surface 42501 as 403, #9 boot-time check, #11 broad CI path filter, #12 metrics, expanded #1 with full table list). Full SQL migrations + registry + with_tenant helper + boot check in §3. 16 ACs. 5 full Rust test bodies including proptest. CI workflow YAML. 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — SQL injection via `format!` interpolation in SET LOCAL
First-pass §6 had `format!("SET LOCAL app.tenant_id = '{}'", tenant_id)`. Even with Uuid type, the principle "never interpolate, always bind" applies. Resolved: §1 #3 normative; `with_tenant` helper uses `sqlx::query("...").bind(tenant_id.to_string())`; AC #13 + code-grep lint forbids `format!` near SET LOCAL.

### ISS-002 — Table list incomplete (`[...every other tenant table]`)
First-pass §1 #1 had `[...every other tenant table]` placeholder. Code-gen agent has nothing to enumerate. Resolved: §1 #1 explicit list of 12 slice-1 tables; `TENANT_SCOPED_TABLES` registry in `rls/registry.rs`; CI test asserts no orphan tables.

### ISS-003 — `cyberos_ops` bypass role unspecified (who has it, what it does)
First-pass §10 mentioned "cyberos_ops requires sev-1 access logging" but no role definition, no audit row spec. Resolved: §1 #5 normative + 0004_rls_roles.sql migration; `auth.rls_bypass_used` audit row builder; AC #7 + §5 test asserts emit; sev-2 alarm on baseline drift.

### ISS-004 — Property test pattern not aligned with FR-AI-018
First-pass §1 #4 mentioned "1000 random tenant pairs × 10K queries × ZERO cross-tenant reads" but didn't reference FR-AI-018's proven pattern. Resolved: §1 #6 explicitly mirrors FR-AI-018; §5 has full proptest body; CI workflow added with non-skip enforcement.

### ISS-005 — `SET` vs `SET LOCAL` distinction not enforced
First-pass §6 used `SET LOCAL` correctly but didn't explain WHY (connection-pool contamination). Without explanation, future engineers might "fix" by switching to `SET` for "performance" — catastrophic. Resolved: §1 #3 + §2 paragraph + §10 row + §11 note all reinforce the SET LOCAL discipline; code-grep lint catches `SET app.tenant_id` (without LOCAL).

### ISS-006 — WITH CHECK not specified — silent wrong-tenant inserts possible
First-pass §3 had `CREATE POLICY ... USING (...)` only. Without WITH CHECK, INSERT with wrong tenant_id succeeds silently (USING filters subsequent SELECTs, hiding the row). Resolved: §1 #2 normative requires both clauses; §3 migration shows both; AC #3 + §5 test asserts INSERT rejection; §2 rationale paragraph explains the silent-failure mode.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-AUTH-003 audit.*
