---
fr_id: FR-REW-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

REW quarterly P3 distribution with FR-LEARN-007 VP shares + CEO+CFO dual-sign + idempotent + sum-matches-fund invariant. 240 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (distribution_status enum cardinality 6, CEO+CFO dual-sign with separation, UNIQUE(tenant, quarter) idempotency, sum-of-payouts matches fund (±1 VND), append-only via REVOKE except status cols, fund_vnd > 0 CHECK). **Score = 10/10.**

*End of FR-REW-008 audit.*
