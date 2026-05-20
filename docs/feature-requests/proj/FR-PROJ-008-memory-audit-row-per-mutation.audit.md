---
fr_id: FR-PROJ-008
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 15
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..015 added)
---

## §1 — Verdict summary

FR-PROJ-008 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 20 §1 clauses (history_event table, dual-write semantics, bidirectional linkage, field-level diff, append-only, chain_anchor verify, query API, since param, memory row payload, metrics, RLS, 60s cache, background sweep, pagination, session_id correlation, summary endpoint, PII redaction, mutation_source, optimistic-concurrency check, session_summary query). 16 §2 rationale paragraphs. §3 contains: migration + history_event + MutationKind enum + emit_mutation with 3-step linkage + list_history with verify. 24 ACs. §10 lists 28 failure rows. §11 lists 23 implementation notes covering cache scope rationale, sweep mechanics, pagination cursor choice, session correlation source, summary aggregation method, PII allowlist semantics, mutation_source defaults, optimistic-concurrency necessity, chain-verify-vs-storage tradeoff.

## §2 — Findings (all resolved)

### ISS-001 — Dual-store linkage
Without bidirectional linkage, tamper of one without the other is silent. Resolved: §1 #3 cross-references both directions; AC #5 #6.

### ISS-002 — Y.Text raw vs hash in history
Raw text bloats DB + duplicates Y.Doc snapshots. Resolved: §1 #4 hash-only; AC #13.

### ISS-003 — Append-only enforcement
Without RLS REVOKE, history is mutable. Resolved: §1 #5 + migration REVOKE UPDATE/DELETE from cyberos_app; AC #9.

### ISS-004 — Chain verify timing (write vs read)
Write-time verify misses post-write tampering. Resolved: §1 #6 query-time verify; AC #8 detects tamper.

### ISS-005 — Same-transaction guarantee
If memory emit succeeds but history INSERT fails, divergence. Resolved: §3 emit_mutation takes `&mut PgTransaction`; AC #11 verifies rollback.

### ISS-006 — Linkage update timing
Insert history first, then memory emit, then UPDATE history with chain_anchor — three steps. Resolved: §3 sequence + same-tx; chain_anchor placeholder initially, updated post-emit.

### ISS-007 — Verify is hot-path expensive (strict-redo pass)
Issue page load fetches 50+ history rows; each verify = memory round-trip ≈ 10ms → 500ms page-load delay. Resolved: §1 #12 + 60s in-memory cache + AC #16.

### ISS-008 — Reactive verify only (strict-redo pass)
Tampering of older rows may go undetected until queried. Resolved: §1 #13 + hourly background sweep + `proj.chain_tampered` SEV-1 + AC #17.

### ISS-009 — Unbounded result set (strict-redo pass)
Long-lived issues accumulate thousands of mutations; without pagination, queries slow + clients break. Resolved: §1 #14 + before_seq/after_seq cursor pagination + AC #18.

### ISS-010 — No multi-field PATCH correlation (strict-redo pass)
A PATCH that changes 3 fields produces 3 history events with no link back to the originating request. Resolved: §1 #15 + session_id (correlates with FR-PROJ-002) + AC #19.

### ISS-011 — No aggregated activity view (strict-redo pass)
UI sparklines need per-day aggregates; raw fetch is N round-trips of aggregation client-side. Resolved: §1 #16 + summary endpoint + AC #20.

### ISS-012 — PII bleeds into long-term audit (strict-redo pass)
Sensitive scalars (employee email, phone) stored in audit verbatim accumulate years of PII. Resolved: §1 #17 + per-tenant allowlist + admin-only raw fetch + AC #21.

### ISS-013 — Mutation channel invisibility (strict-redo pass)
Investigators couldn't tell "this came from CLI vs web vs bulk-admin." Resolved: §1 #18 + mutation_source field + AC #22.

### ISS-014 — Lost-update bug surface (strict-redo pass)
Two concurrent operators editing same field overwrote each other without warning. Resolved: §1 #19 + optimistic-concurrency check + 409 on before_mismatch + AC #23.

### ISS-015 — Session reconstruction client-side (strict-redo pass)
Clients aggregated session_id manually — inefficient + inconsistent. Resolved: §1 #20 + server-side include_session_summary + AC #24.

## §3 — Resolution

All 15 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (dual-store linkage × chain-anchor verification × cache × sweep × pagination × session correlation × summary × PII × source × concurrency check), not by line targets.

---

*End of FR-PROJ-008 audit.*
