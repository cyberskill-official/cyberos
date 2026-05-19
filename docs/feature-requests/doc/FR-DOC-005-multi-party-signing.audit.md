---
fr_id: FR-DOC-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

DOC multi-party signing (ordered/parallel/counter-sign) with FR-DOC-006 verify gate + per-region CA routing + reminder cadence. 360 lines, 13 §1 clauses, 20 ACs, 4 tests, 10 failure modes, 5 notes. 8 issues resolved (verify-before-sign invariant, per-region CA routing (VN→DOC-004 / EU→DOC-002 / other→DOC-003), workflow_kind enum cardinality 3, signature_status enum cardinality 6, decline blocks workflow → failed, append-only via REVOKE except status cols, signed payload SHA256 in memory, reminder cron 24/72/168h configurable). **Score = 10/10.**

*End of FR-DOC-005 audit.*
