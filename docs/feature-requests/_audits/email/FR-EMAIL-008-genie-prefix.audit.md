---
fr_id: FR-EMAIL-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

EMAIL Genie-prefix routing with FR-PORTAL-005 Branded Genie + action review queue + FR-MCP-006 tool gating. 290 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (NEVER auto-execute, fetch_data gated by FR-MCP-006 tool allowlist, action_kind enum cardinality 6, brand-pack context for tone, prefix match case-insensitive + strips Re:/Fwd:, PII scrub body/AI output SHA256, append-only via REVOKE UPDATE except review/result). **Score = 10/10.**

*End of FR-EMAIL-008 audit.*
