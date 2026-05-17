---
fr_id: FR-CRM-004
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM deal→engagement conversion with idempotency + bi-dir backlink + rate-card binding + AM assignment + recognition_method choice. 230 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (UNIQUE(deal_id) idempotency, bi-dir backlink (deal.converted_engagement_id + eng.source_deal_id), conversion_source enum cardinality 3, append-only via REVOKE UPDATE, stage-revert no auto-unconvert (audit only), PII scrub deal_value SHA256). **Score = 10/10.**

*End of FR-CRM-004 audit.*
