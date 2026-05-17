---
fr_id: FR-MCP-002
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

MCP server heartbeat lifecycle with 10s interval + 3-miss-unhealthy + auto-recovery + skill cascade. 200 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (server_health_status enum cardinality 4, 3-miss threshold prevents false-positive, auto-recovery on next heartbeat, skill_unavailable cascade to FR-MCP-001, UNIQUE(tenant, module_name), monitor cron 5s for low-latency detection). **Score = 10/10.**

*End of FR-MCP-002 audit.*
