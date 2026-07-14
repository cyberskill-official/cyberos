---
task_id: TASK-APP-004
audited: 2026-06-29
verdict: PASS
score: 9.5/10
template: task@1
rubric: task-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP MCP registry and tools panel - a panel inside the TASK-APP-001 operator console (the same static single-page app, the same auth gate, the same CDS tokens) that surfaces mcp-gateway operator state: the federated modules and their tool catalogs (`tools/list`, TASK-MCP-001), per-module server health in the closed `healthy` / `degraded` / `unhealthy` / `deregistered` set (`GET /v1/mcp/servers`, TASK-MCP-002), the aggregate counts (`GET /mcp/healthz`), and the OAuth clients plus Protected Resource Metadata (TASK-MCP-004 / TASK-MCP-005). It reads only mcp-gateway endpoints that already ship, adds no backend, and is read-only in its first release with tool-triggering named in Out of scope.

Frontmatter (FM-001..111, FM-004): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal task@1; id TASK-APP-004; title 96 chars (FM-101 ok); author @stephen (FM-102 ok); department engineering; status draft; priority p3; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists; depends_on cites the real ids TASK-APP-001, TASK-AUTH-004, and the mcp-gateway FRs TASK-MCP-001..006 (verified against docs/tasks/mcp/, no invented ids). Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 12 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - fold into the TASK-APP-001 ai-gateway-health panel, a standalone MCP admin app, and a trigger-now-with-a-new-`app`-backend; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing TASK-APP-001, the six mcp-gateway FRs, TASK-AUTH-004, and the Caddy front (QA-008 ok). Heading hierarchy is well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): the first release is read-only over the mcp-gateway and tool-triggering is deferred, so the panel shows the federation and its health before it can act on it; and the destructive-tool safety rule is a standing forward contract (clause 8) rather than an exercised path, since clause 7 keeps the first release from triggering any tool at all. Both are disclosed in Scope and in the normative clauses (clauses 7 and 8) rather than hidden, and the health-state and annotation fidelity rules (clauses 5 and 6) keep the read views honest to what the gateway actually returns.

Verdict: PASS. Ready for implementation of the four read views inside the TASK-APP-001 auth-gated CDS shell; the tool-trigger screen is the named follow-up, and when it lands it routes destructive tools through the TASK-MCP-006 confirmation gate per clause 8.

*End of TASK-APP-004 audit.*
