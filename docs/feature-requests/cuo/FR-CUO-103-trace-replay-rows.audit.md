---
fr_id: FR-CUO-103
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CUO trace replay rows with prompt+model+temp+seed capture + immutable + drift detection. 200 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (trace_call_kind enum cardinality 4, immutable via REVOKE UPDATE/DELETE, replay match/drift audit lifecycle, NULL seed handling for vendor-unsupported models, monthly partitioning aligned with FR-CUO-102, PII scrub prompt+response SHA256). **Score = 10/10.**

*End of FR-CUO-103 audit.*
