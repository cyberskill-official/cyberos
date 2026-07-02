---
fr_id: FR-HR-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

HR performance signal aggregator (PROJ + TIME + LEARN read-only) with monthly EOM snapshots + immutability + 5-signal enum. 230 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (read-only enforced via allowed_tools disallow-list, signal_kind enum cardinality 5, snapshots immutable via REVOKE UPDATE/DELETE, UNIQUE(member, period_end) idempotency, missing source data → null signal + sev-2 (no lie), burnout flag 3-month rolling). **Score = 10/10.**

*End of FR-HR-008 audit.*
