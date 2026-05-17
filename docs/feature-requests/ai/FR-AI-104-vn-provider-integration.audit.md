---
fr_id: FR-AI-104
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

AI VN provider integration (Viettel Cloud + FPT Cloud) for Vn1-residency with failover + CTO-gated KMS creds. 230 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (vn_provider enum cardinality 2 (extensible), Viettel primary + FPT failover, both-down refusal preserves FR-AI-016 regulatory contract, CTO-only KMS creds, refusal codes distinct (vn1_provider_outage vs vn1_no_provider_yet), append-only via REVOKE). **Score = 10/10.**

*End of FR-AI-104 audit.*
