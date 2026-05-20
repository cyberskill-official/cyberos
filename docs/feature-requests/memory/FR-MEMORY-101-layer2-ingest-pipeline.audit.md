---
fr_id: FR-MEMORY-101
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
---

## §1 — Verdict summary

FR-MEMORY-101 expanded from 117 lines to ~810. Added 6 §1 clauses (#10 BGE retry budget; #11 AGE graceful-fallback explicit; #12 frame-level CRC validation; #13 concurrent multi-tenant tokio tasks; #14 graceful shutdown; #15 --rebuild flag for FR-MEMORY-102). 8 §2 rationale paragraphs. Full Rust ingest loop + chain_anchor module + migrations + RLS integration in §3. 17 ACs. 7 full Rust test bodies. 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — RLS not added to TENANT_SCOPED_TABLES registry (FR-AUTH-003 §1 #1)
First-pass enabled RLS on layer2_memories but didn't update FR-AUTH-003 registry. New tenant create wouldn't auto-provision policy. Resolved: §1 #8 + 0001_layer2.sql includes USING + WITH CHECK; explicit registry update in modified_files.

### ISS-002 — BGE retry budget unspecified
First-pass §10 said "ingest retries with backoff" without budget. Could retry indefinitely. Resolved: §1 #10 + 5 attempts at 100/250/500/1000/2000ms; pending_embed_retry state for backfill.

### ISS-003 — Frame-level corruption handling unspecified
First-pass had no CRC check. Bad frame would block ingest indefinitely. Resolved: §1 #12 + chain_anchor::validate_frame; skip + sev-2; AC #11 + §5 test.

### ISS-004 — Graceful shutdown unspecified
SIGTERM mid-transaction would leave pgvector + AGE inconsistent. Resolved: §1 #14 + drain pattern; AC #13.

### ISS-005 — Concurrent multi-tenant ingest pattern unspecified
First-pass had single loop iterating tenants. Doesn't scale. Resolved: §1 #13 + tokio task per tenant + JoinSet; AC #12.

### ISS-006 — chain_anchor verification on read not specified
First-pass §1 #4 added chain_anchor at write but didn't say read paths verify. The whole point is read-time tamper detection. Resolved: §3 chain_anchor::verify; AC #10 + §5 test asserts sev-1 on mismatch.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-MEMORY-101 audit.*
