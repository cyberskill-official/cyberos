---
fr_id: FR-TIME-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 10
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-TIME-001 ships the TimeEntry append-only schema — the invoice-grade hours record. Scope: 25 §1 normative clauses covering closed entry_kind (4) + entry_status (4) enums, correction_to self-FK with acyclic + chain-topology + cross-scope enforcement, RLS isolation, SQL-grant append-only, 1–1440-minute per-row cap, rate_card_snapshot JSONB, multi-currency snapshot, current_time_entries_view + entry_chain_walker function, REST surface (create/correct/get/list with idempotency + history), OTel emission, PII scrubbing of description, FK to auth.subjects with ON DELETE RESTRICT, performance budget, audit-row pair (recorded + corrected). 18 rationale paragraphs. §3 contains: migration 0001 (table + 4 triggers + 2 enums + RLS + REVOKE), migration 0002 (current view + chain walker), Rust types, validation, REST handlers. 26 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Mutations could rewrite history
First-pass allowed UPDATE on time_entries (operator convenience). Resolved: §1 #5 + DEC-230 + feature-request-audit skill rule 12 + `REVOKE UPDATE, DELETE FROM cyberos_app`; AC #8 + AC #9.

### ISS-002 — Tree-shaped corrections (two correctors of same prior) caused ambiguous "current value"
Resolved: §1 #8 + DEC-226 + `enforce_chain_topology` trigger + AC #13.

### ISS-003 — Cycle in correction_to chain
Resolved: §1 #7 + DEC-225 + `detect_correction_cycle` trigger walking up to depth 100 + AC #14, #15.

### ISS-004 — Cross-scope corrections allowed silent engagement rewrite
First-pass allowed correction to mutate engagement_id. Resolved: §1 #11 + `enforce_correction_inheritance` trigger checking (tenant_id, engagement_id, issue_id, member_subject_id) inheritance; AC #11, #12.

### ISS-005 — Rate-card mutation retroactively shifted past billable amounts
Resolved: §1 #21 + DEC-224 + `rate_card_snapshot JSONB` on the row (snapshot pattern); FR-TIME-005 fills.

### ISS-006 — Per-row duration unbounded; single buggy entry could log a year
Resolved: §1 #1 + DEC-227 + DEC-228 + DB CHECK constraint `BETWEEN 1 AND 1440`; AC #5, #6.

### ISS-007 — Future-dated entries accepted
Resolved: §1 #10 + 5-minute future tolerance + handler validation; AC #7.

### ISS-008 — `entry_kind` open-ended
Resolved: §1 #3 + 4-value closed Postgres enum + `EntryKind::ALL` cardinality test; AC #1.

### ISS-009 — Current-effective row predicate scattered
First-pass left downstream FRs to NOT-IN-subquery on every read. Resolved: §1 #9 + `current_time_entries_view` SQL view + index on `correction_to WHERE correction_to IS NOT NULL`; AC #16.

### ISS-010 — Description PII could leak via memory chain
First-pass stored raw description in memory row. Resolved: §1 #13 + FR-MEMORY-111 PII scrubbing before chain commit; description retained in tenant-scoped Postgres row.

## §3 — Resolution

All 10 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (append-only via SQL grant × correction-via-new-row × acyclic chain × tree-rejection × cross-scope rejection × rate-card snapshot × multi-currency × per-row duration cap × RLS isolation × audit chain × view + walker × REST + idempotency × OTel), not by line targets.

---

*End of FR-TIME-001 audit.*
