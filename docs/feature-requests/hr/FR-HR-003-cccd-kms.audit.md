---
fr_id: FR-HR-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

HR CCCD photo with per-tenant KMS keyspace + ROOT-CHRO-only decrypt + sev-1 audit + consent gate + PDPL Law 91/2025 Art. 23 compliance. 250 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (separate KMS keyspace per tenant, ROOT-CHRO-only decrypt with 403 on non-CHRO, sev-1 audit + CHRO email on decrypt, consent_token required for upload (PDPL Art. 23), access_kind enum cardinality 4, append-only access log, encrypted_photo never in BRAIN chain). **Score = 10/10.**

*End of FR-HR-003 audit.*
