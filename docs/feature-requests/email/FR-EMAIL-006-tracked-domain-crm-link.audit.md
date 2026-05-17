---
fr_id: FR-EMAIL-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

EMAIL tracked-domain → CRM auto-link with per-tenant allowlist + auto-contact creation + FR-AI-003 company inference. 240 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (untracked silent skip, existing-contact reuse not duplicate, link_origin enum 4 closed + cardinality test, PII scrub email/name SHA256, case-insensitive domain match, FR-AI-003 lookup 24h cached). **Score = 10/10.**

*End of FR-EMAIL-006 audit.*
