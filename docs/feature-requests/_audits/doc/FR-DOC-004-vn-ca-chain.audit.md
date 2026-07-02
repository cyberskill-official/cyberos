---
fr_id: FR-DOC-004
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

DOC VN CA chain (VnPay + MK Group + Viettel-CA) with VNeID linkage + VN Root CA validation + Decree 130/2018 compliance. 320 lines, 15 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (CISO-only KMS creds, vn_ca_partner enum cardinality 3, vn_ca_request_kind enum cardinality 5, VNeID required for qualified signature (Decree 130), VN Root CA validation enforced (non-VN chain rejected), append-only sigs, vneid_subject_id SHA256 in PII chain). **Score = 10/10.**

*End of FR-DOC-004 audit.*
