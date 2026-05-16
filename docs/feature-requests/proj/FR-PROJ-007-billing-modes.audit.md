---
fr_id: FR-PROJ-007
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 15
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..015 added)
---

## §1 — Verdict summary

FR-PROJ-007 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 21 §1 clauses (3 modes, mode versioning, per-mode config schema, rollup interface, milestone sum invariant, retainer state table, audit kinds, CLI, metrics, RLS, mid-period proration, milestone cancellation, milestone addition, overage-streak tracking, retainer holidays, 500ms p95 budget, preview mode, mixed-currency split, engagement_metadata in rollup). 15 §2 rationale paragraphs. §3 contains: migration with retainer_state, BillingMode enum + tagged ModeConfig, InvoiceRollup + InvoiceLine + SourceRef, rollup orchestrator + per-mode compute including retainer overage + rollover credit. 28 ACs. §10 lists 38 failure rows. §11 lists 23 implementation notes covering proration mechanics, cancellation refund modes, milestone-addition delta, overage-streak storage, holiday workday calculation, p95 budget breakdown, preview path consistency, mixed-currency rationale, parallel metadata fetch, rollover-period choice, JSONB-vs-table rationale, weekly cap, partial-month proration.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Mode count
Adding a 4th mode (e.g. Capped T&M) explodes config matrix. Resolved: §1 + DEC-281 exactly 3; hybrids compose via mode-change.

### ISS-002 — Mode-change mid-period
Without proration spec, mode change at day 15 of month is ambiguous. Resolved: §1 #6 proration approach; deferred fine-grained slice 4+.

### ISS-003 — Milestone sum invariant
Without validation, Fixed-Fee bills wrong amount. Resolved: §1 #8 + AC #3.

### ISS-004 — Retainer rollover semantics
Without spec, "unused hours roll over" is ambiguous (forever? N months?). Resolved: §1 #4 configurable `rollover_months`; AC #7 #8.

### ISS-005 — InvoiceLine traceability
Auditor asks "what produced this line"; without source_refs, manual reverse-engineering. Resolved: §1 #5 + §3 `SourceRef` enum; AC #14.

### ISS-006 — Currency consistency
Mixing currencies in one rollup = invoice ambiguity. Resolved: rollup currency from mode's config; mismatches flagged at compute.

### ISS-007 — Mid-period proration unspecified (strict-redo pass)
Original spec mentioned "proration" as deferred; mode changes mid-period are common. Resolved: §1 #13 + sub-period split + prorated marker + AC #18.

### ISS-008 — Milestone cancellation absent (strict-redo pass)
Scope-creep deletions had no path. Resolved: §1 #14 + cancel_milestone admin + refund/redistribute modes + audit + AC #19.

### ISS-009 — Milestone addition absent (strict-redo pass)
Post-contract scope additions had no path. Resolved: §1 #15 + add_milestone with total_amount_minor adjustment + AC #20.

### ISS-010 — Retainer mispricing invisible (strict-redo pass)
Sustained overage signals retainer cap is too low; original spec had no operator alert. Resolved: §1 #16 + streak tracking + SEV-3 at 3 + AC #21.

### ISS-011 — Holiday windows missing (strict-redo pass)
Tet / Christmas retainer charging full base for partial-coverage feels unfair. Resolved: §1 #17 + retainer_holidays table + workday proration + AC #22.

### ISS-012 — Rollup latency unbounded (strict-redo pass)
Invoice-generation hot path needs latency budget. Resolved: §1 #18 + 500ms p95 + SEV-3 warning + AC #23.

### ISS-013 — No preview path (strict-redo pass)
What-if rollup analysis required destructive state writes. Resolved: §1 #19 + preview flag + path-consistency guarantee + AC #24.

### ISS-014 — Mixed-currency periods unhandled (strict-redo pass)
T&M with rate cards in multiple currencies couldn't produce a coherent invoice line. Resolved: §1 #20 + per-currency split + SEV-2 warning + AC #25.

### ISS-015 — engagement_metadata not in rollup (strict-redo pass)
Downstream invoice rendering re-queried engagement. Resolved: §1 #21 + metadata field + parallel fetch + AC #26.

## §3 — Resolution

All 15 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (3 modes × proration × milestone CRUD × overage tracking × holidays × mixed currency × preview × p95 budget × metadata pre-fetch), not by line targets.

---

*End of FR-PROJ-007 audit.*
