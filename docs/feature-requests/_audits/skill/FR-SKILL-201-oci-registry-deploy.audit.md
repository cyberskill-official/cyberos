---
fr_id: FR-SKILL-201
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

SKILL OCI registry deploy with cosign signing + tag immutability + tenant ACL + yank lifecycle. 240 lines, 13 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (bundle_status enum cardinality 6, UNIQUE(registry, image, tag) tag immutability, cosign verify on pull with sev-1 on mismatch, tenant_acl JSONB enforcement, yank preserves bundle for audit, append-only via REVOKE). **Score = 10/10.**

*End of FR-SKILL-201 audit.*
