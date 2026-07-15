---
task_id: TASK-DOC-010
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

DOC third-party import (DocuSign/Adobe Sign/HelloSign) with LTV preservation + idempotency + KMS creds + async. 280 lines, 13 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (LTV preservation — byte-identical storage, no new signature; UNIQUE(provider, source_doc_id, tenant_id) idempotency; CLO-only creds in KMS; import_source enum cardinality 4; LTV invalid flagged not rejected; TASK-DOC-007 lifecycle metadata populated). **Score = 10/10.**

*End of TASK-DOC-010 audit.*
