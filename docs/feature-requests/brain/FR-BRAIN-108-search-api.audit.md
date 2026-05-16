---
fr_id: FR-BRAIN-108
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-BRAIN-108 expanded from 88 lines to ~810. Added 6 §1 clauses (#10 chain_anchor verify per result with sev-1; #11 JWT auth; #12 empty results = 200; #13 multi-language support; #14 OTel metrics; expanded #8 with all-3-fail handling). 8 §2 rationale paragraphs. Full Rust types + 3 backend modules + RRF + rerank + chain_anchor_verify + ACL filter in §3. 18 ACs. 9 full Rust test bodies including p95 benchmark. PGroonga migration. 20 failure modes. 10 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — chain_anchor verification on results not specified
First-pass mentioned chain_anchor at write but not at read. Whole point is read-time tamper detection. Resolved: §1 #10 + chain_anchor_verify.rs + sev-1 metric + AC #15 + §5 test.

### ISS-002 — All-3-fail handling unspecified
First-pass §10 row "BGE down → fallback full-text" but didn't specify what happens when all 3 fail. Resolved: §1 #8 + 503 SERVICE_UNAVAILABLE + AllBackendsFailed; AC #10.

### ISS-003 — Vietnamese tokenisation mechanism unspecified
First-pass §1 #2 said "PGroonga (Vietnamese-aware)" without tokeniser config. Resolved: §1 #2 + DEC-198 + 0003_pgroonga.sql with TokenMecab + custom dictionary; AC #11.

### ISS-004 — JWT auth not specified
First-pass had no auth path. Resolved: §1 #11 FR-AUTH-004 JWT; tenant_id + actor_id from claims; AC #16.

### ISS-005 — Explain payload schema undefined
First-pass §1 #9 mentioned `?explain` without payload structure. Resolved: §3 ExplainPayload struct + AC #14 + §8 example response.

### ISS-006 — Empty results status unspecified
Should empty be 200 or 404? Important for caller logic. Resolved: §1 #12 + AC #12; 200 with `[]` (search semantics).

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-BRAIN-108 audit.*
