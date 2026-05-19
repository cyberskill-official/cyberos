---
fr_id: FR-REW-001
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

REW 3P income schema (P1/P2/P3, 8-kind) with separate KMS keyspace + ROOT-CFO decrypt + immutable + sev-1 audit on access. 260 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (income_kind enum cardinality 8, separate KMS keyspace from HR per defense-in-depth, CFO-only decrypt with sev-1 audit, append-only via REVOKE UPDATE/DELETE, correction via correction_of pointer, encrypted_amount never in memory chain, CFO email notification on decrypt). **Score = 10/10.**

*End of FR-REW-001 audit.*
