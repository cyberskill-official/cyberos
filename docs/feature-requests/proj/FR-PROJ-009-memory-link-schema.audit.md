---
fr_id: FR-PROJ-009
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..017 added)
---

## §1 — Verdict summary

FR-PROJ-009 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 20 §1 clauses (table, 3 link types, validation, soft-delete, REST endpoints, audit kinds, duplicate prevention, RLS, metrics, annotation field, cycle detection, link_strength, batch create, graph traversal, link transfer on split, non-empty removal_reason, cites_with_quote, metadata JSONB, review-pending state, traversal audit sampling). 17 §2 rationale paragraphs. §3 contains migration with all fields + Rust LinkType + MemoryLink + create/remove logic. 26 ACs. §10 lists 30 failure rows. §11 lists 24 implementation notes covering memory_row_id usage, scope integration with FR-SKILL-103, NS timestamps, soft-delete audit pairing, annotation redaction, cycle DFS bounds, batch concurrency model, graph limits, link-strength operator-typed rationale, sample-rate calibration, UUIDs vs auto-increment.

## §2 — Findings (all resolved)

### ISS-001 — Link type count
3 vs 5 vs configurable. Resolved: §1 #2 + §2 cites/implements/supersedes are the empirical core.

### ISS-002 — Validation at write-time
Without it, dangling links accumulate. Resolved: §1 #3 + DEC-301 + AC #4.

### ISS-003 — Soft vs hard delete
Hard delete loses audit. Resolved: §1 #4 soft-delete with reason; AC #8 #11.

### ISS-004 — Forward-only supersedes
Without invariant, link semantics invert. Resolved: §1 #3 + DEC-302 + AC #6.

### ISS-005 — Duplicate semantics
Same memory + same issue could be both cites AND implements. Resolved: §1 #7 + UNIQUE on (issue, path, link_type); AC #2.

### ISS-006 — Cross-tenant
Without check, link reveals existence. Resolved: §1 #3 + AC #7.

### ISS-007 — Link semantics lost without annotation (strict-redo pass)
Pure link type loses "see section 3" context. Resolved: §1 #10 + annotation field + PII redact + AC #16.

### ISS-008 — Supersedes cycles allowed (strict-redo pass)
A supersedes B + B supersedes A creates undefined state. Resolved: §1 #11 + DFS cycle detection + AC #17.

### ISS-009 — Link strength unsignaled (strict-redo pass)
Citation drift alerts should prioritise strong cites; original spec had no strength field. Resolved: §1 #12 + link_strength enum + AC #18.

### ISS-010 — Bulk creation expensive (strict-redo pass)
Migration flows create many links per issue; per-link HTTP slow. Resolved: §1 #13 + batch endpoint with 50-cap + AC #19.

### ISS-011 — No multi-hop traversal (strict-redo pass)
UI knowledge-graph features need depth-N queries. Resolved: §1 #14 + graph endpoint with depth + types filter + AC #20.

### ISS-012 — Split semantics undefined (strict-redo pass)
When issue is split, link destination undefined. Resolved: §1 #15 + default-first sub-issue + operator override + AC #21.

### ISS-013 — Empty removal_reason silent (strict-redo pass)
Soft-remove without reason loses audit value. Resolved: §1 #16 + 400 if empty + AC #22.

### ISS-014 — Legal-citation quote loss (strict-redo pass)
Heavy-citation workflows need to preserve exact text. Resolved: §1 #17 + cites_with_quote variant + AC #23.

### ISS-015 — Per-tenant extension blocked (strict-redo pass)
Tenant-specific tags/compliance refs had no home. Resolved: §1 #18 + metadata JSONB + AC #24.

### ISS-016 — No curation policy (strict-redo pass)
Some tenants want admin-review before links activate. Resolved: §1 #19 + review-pending state + per-tenant toggle + AC #25.

### ISS-017 — Traversal analytics missing (strict-redo pass)
Which links are heavily traversed? Operator question went unanswered. Resolved: §1 #20 + traversal audit row + 10% sample + AC #26.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (3 link types × validation × soft-delete × cycle detection × graph traversal × annotation × quote × strength × batch × split-transfer × review state × traversal analytics × metadata extension), not by line targets.

---

*End of FR-PROJ-009 audit.*
