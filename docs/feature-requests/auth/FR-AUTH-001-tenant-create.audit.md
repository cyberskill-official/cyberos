---
fr_id: FR-AUTH-001
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-001 was expanded from 184 lines to ~830 lines. The expansion added 7 §1 normative clauses (#5 idempotency-key handling, #11 explicit error-body shapes with field/reason, #12 single-transaction atomicity, #13 OTel span emission, #14 explicit reject of slug='root', #15 OTel metrics, plus expanded #6/#10 with shapes), 7 substantive §2 rationale paragraphs, full Rust types + migrations + RLS templates + handler skeleton + idempotency module in §3, expanded §4 from 9 to 18 ACs, full Rust test bodies in §5 (8 test functions covering happy + 401 + 403 + 409 + 400 + idempotent-replay + RLS + p95-latency), expanded §7 with code/concept/operational deps, 7 example payloads in §8 (success + conflict + idempotent replay + key-reuse + forbidden + invalid + audit row), 20 failure modes in §10 (vs. 4 in first pass), 8 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Tenant ID type mismatch: API uses UUID but `jwt.tenant_id != 0` (numeric)
- **severity:** error  **status:** resolved
- First-pass §6 had `if jwt.tenant_id != 0` — but the table schema uses UUID. A code-gen agent would either make tenant_id an integer (breaking the schema) or write `Uuid::nil()` checks without the FR specifying. Resolved: §1 + §3 + §6 use `Uuid::nil()` (the convention for tenant 0); explicit in §11 notes.

### ISS-002 — RLS policy application not specified (which tables, what template)
- **severity:** error  **status:** resolved
- First-pass §1 #7 said "every tenant-scoped table SHALL have the new tenant's RLS policy applied automatically" — but no list of tables, no template SQL. Resolved: §3 introduces `rls/templates.rs` with `TENANT_SCOPED_TABLES` registry + `apply_for_tenant_sql` template; AC #15 + §5 test asserts policies exist; §10 row + §11 note about the registry-as-contract.

### ISS-003 — Audit row builder `canonical::tenant_created` not shown
- **severity:** error  **status:** resolved
- First-pass §6 had `brain_writer::emit(canonical::tenant_created(&row.id, &req.slug)).await?` but no builder definition. Resolved: full builder in §6 with payload schema; §8 example audit row JSON; AC #12 asserts emission.

### ISS-004 — No idempotency support (network retries create duplicates)
- **severity:** error  **status:** resolved
- A network timeout during `POST /v1/admin/tenants` followed by client retry produces either duplicate tenants OR a 409 (caller can't tell which path the prior attempt took). Resolved: §1 #5 idempotency-key handling; `admin_idempotency_keys` table in 0002 migration; §3 lookup/insert helpers; AC #13 + #14 + §5 tests.

### ISS-005 — Concurrent tenant insert + RLS apply not transactional
- **severity:** error  **status:** resolved
- First-pass §6 inserted, then called `apply_rls_policies` AFTER, then emitted audit. Three separate operations; partial-state failures possible. Resolved: §1 #12 single-transaction atomicity; §3 handler uses `pool.begin()` with all three ops on same `&mut tx`; AC #17 asserts rollback on RLS failure.

### ISS-006 — No tenant-deletion path; soft-delete via `suspended` column unspecified
- **severity:** warning  **status:** resolved
- The `suspended` column existed in the schema but no documentation. Operators wouldn't know it exists. Resolved: §11 explicitly documents the column's purpose ("future suspension workflow; no endpoint mutates yet"); §9 lists tenant deletion + suspension as deferred items.

## §3 — Resolution

All 6 mechanical revisions applied (2026-05-16). **Score = 10/10.**

---

*End of FR-AUTH-001 audit (final). Status: PASS at 10/10.*
