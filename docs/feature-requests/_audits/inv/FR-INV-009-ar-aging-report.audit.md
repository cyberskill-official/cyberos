---
fr_id: FR-INV-009
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

AR aging with 6 buckets + as-of determinism + outstanding_balance + multi-currency. 240 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (as_of_date determinism enforced — reject `now()`, outstanding_balance not full amount per partial-paid, FX missing → nearest-prior sev-2, bucket edge correctness 0/1/30/31/etc, PII scrubbed via SHA256(total), pagination for tenants >10k invoices). **Score = 10/10.**

*End of FR-INV-009 audit.*
