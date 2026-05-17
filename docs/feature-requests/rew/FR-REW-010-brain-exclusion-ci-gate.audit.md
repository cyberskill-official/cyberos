---
fr_id: FR-REW-010
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

REW BRAIN structural exclusion CI gate with static grep + runtime payload check + field blocklist + sev-1 audit. 220 lines, 8 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (exclusion_check_kind enum cardinality 4, CI grep + runtime defense-in-depth, runtime check rejects comp fields, sev-1 audit on violation, audit body structurally excludes comp, CI cannot be bypassed). **Score = 10/10.**

*End of FR-REW-010 audit.*
