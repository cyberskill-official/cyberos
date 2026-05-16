---
fr_id: FR-DOC-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-DOC-001 ships the DOC document repository — S3 Object-Lock Compliance + per-tenant residency pinning + versioned + ACL'd + hash-chained audit + 10-year+ retention. Scope: 26 §1 normative clauses covering 2 closed Postgres enums (bucket_scope 7, document_status 4), append-only versions, RLS, S3 Object-Lock Compliance mode application on archive, residency-pinned bucket selection (FR-AI-016), hash-chained per-document audit log with chain-integrity trigger, dual-signoff legal hold (CLO + CSO), GDPR/PDPL erasure block under hold, cross-scope move forbidden, retention immutable post-archive, REST surface with presigned upload URLs + finalize step + SHA-256 integrity check, 8 BRAIN audit row kinds with sev assignments, OTel emission, scope-based retention periods (HR 50y, ESOP 75y, others 10y), explicit `purpose` parameter on access. 22 rationale paragraphs. §3 contains: migration 0001 (metadata + versions + 2 enums + 2 triggers + RLS + REVOKE + retention policy function), migration 0002 (audit log + chain integrity trigger + replayer), Rust types, residency resolver, S3 Object-Lock application, legal-hold REST handler. 28 ACs. 33 failure-mode rows. 24 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Object-Lock Governance allowed root override
First-pass had no specific mode. Resolved: §1 #7 + DEC-280 + Compliance mode hard-coded; `Mode=Governance` rejected at CI.

### ISS-002 — Residency pinning bypassable
First-pass let operators choose any bucket. Resolved: §1 #8 + DEC-281 + residency resolver + AC #8 + #9 (residency_mismatch).

### ISS-003 — Cross-scope move shifted retention silently
First-pass allowed bucket_scope mutation. Resolved: §1 #14 + DEC-292 + trigger `cross_scope_move_forbidden`; AC #5.

### ISS-004 — Audit chain tamper-detection missing
First-pass had no hash chain. Resolved: §1 #10 + DEC-283 + `chain_hash`/`prev_hash` + `enforce_audit_chain_integrity` trigger; AC #19 + #20.

### ISS-005 — Legal hold single-signer
First-pass had no dual-signoff requirement. Resolved: §1 #12 + DEC-285 + CLO + CSO co-sign + `self_co_sign` check; AC #15 + #16.

### ISS-006 — Erasure ignored legal hold
First-pass had no predicate. Resolved: §1 #13 + `is_erasure_blocked` predicate; AC #17.

### ISS-007 — Retention period drift
First-pass had a single hard-coded retention. Resolved: §1 #7 + DEC-287 + `bucket_retention_policy` SQL function with per-scope years (HR 50, ESOP 75, others 10); AC #12 + #13 + #14.

### ISS-008 — Access read untracked
First-pass had no GET audit. Resolved: §1 #23 + DEC-290 + sev-2 `doc.access_audited` row + required `purpose` parameter.

### ISS-009 — `retention_until` mutable post-archive
Resolved: §1 #15 + DEC-287 + trigger `retention_immutable_post_archive`; AC #11.

### ISS-010 — Versions UPDATE-able by app code
Resolved: §1 #6 + DEC-286 + `REVOKE UPDATE, DELETE FROM cyberos_app`; AC #6 + #7.

### ISS-011 — Integrity gap between client SHA + S3 ETag
First-pass trusted client-supplied hash. Resolved: §1 #17 + finalize step validates against actual S3 ETag; AC #22.

## §3 — Resolution

All 11 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (2 closed enums × RLS × append-only versions × Object-Lock Compliance × per-tenant residency × hash-chained audit × dual-signoff legal hold × erasure block × cross-scope forbidden × retention immutability × presigned upload + finalize integrity check × 8 BRAIN audit kinds × OTel), not by line targets.

---

*End of FR-DOC-001 audit.*
