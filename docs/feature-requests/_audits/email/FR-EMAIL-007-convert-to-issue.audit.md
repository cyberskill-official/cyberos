---
fr_id: FR-EMAIL-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

EMAIL convert-to-issue with bi-dir backlink + AI summary + attachment refs + 3 source modes. 260 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (attachments via doc_links not S3 copy, bi-dir backlink (linked_issue_id + source_thread_id), convert_source enum cardinality 3, AI fallback to subject on timeout, multiple-convert allowed by design, PII scrub body/subject SHA256). **Score = 10/10.**

*End of FR-EMAIL-007 audit.*
