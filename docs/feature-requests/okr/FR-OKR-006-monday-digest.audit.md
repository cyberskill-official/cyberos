---
fr_id: FR-OKR-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

OKR Monday CUO digest with 4 sections + opt-in recipients + email+chat delivery + idempotent per-week. 240 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (digest_delivery enum cardinality 4, recipient opt-in table, UNIQUE(tenant, iso_week) idempotency, CUO failure degrades to raw + sev-2, per-recipient delivery isolation, append-only via REVOKE). **Score = 10/10.**

*End of FR-OKR-006 audit.*
