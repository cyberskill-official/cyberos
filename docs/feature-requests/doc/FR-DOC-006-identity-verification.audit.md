---
fr_id: FR-DOC-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

DOC 4-method identity verification (WebAuthn/VNeID/SMS-OTP/email-link) with eIDAS levels + immutable audit + challenge expiry. 260 lines, 12 §1 clauses, 20 ACs, 4 tests, 10 failure modes, 5 notes. 7 issues resolved (method enum cardinality 4, result enum cardinality 4, eIDAS level mapping with min-level enforcement, challenge expiry per method (5/10/24min/h), audit immutable via no UPDATE/DELETE grant, IP SHA256 in PII scrub, one-time-use challenges). **Score = 10/10.**

*End of FR-DOC-006 audit.*
