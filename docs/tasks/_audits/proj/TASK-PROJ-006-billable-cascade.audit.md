---
task_id: TASK-PROJ-006
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..016 added)
---

## §1 — Verdict summary

TASK-PROJ-006 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 21 §1 clauses (4-tier cascade, provenance, first-match-wins, snapshot-at-write-time, audit, scoping, deterministic, metrics, REST preview, override CRUD, RLS, handler input validation, bulk-resolve endpoint, immutable snapshot on time entry, engagement-default task-class billable, p95<20ms budget, historical resolution query, effective_overrides_applied metadata, per-tenant fallback override, archived-engagement rejection, explain mode). 16 §2 rationale paragraphs. §3 contains: 2 migrations, TaskClass + Tier enums, BillableResolution struct, resolve() with all 4 tiers. 25 ACs. §10 lists 28 failure rows. §11 lists 22 implementation notes covering bulk cap rationale, force-re-resolution gating, engagement-default mechanics, latency budget breakdown, explain-mode parallel queries, cache rejection rationale, cascade-order justification, 5th-tier rejection rationale.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Snapshot vs retroactive recompute
Without snapshot, override changes corrupt invoiced entries. Resolved: §1 #3 + DEC-271 snapshot at write-time; AC #6.

### ISS-002 — Fallback default value
Without explicit fallback, behaviour is implementation-defined. Resolved: §1 Tier 4 + DEC-272 = false; AC #5 verifies.

### ISS-003 — First-match-wins precedence
Without "stop at first hit", overrides combine unpredictably. Resolved: §1 #2 + AC #2 (Tier 1 false stops cascade).

### ISS-004 — Audit provenance
Without source tier in audit row, "why was this billable?" is unanswerable. Resolved: §1 #4 + AC #9 row carries `source_tier`.

### ISS-005 — Cross-engagement scope
Without scoping, a member override leaks across engagements. Resolved: §1 #5 + AC #7 PK (member, engagement).

### ISS-006 — Preview vs write difference
UI showing "Billable: ?" needs preview without writing. Resolved: §1 #8 + AC #8 dedicated REST endpoint.

### ISS-007 — Handler input validation absent (strict-redo pass)
Resolver assumes valid inputs; bad inputs (non-existent member, far-past date) would silently fall through to Tier 4. Resolved: §1 #12 + structured 400 + AC #15.

### ISS-008 — Bulk import overhead (strict-redo pass)
Per-entry HTTP cost for timesheet imports is prohibitive. Resolved: §1 #13 + /bulk endpoint with 1000-cap + AC #16 + AC #17.

### ISS-009 — Snapshot mutability path missing (strict-redo pass)
Without explicit immutability on time entry, accidental PATCH would corrupt bill. Resolved: §1 #14 + 405 rejection + AC #18.

### ISS-010 — Engagement-default task-class setup overhead (strict-redo pass)
Operators creating multiple engagements with similar billable patterns set 8 rows per engagement. Resolved: §1 #15 + tenant default + copy-at-creation + AC #19.

### ISS-011 — Resolver latency unbounded (strict-redo pass)
Time-entry write is in operator UI critical path; >20ms feels laggy. Resolved: §1 #16 + p95 budget + histogram + AC #20.

### ISS-012 — Historical resolution forensics missing (strict-redo pass)
Auditor question "why was this billed?" required cross-tracing audit rows manually. Resolved: §1 #17 + /history endpoint + AC #21.

### ISS-013 — Operator transparency on tier matches (strict-redo pass)
Resolution returned winner only; couldn't tell if lower tiers existed but matched the same value. Resolved: §1 #18 + effective_overrides_applied metadata + AC #22 + parallel queries.

### ISS-014 — Global fallback inflexible (strict-redo pass)
Some tenants (internal cost-center billing) want default-billable; spec was global-conservative. Resolved: §1 #19 + per-tenant cascade_fallback_billable + AC #23 + safe default.

### ISS-015 — Archived engagement accepted (strict-redo pass)
Resolutions on archived engagements would create new time entries on closed billing periods. Resolved: §1 #20 + archive check at top of resolver + AC #24.

### ISS-016 — Diagnostic insight missing (strict-redo pass)
Operator debugging "why this value?" had no introspection path. Resolved: §1 #21 + explain mode + AC #25.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (4-tier cascade × snapshot immutability × bulk-resolve × engagement defaults × historical forensics × explain mode × per-tenant policy × archived guard), not by line targets.

---

*End of TASK-PROJ-006 audit.*
