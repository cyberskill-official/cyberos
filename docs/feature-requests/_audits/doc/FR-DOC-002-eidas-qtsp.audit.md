---
fr_id: FR-DOC-002
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

DOC eIDAS QTSP integration (GlobalSign + Cryptomathic) with partner abstraction + PAdES-B-LT + cert chain validation + EU Trust List. 320 lines, 13 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (CISO-only KMS creds, partner enum cardinality 2 (extensible), request_kind enum cardinality 4, PAdES-B-LT format with LTV chain, append-only sigs table, OCSP/CRL revocation checked + revoked-cert detection, sandbox/prod env separation). **Score = 10/10.**

*End of FR-DOC-002 audit.*
