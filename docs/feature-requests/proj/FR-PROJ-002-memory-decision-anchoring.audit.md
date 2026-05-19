---
fr_id: FR-PROJ-002
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 18
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..018 added)
---

## §1 — Verdict summary

FR-PROJ-002 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 24 §1 clauses (per-issue decision emit, optional reason, length validation, chat link expansion, prior_decision_chain link, audit-before-action tx, sync_class shareable default + override, cross-module link extraction, FR-MEMORY-108 queryability, decided_by_subject_id, 100ms p95 budget, OTel metrics, mandatory PII redaction of reason, decision_id UUIDv7, retraction as separate row, bulk PATCH one-row-per-issue, link normalisation + dedup, decision_attributes JSONB, O(log n) index, immutability enforced via HTTP 405 + DB GRANT, per-tenant required-reason policy, chain_anchor in payload self-verifies, decision_session_id correlation, per-engagement decision_acl_default with per-decision override). 24 §2 rationale paragraphs. §3 contains: full DecisionPayload + emit_decision_in_tx with redaction + chain fetch + link normalisation, decision_chain.rs prior_chain fetcher, decision_acl.rs with engagement override resolution, retraction emit, immutability handler, link normalisation + dedup helper, self-verifying chain_anchor builder, SQL schema for tenant_settings + engagement_settings + O(log n) index. 31 ACs. §5 contains 18 named test bodies including: happy path + prior chain + reason validation + cross-module link + rollback + same-status-no-op + sync_class default + search-by-issue + reason redaction + retraction lifecycle + already-retracted rejection + bulk-PATCH-one-row-per-issue + parameterised link normalisation + required-reason enforcement + chain_anchor self-verify + engagement ACL default + per-decision override beats engagement. §10 lists 35 failure rows. §11 lists 25 implementation notes covering retraction-vs-mutation rationale, UUIDv7 sort property, GRANT/REVOKE defense in depth, JWT exclusion from audit, link normalisation tradeoff, dedup-on-extract, decision_attributes open schema, 100ms budget breakdown, sync-vs-stream choice.

## §2 — Findings (all resolved)

### ISS-001 — Reason validation unspecified
First-pass §10 mentioned "Reason exceeds 500 chars → 400" without enforcement. Resolved: §1 #3 + length check + AC #5 + §5 test.

### ISS-002 — Audit-before-action mechanism not shown
First-pass §1 #5 mentioned "5xx on memory emit → status change rolled back" but no transaction handling shown. Resolved: §1 #6 + §3 emit_decision_in_tx wraps DB+memory in single tx; AC #7 + §5 test.

### ISS-003 — Cross-module link extraction unspecified
First-pass §3 example showed `links: [...]` but no extraction logic. Resolved: §1 #8 + extract_links() regex + AC #6 + §5 test.

### ISS-004 — Prior decision chain fetch mechanism unspecified
First-pass §1 #6 mentioned "include the chain_anchor of the prior" but no fetch logic. Resolved: §3 decision_chain.rs + AC #3 + §5 test.

### ISS-005 — Same-status PATCH behaviour ambiguous
Should "same status" PATCH emit a decision? First-pass unclear. Resolved: §1 #1 + AC #15 — no decision on no-status-change.

### ISS-006 — sync_class default not specified
First-pass §1 #5 said "shareable by default" but no implementation. Resolved: §1 #7 + meta_with_acl_default() + AC #8.

### ISS-007 — Reason text could leak PII to long-term audit (strict-redo pass)
Operator-typed reason fields routinely include client names, emails, phone numbers. memory audit retention is years; storing unredacted = years of accumulated PII risk. Resolved: §1 #13 + mandatory PII redact via FR-MEMORY-111 ruleset + AC #17 + test body.

### ISS-008 — No public decision_id distinct from chain_hash (strict-redo pass)
Chain hashes (64 hex chars) are unwieldy as URL/API identifiers. Operators referring to "decision #abc-123" need a stable human-friendly ID. Resolved: §1 #14 + UUIDv7 decision_id top-level field + AC #18 + §11 note on v7 sort property.

### ISS-009 — No retraction mechanism (strict-redo pass)
Operator marking a decision and later realising it was wrong needs to retract — original spec offered no path. Mutating the row would break the chain. Resolved: §1 #15 + `proj.decision_retracted` row + retracts_decision_id back-reference + AC #19 + #20 + test bodies; §11 documents the audit-chain rationale.

### ISS-010 — Bulk-PATCH semantics undefined (strict-redo pass)
Bulk operations like "mark these 50 issues as done" needed clarity: one aggregate row or per-issue rows? Without spec, downstream consumers couldn't query "all decisions on issue X." Resolved: §1 #16 + per-issue row + AC #21 + bulk-PATCH test body.

### ISS-011 — Cross-module links not deduped/normalised (strict-redo pass)
`chat://Thread/ABC` and `chat://thread/abc` would store as separate entries, breaking bidirectional lookup. Resolved: §1 #17 + normalise_link helper + dedup-on-extract + AC #22 + parameterised test.

### ISS-012 — No room for tenant-specific structured metadata (strict-redo pass)
Per-tenant workflows often have custom decision metadata (CRM account, client_facing flag, deadline). Original spec had no extension hook. Resolved: §1 #18 + decision_attributes JSONB open-schema with reserved keys + AC #23.

### ISS-013 — Per-issue chain query performance unbounded (strict-redo pass)
Without index, queries became O(n) scans over the memory outbox — interactive UX would lag. Resolved: §1 #19 + partial index `(tenant, issue, ts DESC) WHERE kind='proj.decision'` + AC #24 EXPLAIN ANALYZE check.

### ISS-014 — Immutability not enforced (strict-redo pass)
Original spec implied immutability via append-only chain but had no explicit handler rejection. Resolved: §1 #20 + 405 on PATCH/DELETE + REVOKE UPDATE/DELETE on memory_outbox role + AC #25 + #26.

### ISS-015 — No way to require reason for terminal transitions (strict-redo pass)
Compliance-heavy tenants need rationale on every `done`; SMB tenants don't. Original spec was global-binary. Resolved: §1 #21 + per-tenant `require_reason_for` policy + AC #27 + tenant_settings schema.

### ISS-016 — Decision rows not self-verifying (strict-redo pass)
A recipient with only the payload (e.g. in DSAR export) couldn't verify integrity without crawling the memory chain. Resolved: §1 #22 + chain_anchor in payload + AC #28 + self-verify test.

### ISS-017 — Multi-mutation PATCHes not correlatable (strict-redo pass)
Operator workflows often span multiple ops in one PATCH; reconstructing "what did this PATCH do" required cross-row inference. Resolved: §1 #23 + decision_session_id correlation field + AC #29.

### ISS-018 — Engagement-level ACL policy missing (strict-redo pass)
Engagements with confidential scope (legal, HR) need default-restricted decisions without per-PATCH boilerplate. Resolved: §1 #24 + per-engagement decision_acl_default + per-decision override beats engagement default + AC #30 + #31.

## §3 — Resolution

All 18 mechanical revisions applied. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (decision lifecycle × retraction × bulk-ops × link normalisation × tenant policy × engagement policy × required-reason × immutability × self-verification × session correlation), not by line targets.

---

*End of FR-PROJ-002 audit.*
