---
fr_id: FR-EMAIL-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

EMAIL CaMeL dual-LLM prompt-injection defense per Google DeepMind 2025 paper. P-LLM/Q-LLM split, opaque variables, policy gate, CISO-audited trust list. 320 lines, 13 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 6 notes. 8 issues resolved (Q-LLM no-tool-access by structural design, variable opacity (P-LLM never sees raw email), policy gate per tool call, trust-list bypass requires CISO audit row, outcome enum cardinality 4, audit log immutable (no UPDATE/DELETE grant), variable TTL 24h, integration with FR-EMAIL-008 wraps AI calls). **Score = 10/10.**

*End of FR-EMAIL-005 audit.*
